use crate::lulu::{Lulu, LuluModSource};
use crate::package_manager::PackageManager;
use mlua::prelude::LuaError;
use mlua::{Lua, LuaSerdeExt};
use rand::rngs::StdRng;
use rand::{Rng, RngCore, SeedableRng};
use regex::Regex;
use reqwest::Method;
use reqwest::{
  Client,
  header::{HeaderMap, HeaderName, HeaderValue},
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::{Read, Write};
use std::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time;
use zip::write::ExtendedFileOptions;
use zip::{ZipWriter, write::FileOptions};

#[derive(Clone)]
pub struct LuluByteArray {
  pub bytes: Vec<u8>,
}

impl mlua::UserData for LuluByteArray {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_method("to_table", |_, this, ()| Ok(this.bytes.clone()));
    methods.add_method("len", |_, this, ()| Ok(this.bytes.len()));
    methods.add_method("to_hex", |_, this, ()| {
      Ok(
        this
          .bytes
          .iter()
          .map(|b| format!("{:02x}", b))
          .collect::<String>(),
      )
    });

    methods.add_method("to_string", |_lua, this, encoding: Option<String>| {
      let enc_name = encoding.unwrap_or_else(|| "utf-8".to_string());
      match enc_name.to_lowercase().as_str() {
        "utf-8" => Ok(String::from_utf8_lossy(&this.bytes).to_string()),
        _ => Err(mlua::Error::RuntimeError(format!(
          "Unsupported encoding '{}'",
          enc_name
        ))),
      }
    });

    methods.add_method_mut("extend", |_, this, other: mlua::AnyUserData| {
      let other_bytes = other.borrow::<LuluByteArray>()?;
      this.bytes.extend(&other_bytes.bytes);
      Ok(())
    });

    methods.add_method_mut("extend_table", |_, this, other: Vec<u8>| {
      this.bytes.extend(other);
      Ok(())
    });

    methods.add_method_mut("push", |_, this, byte: u8| {
      this.bytes.push(byte);
      Ok(())
    });

    methods.add_method_mut("pop", |_, this, ()| Ok(this.bytes.pop()));

    methods.add_method_mut("clear", |_, this, ()| {
      this.bytes.clear();
      Ok(())
    });

    methods.add_method("slice", |_, this, (start, stop): (usize, usize)| {
      let start = start.saturating_sub(1);
      let stop = stop.min(this.bytes.len());
      Ok(LuluByteArray {
        bytes: this.bytes[start..stop].to_vec(),
      })
    });

    methods.add_method("copy", |_, this, ()| {
      Ok(LuluByteArray {
        bytes: this.bytes.clone(),
      })
    });

    methods.add_method("new", |_, _, bytes: Vec<u8>| Ok(LuluByteArray { bytes }));

    methods.add_method("map", |_, this, func: mlua::Function| {
      let mapped = this
        .bytes
        .iter()
        .map(|b| func.call::<u8>(*b).unwrap_or(*b))
        .collect();
      Ok(LuluByteArray { bytes: mapped })
    });
  }
}

#[derive(Clone)]
struct LuluHashSet {
  items: HashSet<usize>, // store RegistryKey pointers as usize
}

impl mlua::UserData for LuluHashSet {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_method_mut("add", |lua, this, value: mlua::Value| {
      let key = lua.create_registry_value(value)?;
      this.items.insert(Box::into_raw(Box::new(key)) as usize);
      Ok(())
    });

    methods.add_method_mut("remove", |lua, this, value: mlua::Value| {
      // scan items and remove the one that matches `value`
      let mut to_remove = None;
      for &ptr in &this.items {
        let key_ref = unsafe { &*(ptr as *mut mlua::RegistryKey) };
        let v = lua.registry_value::<mlua::Value>(key_ref)?;
        if v == value {
          to_remove = Some(ptr);
          break;
        }
      }
      if let Some(ptr) = to_remove {
        this.items.remove(&ptr);
        drop(unsafe { Box::from_raw(ptr as *mut mlua::RegistryKey) }); // drop the key
      }
      Ok(())
    });

    methods.add_method("has", |lua, this, value: mlua::Value| {
      for &ptr in &this.items {
        let key_ref = unsafe { &*(ptr as *mut mlua::RegistryKey) };
        let v = lua.registry_value::<mlua::Value>(key_ref)?;
        if v == value {
          return Ok(true);
        }
      }
      Ok(false)
    });

    methods.add_method("values", |lua, this, _: ()| {
      let tbl = lua.create_table()?;
      for (i, &ptr) in this.items.iter().enumerate() {
        let key_ref = unsafe { &*(ptr as *mut mlua::RegistryKey) };
        let value = lua.registry_value::<mlua::Value>(key_ref)?;
        tbl.set(i + 1, value)?;
      }
      Ok(tbl)
    });

    methods.add_method_mut("clear", |_, this, _: ()| {
      for &ptr in &this.items {
        drop(unsafe { Box::from_raw(ptr as *mut mlua::RegistryKey) });
      }
      this.items.clear();
      Ok(())
    });
  }
}

#[derive(Clone)]
struct LuluHashMap {
  items: HashMap<usize, usize>, // key ptr → value ptr
}

impl mlua::UserData for LuluHashMap {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_method_mut(
      "set",
      |lua, this, (key, value): (mlua::Value, mlua::Value)| {
        let key_ptr = Box::into_raw(Box::new(lua.create_registry_value(key)?)) as usize;
        let value_ptr = Box::into_raw(Box::new(lua.create_registry_value(value)?)) as usize;
        this.items.insert(key_ptr, value_ptr);
        Ok(())
      },
    );

    methods.add_method("get", |lua, this, key: mlua::Value| {
      for (&k_ptr, &v_ptr) in &this.items {
        let k_ref = unsafe { &*(k_ptr as *mut mlua::RegistryKey) };
        let k_val = lua.registry_value::<mlua::Value>(k_ref)?;
        if k_val == key {
          let v_ref = unsafe { &*(v_ptr as *mut mlua::RegistryKey) };
          return Ok(lua.registry_value::<mlua::Value>(v_ref)?);
        }
      }
      Ok(mlua::Value::Nil)
    });

    methods.add_method("has", |lua, this, key: mlua::Value| {
      for (&k_ptr, _) in &this.items {
        let k_ref = unsafe { &*(k_ptr as *mut mlua::RegistryKey) };
        if lua.registry_value::<mlua::Value>(k_ref)? == key {
          return Ok(true);
        }
      }
      Ok(false)
    });

    methods.add_method_mut("remove", |lua, this, key: mlua::Value| {
      let mut to_remove = None;
      for (&k_ptr, &v_ptr) in &this.items {
        let k_ref = unsafe { &*(k_ptr as *mut mlua::RegistryKey) };
        if lua.registry_value::<mlua::Value>(k_ref)? == key {
          to_remove = Some((k_ptr, v_ptr));
          break;
        }
      }
      if let Some((k_ptr, v_ptr)) = to_remove {
        this.items.remove(&k_ptr);
        drop(unsafe { Box::from_raw(k_ptr as *mut mlua::RegistryKey) });
        drop(unsafe { Box::from_raw(v_ptr as *mut mlua::RegistryKey) });
      }
      Ok(())
    });
  }
}

fn split_command(s: &str) -> Vec<String> {
  let mut parts = Vec::new();
  let mut cur = String::new();
  let mut chars = s.chars().peekable();
  let mut in_single = false;
  let mut in_double = false;

  while let Some(ch) = chars.next() {
    match ch {
      '\\' => {
        if let Some(next) = chars.next() {
          cur.push(next);
        }
      }
      '\'' if !in_double => {
        in_single = !in_single;
      }
      '"' if !in_single => {
        in_double = !in_double;
      }
      c if c.is_whitespace() && !in_single && !in_double => {
        if !cur.is_empty() {
          parts.push(cur.clone());
          cur.clear();
        }
      }
      c => cur.push(c),
    }
  }

  if !cur.is_empty() {
    parts.push(cur);
  }

  parts
}

fn create_std(lua: &Lua) -> mlua::Result<()> {
  let std = lua.globals();

  let crypto = lua.create_table()?;
  let sha2 = lua.create_function(|_, data: String| {
    let mut hasher = Sha256::new();
    hasher.update(data);
    Ok(format!("{:x}", hasher.finalize()))
  })?;
  crypto.set("sha256", sha2)?;
  std.set("crypto", crypto)?;

  let uuid_mod = lua.create_table()?;
  uuid_mod.set(
    "v4",
    lua.create_function(|_, ()| Ok(uuid::Uuid::new_v4().to_string()))?,
  )?;
  std.set("uuid", uuid_mod)?;

  let rand = lua.create_table()?;
  let rand_from = lua.create_function(|_, (min, max, seed): (usize, usize, Option<String>)| {
    let mut rng: Box<dyn RngCore> = match seed {
      Some(s) => {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        s.hash(&mut hasher);
        Box::new(StdRng::seed_from_u64(hasher.finish()))
      }
      _ => Box::new(rand::thread_rng()),
    };

    if min == max {
      return Ok(min);
    }

    let (low, high) = if min < max { (min, max) } else { (max, min) };

    Ok(rng.gen_range(low..=high))
  })?;
  rand.set("from", rand_from)?;
  std.set("rand", rand)?;

  let re_mod = lua.create_table()?;
  re_mod.set(
    "exec",
    lua.create_function(|_, (pattern, text): (String, String)| {
      let re = Regex::new(&pattern).map_err(LuaError::external)?;
      Ok(re.is_match(&text))
    })?,
  )?;

  let regex_match_all = lua.create_function(|lua, (pattern, text): (String, String)| {
    let re =
      Regex::new(&pattern).map_err(|e| LuaError::external(format!("Invalid regex: {}", e)))?;

    if let Some(caps) = re.captures(&text) {
      let results = lua.create_table()?;
      for i in 0..caps.len() {
        if let Some(m) = caps.get(i) {
          results.set(i + 1, m.as_str())?;
        } else {
          results.set(i + 1, mlua::Value::Nil)?;
        }
      }

      Ok(Some(results))
    } else {
      Ok(None)
    }
  })?;
  re_mod.set("match", regex_match_all)?;

  re_mod.set(
    "replace",
    lua.create_function(
      |lua, (pattern, text, replacement): (String, String, mlua::Value)| {
        let re = Regex::new(&pattern).map_err(LuaError::external)?;

        match replacement {
          mlua::Value::String(s) => {
            let repl_str = s.to_str()?;
            let result = re.replace_all(&text, |caps: &regex::Captures| {
              let mut s = repl_str.to_string();
              for i in 0..caps.len() {
                let placeholder = format!("${}", i);
                if let Some(m) = caps.get(i) {
                  s = s.replace(&placeholder, m.as_str());
                }
              }
              s
            });
            Ok(result.to_string())
          }

          mlua::Value::Function(f) => {
            let result = re.replace_all(&text, |caps: &regex::Captures| {
              let mut args = Vec::with_capacity(caps.len());
              args.push(caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string());
              for i in 1..caps.len() {
                args.push(caps.get(i).map(|m| m.as_str()).unwrap_or("").to_string());
              }

              match f.call::<String>(
                args
                  .into_iter()
                  .map(|s| mlua::Value::String(lua.create_string(&s).unwrap()))
                  .collect::<mlua::MultiValue>(),
              ) {
                Ok(s) => s,
                Err(_) => "".to_string(),
              }
            });
            Ok(result.to_string())
          }

          _ => Err(LuaError::external(
            "replacement must be a string or a function",
          )),
        }
      },
    )?,
  )?;
  std.set("re", re_mod)?;

  Ok(())
}

fn register_exec(lua: &Lua) -> mlua::Result<()> {
  let exec = lua.create_function(|lua, (command, inherit): (String, Option<bool>)| {
    let parts = split_command(&command);
    if parts.is_empty() {
      return Err(LuaError::external("empty command"));
    }

    let program = &parts[0];
    let args = &parts[1..];

    let inherit = inherit.unwrap_or(false);

    if inherit {
      let status = std::process::Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(LuaError::external)?;

      let result = lua.create_table()?;
      result.set("status", status.code().unwrap_or(-1))?;
      result.set("success", status.success())?;

      Ok(mlua::Value::Table(result))
    } else {
      let output = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(LuaError::external)?;

      let result = lua.create_table()?;
      result.set(
        "stdout",
        String::from_utf8_lossy(&output.stdout).to_string(),
      )?;
      result.set(
        "stderr",
        String::from_utf8_lossy(&output.stderr).to_string(),
      )?;
      result.set("status", output.status.code().unwrap_or(-1))?;
      result.set("success", output.status.success())?;

      Ok(mlua::Value::Table(result))
    }
  })?;

  lua.globals().set("exec", exec)?;
  Ok(())
}

pub fn register_ops(lua: &Lua, lulu: &Lulu) -> mlua::Result<()> {
  let mods = lulu.mods.clone();

  let gmods = lua.create_function(move |_, ()| {
    let mut embedded_scripts: Vec<String> = Vec::new();

    for lmod in &mods {
      embedded_scripts.push(lmod.name.clone());
    }

    Ok(embedded_scripts)
  })?;

  let execmod = lua.create_function({
    let lulu_rc = lulu.clone();
    move |_, name: String| {
      let lulu = &lulu_rc;
      lulu.exec_mod(&name)
    }
  })?;

  lua.globals().set(
    "request_env_load",
    lua.create_function({
      let lulu_rc = lulu.clone();
      let imported = Arc::new(Mutex::new(Vec::new()));
      move |lua, (env, name): (String, Option<String>)| {
        if let Some(module) = get_std_module(&env) {
          let mut imports = imported.lock().unwrap();
          let name = if let Some(name) = name {
            name.clone()
          } else {
            env.clone()
          };
          if imports.contains(&name) {
            return Ok(mlua::Value::Boolean(true));
          }
          module.register(lua)?;
          imports.push(name);
          if !module.deps.is_empty() {
            let tbl = lua.create_table()?;
            tbl.set("__include", module.deps.clone())?;
            return Ok(mlua::Value::Table(tbl));
          }
          return Ok(mlua::Value::Boolean(true));
        }
        if let Ok(modules) = lua
          .globals()
          .get::<mlua::Table>("package")?
          .get::<HashMap<String, mlua::Value>>("preload")
        {
          if let Some(mn) = if let Some(_) = modules.get(&env) {
            Some(env.clone())
          } else if let Some(_) = modules.get(&format!("{}/init", env)) {
            Some(format!("{}/init", env))
          } else {
            None
          } {
            let lulu = &lulu_rc;
            let module = lulu.exec_mod(&mn)?;

            let tbl = lua.create_table()?;
            tbl.set("__into", name)?;
            tbl.set("__value", module)?;

            return Ok(mlua::Value::Table(tbl));
          }
        }
        Ok(mlua::Value::Boolean(false))
      }
    })?,
  )?;

  let bytes_from_mods = {
    let lulu_rc = lulu.clone();
    lua.create_function(move |_, name: String| {
      let lulu = &lulu_rc;
      if let Some(module) = lulu.mods.iter().find(|m| {
        m.name
          == (if name.starts_with("bytes://") {
            name.clone()
          } else {
            format!("bytes://{}", name)
          })
      }) {
        let bytes = match &module.source {
          LuluModSource::Bytecode(bytes) => bytes.clone(),
          LuluModSource::Code(code) => code.as_bytes().to_vec(),
        };
        Ok(LuluByteArray { bytes })
      } else {
        Err(mlua::Error::RuntimeError(format!(
          "Module '{}' not found",
          name
        )))
      }
    })
  }?;

  let sleep_fn = lua.create_async_function(async move |_, secs: u64| -> mlua::Result<()> {
    time::sleep(time::Duration::from_secs(secs)).await;
    Ok(())
  })?;
  lua.globals().set("sleep", sleep_fn)?;

  lua.globals().set("__get_mods__", gmods)?;
  lua.globals().set("bytes_from", bytes_from_mods)?;
  lua.globals().set("exec_mod", execmod)?;
  lua.globals().set("argv", lulu.args.clone())?;

  let ptr_of = lua.create_function(|lua, value: mlua::Value| {
    let ptr = lua.create_registry_value(value)?;
    Ok(Box::into_raw(Box::new(ptr)) as usize)
  })?;
  lua.globals().set("ptr_of", ptr_of)?;

  let ptr_deref = lua.create_function(|lua, ptr: usize| {
    let value_ptr = ptr as *mut mlua::RegistryKey;
    if value_ptr.is_null() {
      return Ok(mlua::Value::Nil);
    }
    let key = unsafe { &*value_ptr };
    let value = lua.registry_value::<mlua::Value>(&key)?;
    Ok(value)
  })?;
  lua.globals().set("ptr_deref", ptr_deref)?;

  let ptr_set = lua.create_function(|lua, (ptr, new_val): (usize, mlua::Value)| {
    let value_ptr = ptr as *mut mlua::RegistryKey;
    if value_ptr.is_null() {
      return Err(mlua::Error::RuntimeError("Null pointer".to_string()));
    }
    let key = unsafe { &mut *value_ptr };
    lua.replace_registry_value(key, new_val.clone())?;
    Ok(new_val)
  })?;
  lua.globals().set("ptr_set", ptr_set)?;

  lua.globals().set(
    "reads",
    lua.create_function(|_, path: String| Ok(fs::read_to_string(path)?))?,
  )?;

  lua.globals().set(
    "exists",
    lua.create_function(|_, path: String| Ok(std::path::Path::new(&path).exists()))?,
  )?;

  lua.globals().set(
    "mkdir",
    lua.create_function(|_, path: String| {
      fs::create_dir_all(&path)?;
      Ok(())
    })?,
  )?;

  lua.globals().set(
    "cp",
    lua.create_function(|_, (src, dest): (String, String)| {
      fs::copy(&src, &dest)?;
      Ok(())
    })?,
  )?;

  lua.globals().set(
    "rename",
    lua.create_function(|_, (old, new): (String, String)| {
      fs::rename(&old, &new)?;
      Ok(())
    })?,
  )?;

  lua.globals().set(
    "mv",
    lua.create_function(|_, (src, dest): (String, String)| {
      fs::copy(&src, &dest)?;
      fs::remove_file(&src)?;
      Ok(())
    })?,
  )?;

  lua.globals().set(
    "rm",
    lua.create_function(|_, path: String| {
      let p = std::path::Path::new(&path);
      if p.is_dir() {
        fs::remove_dir_all(p)?;
      } else if p.is_file() {
        fs::remove_file(p)?;
      }
      Ok(())
    })?,
  )?;

  lua.globals().set(
    "read",
    lua.create_function(|_, path: String| {
      let mut file = fs::File::open(&path)?;
      let mut buffer = Vec::new();
      file.read_to_end(&mut buffer)?;
      Ok(LuluByteArray { bytes: buffer })
    })?,
  )?;

  register_exec(lua)?;

  lua.globals().set(
    "exit",
    lua.create_function(|_, code: Option<i32>| {
      std::process::exit(code.unwrap_or(0));
      #[allow(unreachable_code)]
      Ok(())
    })?,
  )?;

  lua.globals().set(
    "foreach",
    lua.create_function(|lua, items: Vec<mlua::Value>| {
      Ok(lua.create_function(move |_, func: mlua::Function| {
        let mut mapped: Vec<mlua::Value> = Vec::new();
        for i in items.clone() {
          mapped.push(func.call(i)?)
        }
        Ok(mapped)
      }))
    })?,
  )?;

  lua.globals().set(
    "range",
    lua.create_function(|_, (x, y, z): (usize, usize, Option<usize>)| {
      let nums: Vec<usize> = (x..=y).step_by(z.unwrap_or(1) as usize).collect();
      Ok(nums)
    })?,
  )?;

  let lulu_clone = lulu.clone();

  lua.globals().set(
    "require_cached_async",
    lua.create_async_function(move |_, url: String| {
      let mut lulu_clone = lulu_clone.clone();
      async move {
        let pkg_manager = PackageManager::new().map_err(mlua::Error::external)?;

        let cache_path = pkg_manager.get_package_cache_path(&url);
        if !pkg_manager.is_cached(&url) {
          pkg_manager
            .fetch_package(&url, &cache_path)
            .await
            .map_err(mlua::Error::external)?;
          pkg_manager
            .build_package(&cache_path)
            .await
            .map_err(mlua::Error::external)?;
        }

        let cache_lulib_dir = cache_path.join(".lib");
        if cache_lulib_dir.exists() {
          for entry in std::fs::read_dir(&cache_lulib_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file()
              && entry.path().extension().and_then(|s| s.to_str()) == Some("lulib")
            {
              let mods = crate::bundle::load_lulib(&entry.path())?;
              crate::bundle::reg_bundle_nods(&mut lulu_clone, mods.clone())?;

              let (modname, _) = mods
                .iter()
                .find(|(m, _)| m.ends_with("init"))
                .ok_or_else(|| mlua::Error::RuntimeError(format!("No main was found")))?;

              return Ok(lulu_clone.exec_mod(modname.as_str())?);
            }
          }
        }

        Ok(mlua::Value::Nil)
      }
    })?,
  )?;

  let set_ctor = lua.create_function(|_, ()| {
    Ok(LuluHashSet {
      items: HashSet::new(),
    })
  })?;
  lua.globals().set("HashSet", set_ctor)?;

  let map_ctor = lua.create_function(|_, ()| {
    Ok(LuluHashMap {
      items: HashMap::new(),
    })
  })?;
  lua.globals().set("HashMap", map_ctor)?;
  lua.globals().set(
    "ByteArray",
    lua.create_function(|_, bytes: Vec<u8>| Ok(LuluByteArray { bytes }))?,
  )?;
  lua.globals().set(
    "exec_sandboxed",
    lua.create_function(
      |lua, (code, name, env): (String, Option<String>, Option<mlua::Table>)| {
        let mut chunk = lua.load(code);

        if let Some(name) = name {
          chunk = chunk.set_name(name);
        }

        if let Some(env) = env {
          chunk = chunk.set_environment(env);
        } else {
          chunk = chunk.set_environment(lua.create_table()?);
        }

        chunk.eval::<mlua::Value>()
      },
    )?,
  )?;

  lua.globals().set(
    "setup_downloader",
    lua.create_function(|lua, options: Option<mlua::Table>| {
      let mut pm = PackageManager::new().map_err(|e| {
        eprintln!("Failed to initialize package manager: {}", e);
        mlua::Error::external(e)
      })?;
      if let Some(options) = options {
        if let Ok(format) = options.get::<String>("format") {
          pm.downloader.format = format;
        }

        if let Ok(download_text) = options.get::<String>("download_text") {
          pm.downloader.download_text = download_text;
        }

        if let Ok(progress_bar_size) = options.get::<usize>("progressbar_size") {
          pm.downloader.progress_bar_size = progress_bar_size;
        }

        if let Ok(progress_bar_colors) = options.get::<Vec<u8>>("progressbar_colors") {
          pm.downloader.progress_bar_colors = (
            (
              progress_bar_colors[0],
              progress_bar_colors[1],
              progress_bar_colors[2],
            ),
            (
              progress_bar_colors[3],
              progress_bar_colors[4],
              progress_bar_colors[5],
            ),
          );
        }
      }
      lua
        .globals()
        .set("__lulu_pac_man", lua.create_any_userdata(pm)?)?;
      Ok(())
    })?,
  )?;

  lua.globals().set(
    "download_file",
    lua.create_async_function(async |lua, url: String| {
      let pm = lua.globals().get::<mlua::AnyUserData>("__lulu_pac_man")?;
      let pm = pm.borrow::<PackageManager>()?;
      pm.download_file(&url).await.map_err(|e| {
        eprintln!("Failed to download file: {}", e);
        mlua::Error::external(e)
      })
    })?,
  )?;
  lua.globals().set(
    "download_uncached",
    lua.create_async_function(async |lua, (url, path): (String, String)| {
      let pm = lua.globals().get::<mlua::AnyUserData>("__lulu_pac_man")?;
      let pm = pm.borrow::<PackageManager>()?;
      pm.download_url(&url, &std::path::Path::new(&path))
        .await
        .map_err(|e| {
          eprintln!("Failed to download file: {}", e);
          mlua::Error::external(e)
        })
    })?,
  )?;
  lua.globals().set(
    "require_cached",
    lua
      .load(mlua::chunk! {
        local f = coroutine.create(function(...)
          require_cached_async(...)
          return false
        end)
        local done = true
        while done do
          done = coroutine.resume(f, ...)
        end
      })
      .into_function()?,
  )?;
  lua.globals().set(
    "sync_call",
    lua
      .load(mlua::chunk! {
        local args = {...}
        local fn = args[1]
        table.remove(args, 1)
        local f = coroutine.create(function(...)
          fn(...)
          return false
        end)
        local done = true
        while done do
          done = coroutine.resume(f, unpack(args))
        end
      })
      .into_function()?,
  )?;

  create_std(lua)?;

  Ok(())
}

pub fn register_consts(lua: &Lua) -> mlua::Result<()> {
  lua.globals().set("CURRENT_OS", std::env::consts::OS)?;

  lua.globals().set("CURRENT_ARCH", std::env::consts::ARCH)?;

  lua.globals().set("LULU_VER", env!("CARGO_PKG_VERSION"))?;

  Ok(())
}

#[derive(Default)]
pub struct STDModule {
  pub name: String,
  pub deps: Vec<String>,
  pub functions: HashMap<String, Box<dyn Fn(&Lua) -> mlua::Result<mlua::Function> + Send + Sync>>,
  pub files: Vec<(String, String)>,
  pub macros: Vec<(String, Vec<String>, String)>,
  pub on_register:
    Option<Box<dyn Fn(&Lua, mlua::Table) -> mlua::Result<mlua::Table> + Send + Sync>>,
}

impl STDModule {
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      functions: HashMap::new(),
      files: Vec::new(),
      macros: Vec::new(),
      deps: Vec::new(),
      on_register: None,
    }
  }

  #[allow(unused)]
  pub fn add_function<T, R, F>(mut self, name: impl Into<String>, func: F) -> Self
  where
    T: mlua::FromLuaMulti + Send + 'static,
    R: mlua::IntoLuaMulti + Send + 'static,
    F: Fn(&Lua, T) -> mlua::Result<R> + Clone + Send + Sync + 'static,
  {
    let name = name.into();
    self.functions.insert(
      name,
      Box::new(move |lua| Ok(lua.create_function(func.clone())?)),
    );
    self
  }

  #[allow(unused)]
  pub fn add_file(mut self, path: impl Into<String>, content: impl Into<String>) -> Self {
    self
      .files
      .push((path.into(), crate::compiler::compile(&content.into())));
    self
  }

  #[allow(unused)]
  pub fn add_macro(
    mut self,
    name: impl Into<String>,
    args: impl Into<Vec<String>>,
    content: impl Into<String>,
  ) -> Self {
    self.macros.push((name.into(), args.into(), content.into()));
    self
  }

  #[allow(unused)]
  pub fn depend_on(mut self, name: String) -> Self {
    self.deps.push(name);
    self
  }

  pub fn on_register<F>(mut self, callback: F) -> Self
  where
    F: Fn(&Lua, mlua::Table) -> mlua::Result<mlua::Table> + Send + Sync + 'static,
  {
    self.on_register = Some(Box::new(callback));
    self
  }

  pub fn register(&self, lua: &Lua) -> mlua::Result<()> {
    let tbl = lua.create_table()?;

    for (name, make_fn) in &self.functions {
      tbl.set(name.as_str(), (make_fn)(lua)?)?;
    }

    if let Some(cb) = &self.on_register {
      lua.globals().set(self.name.as_str(), (cb)(lua, tbl)?)?;
    } else {
      lua.globals().set(self.name.as_str(), tbl)?;
    }

    for (path, content) in &self.files {
      lua.load(content).set_name(path).exec()?;
    }

    Ok(())
  }

  pub fn into(self) -> Arc<Self> {
    let name = self.name.to_string();
    let module = Arc::new(self);
    STD_MODULES
      .write()
      .unwrap()
      .insert(name.to_string(), module.clone());
    module
  }
}

use axum::{
  Router,
  extract::{Request, State},
  http::StatusCode,
  response::{IntoResponse, Response},
  routing::any,
};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::Mutex as TokioMutex;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{WebSocketStream, connect_async, tungstenite::protocol::Message};

lazy_static::lazy_static! {
  pub static ref STD_MODULES: RwLock<HashMap<String, Arc<STDModule>>> =
      RwLock::new(HashMap::new());
}

lazy_static::lazy_static! {
  pub static ref TOK_ASYNC_HANDLES: Mutex<Vec<JoinHandle<()>>> = Mutex::new(Vec::new());
}

#[derive(Clone)]
struct LuluTcpStream {
  reader: Arc<TokioMutex<ReadHalf<TcpStream>>>,
  writer: Arc<TokioMutex<WriteHalf<TcpStream>>>,
}

impl LuluTcpStream {
  fn new(stream: TcpStream) -> Self {
    let (reader, writer) = tokio::io::split(stream);
    Self {
      reader: Arc::new(TokioMutex::new(reader)),
      writer: Arc::new(TokioMutex::new(writer)),
    }
  }
}

impl mlua::UserData for LuluTcpStream {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_async_method("read", |_, this, n: Option<usize>| async move {
      let mut reader = this.reader.lock().await;
      let n = n.unwrap_or(1024);
      let mut buf = vec![0; n];
      match reader.read(&mut buf).await {
        Ok(0) => Ok(None), // EOF
        Ok(bytes_read) => {
          buf.truncate(bytes_read);
          Ok(Some(LuluByteArray { bytes: buf }))
        }
        Err(e) => Err(mlua::Error::external(e)),
      }
    });

    methods.add_async_method("write", |_, this, data: mlua::Value| async move {
      let mut writer = this.writer.lock().await;
      let bytes = match data {
        mlua::Value::String(s) => s.as_bytes().to_vec(),
        mlua::Value::UserData(ud) => ud.borrow::<LuluByteArray>()?.bytes.clone(),
        _ => return Err(mlua::Error::external("string or ByteArray")),
      };
      writer
        .write_all(&bytes)
        .await
        .map_err(mlua::Error::external)?;
      Ok(())
    });

    methods.add_async_method("close", |_, this, ()| async move {
      let mut writer = this.writer.lock().await;
      writer.shutdown().await.map_err(mlua::Error::external)?;
      Ok(())
    });
  }
}

#[derive(Clone)]
struct LuluTcpListener {
  listener: Arc<TcpListener>,
}

impl mlua::UserData for LuluTcpListener {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_async_method("accept", |_, this, ()| async move {
      let (socket, _) = this
        .listener
        .accept()
        .await
        .map_err(mlua::Error::external)?;
      Ok(LuluTcpStream::new(socket))
    });
  }
}

#[derive(Clone)]
struct LuluUdpSocket {
  socket: Arc<UdpSocket>,
}

impl LuluUdpSocket {
  fn new(socket: UdpSocket) -> Self {
    Self {
      socket: Arc::new(socket),
    }
  }
}

impl mlua::UserData for LuluUdpSocket {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_async_method(
      "send_to",
      |_, this, (addr, data): (String, mlua::Value)| async move {
        let bytes = match data {
          mlua::Value::String(s) => s.as_bytes().to_vec(),
          mlua::Value::UserData(ud) => ud.borrow::<LuluByteArray>()?.bytes.clone(),
          _ => return Err(mlua::Error::external("string or ByteArray")),
        };
        let sent = this
          .socket
          .send_to(&bytes, &addr)
          .await
          .map_err(mlua::Error::external)?;
        Ok(sent)
      },
    );

    methods.add_async_method("recv_from", |lua, this, n: Option<usize>| async move {
      let n = n.unwrap_or(65535);
      let mut buf = vec![0; n];
      let (len, addr) = this
        .socket
        .recv_from(&mut buf)
        .await
        .map_err(mlua::Error::external)?;
      buf.truncate(len);
      let result = lua.create_table()?;
      result.set("data", LuluByteArray { bytes: buf })?;
      result.set("addr", addr.to_string())?;
      Ok(result)
    });
  }
}

#[derive(Clone)]
struct LuluWebSocket {
  stream:
    Arc<TokioMutex<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>>,
}

impl LuluWebSocket {
  fn new(
    stream: WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
  ) -> Self {
    Self {
      stream: Arc::new(TokioMutex::new(stream)),
    }
  }
}

impl mlua::UserData for LuluWebSocket {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_async_method("read", |lua, this, ()| async move {
      let mut stream = this.stream.lock().await;
      match stream.next().await {
        Some(Ok(msg)) => {
          match msg {
            Message::Text(t) => Ok(mlua::Value::String(lua.create_string(&t)?)),
            Message::Binary(b) => Ok(mlua::Value::UserData(
              lua.create_userdata(LuluByteArray { bytes: b.to_vec() })?,
            )),
            _ => Ok(mlua::Value::Nil), // Ignore Ping/Pong/Frame/Close
          }
        }
        Some(Err(e)) => Err(mlua::Error::external(e)),
        _ => Ok(mlua::Value::Nil), // Stream closed
      }
    });

    methods.add_async_method("write", |_, this, data: mlua::Value| async move {
      let mut stream = this.stream.lock().await;
      let msg = match data {
        mlua::Value::String(s) => Message::Text(s.to_str()?.to_string().into()),
        mlua::Value::UserData(ud) => {
          Message::Binary(ud.borrow::<LuluByteArray>()?.bytes.clone().into())
        }
        _ => return Err(mlua::Error::external("string or ByteArray")),
      };
      stream.send(msg).await.map_err(mlua::Error::external)?;
      Ok(())
    });

    methods.add_async_method("close", |_, this, ()| async move {
      let mut stream = this.stream.lock().await;
      stream.close(None).await.map_err(mlua::Error::external)?;
      Ok(())
    });
  }
}

#[derive(Clone)]
pub struct LuluThreadHandle {
  pub handle: Arc<Mutex<Option<JoinHandle<mlua::Result<mlua::Value>>>>>,
}

impl mlua::UserData for LuluThreadHandle {}

#[derive(Clone)]
pub struct LuluSledDB {
  pub db: sled::Db,
}

#[derive(Clone)]
pub struct LuluSledDBMulti {
  pub db: sled::Db,
  pub table_name: String,
  pub indexed_fields: Vec<String>,
}

impl mlua::UserData for LuluSledDB {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_method("set", |_, this, (key, value): (String, mlua::Value)| {
      let bytes = serde_json::to_vec(&value).map_err(|e| mlua::Error::external(e))?;
      this.db.insert(key, bytes).map_err(mlua::Error::external)?;
      this.db.flush().map_err(mlua::Error::external)?;
      Ok(())
    });

    methods.add_method("get", |lua, this, key: String| {
      if let Some(bytes) = this.db.get(key).map_err(mlua::Error::external)? {
        let data: serde_json::Value =
          serde_json::from_slice(&bytes).map_err(mlua::Error::external)?;
        let lua_val = lua.to_value(&data)?;
        Ok(lua_val)
      } else {
        Ok(mlua::Value::Nil)
      }
    });

    methods.add_method("remove", |_, this, key: String| {
      this.db.remove(key).map_err(mlua::Error::external)?;
      this.db.flush().map_err(mlua::Error::external)?;
      Ok(())
    });

    methods.add_method("contains", |_, this, key: String| {
      Ok(this.db.contains_key(key).map_err(mlua::Error::external)?)
    });

    methods.add_method(
      "table",
      |_, this, (table_name, indexed_fields): (String, Vec<String>)| {
        Ok(LuluSledDBMulti {
          db: this.db.clone(),
          table_name,
          indexed_fields,
        })
      },
    );

    methods.add_method("id", |_, this, ()| {
      Ok(this.db.generate_id().map_err(mlua::Error::external)?)
    });
  }
}

fn json_gt(a: &serde_json::Value, b: &serde_json::Value) -> bool {
  match (a, b) {
    (serde_json::Value::Number(an), serde_json::Value::Number(bn)) => an.as_f64() > bn.as_f64(),
    (serde_json::Value::String(as_), serde_json::Value::String(bs)) => as_ > bs,
    _ => false, // other types not comparable
  }
}

fn json_lt(a: &serde_json::Value, b: &serde_json::Value) -> bool {
  match (a, b) {
    (serde_json::Value::Number(an), serde_json::Value::Number(bn)) => an.as_f64() < bn.as_f64(),
    (serde_json::Value::String(as_), serde_json::Value::String(bs)) => as_ < bs,
    _ => false,
  }
}

impl mlua::UserData for LuluSledDBMulti {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_method("insert", |_, this, value: mlua::Value| {
      let id = this.db.generate_id().map_err(mlua::Error::external)?;
      let key = format!("{}:{:020}", this.table_name, id);
      let bytes = serde_json::to_vec(&value).map_err(mlua::Error::external)?;
      this.db.insert(&key, bytes).map_err(mlua::Error::external)?;

      let mut json_val: serde_json::Value =
        serde_json::from_slice(&serde_json::to_vec(&value).map_err(mlua::Error::external)?)
          .map_err(mlua::Error::external)?;

      json_val["id"] = serde_json::Value::String(key.clone());
      let bytes = serde_json::to_vec(&json_val).map_err(mlua::Error::external)?;
      this.db.insert(&key, bytes).map_err(mlua::Error::external)?;

      for field in &this.indexed_fields {
        if let Some(field_value) = json_val.get(field) {
          let index_key = format!("{}:idx:{}:{}", this.table_name, field, field_value);
          let mut ids: Vec<String> = if let Some(bytes) = this.db.get(&index_key).unwrap() {
            serde_json::from_slice(&bytes).unwrap_or_default()
          } else {
            Vec::new()
          };
          ids.push(key.clone());
          this
            .db
            .insert(&index_key, serde_json::to_vec(&ids).unwrap())
            .unwrap();
        }
      }

      this.db.flush().map_err(mlua::Error::external)?;
      Ok(key)
    });

    methods.add_method(
      "find",
      |lua,
       this,
       (field, value, limit, offset): (String, mlua::Value, Option<usize>, Option<usize>)| {
        let value: serde_json::Value = lua.from_value(value)?;
        let prefix = format!("{}:idx:{}:", this.table_name, field);
        let mut results = Vec::new();
        let mut skipped = 0;
        let limit = limit.unwrap_or(usize::MAX);

        for item in this.db.scan_prefix(prefix.as_bytes()) {
          let (key, bytes) = item.map_err(mlua::Error::external)?;
          let key_str = String::from_utf8_lossy(&key);
          if !key_str.ends_with(&value.to_string()) {
            continue;
          }

          let ids: Vec<String> = serde_json::from_slice(&bytes).map_err(mlua::Error::external)?;
          for id in ids {
            if let Some(entry_bytes) = this.db.get(&id).map_err(mlua::Error::external)? {
              if skipped < offset.unwrap_or(0) {
                skipped += 1;
                continue;
              }
              if results.len() >= limit {
                break;
              }
              let data: serde_json::Value =
                serde_json::from_slice(&entry_bytes).map_err(mlua::Error::external)?;
              results.push(lua.to_value(&data)?);
            }
          }
        }

        Ok(mlua::Value::Table(lua.create_sequence_from(results)?))
      },
    );

    methods.add_method("index", |lua, this, id: String| {
      
      let key = if id.contains(':') {
        id
      } else {
        format!("{}:{}", this.table_name, id)
      };

      if let Some(bytes) = this.db.get(&key).map_err(mlua::Error::external)? {
        let data: serde_json::Value =
          serde_json::from_slice(&bytes).map_err(mlua::Error::external)?;
        Ok(lua.to_value(&data)?)
      } else {
        Ok(mlua::Value::Nil)
      }
    });

    methods.add_method(
      "lt",
      |lua, this, (field, max_value, limit, offset): (String, mlua::Value, Option<usize>, Option<usize>)| {
        let max_value: serde_json::Value = lua.from_value(max_value)?;
        let prefix = format!("{}:", this.table_name);
        let mut results = Vec::new();
        let mut skipped = 0;
        let limit = limit.unwrap_or(usize::MAX);

        for item in this.db.scan_prefix(prefix.as_bytes()) {
          let (key, value_bytes) = item.map_err(mlua::Error::external)?;
          let key_str = String::from_utf8_lossy(&key);
          if key_str.contains(":idx:") {
            continue;
          }
          let data: serde_json::Value = serde_json::from_slice(&value_bytes).map_err(mlua::Error::external)?;
          if let Some(field_val) = data.get(&field) {
            if json_lt(field_val, &max_value) {
              if skipped < offset.unwrap_or(0) {
                skipped += 1;
                continue;
              }
              if results.len() >= limit {
                break;
              }
              results.push(lua.to_value(&data)?);
            }
          }
        }

        Ok(mlua::Value::Table(lua.create_sequence_from(results)?))
      },
    );

    methods.add_method(
      "gt",
      |lua, this, (field, min_value, limit, offset): (String, mlua::Value, Option<usize>, Option<usize>)| {
        let min_value: serde_json::Value = lua.from_value(min_value)?;
        let prefix = format!("{}:", this.table_name);
        let mut results = Vec::new();
        let mut skipped = 0;
        let limit = limit.unwrap_or(usize::MAX);

        for item in this.db.scan_prefix(prefix.as_bytes()) {
          let (key, value_bytes) = item.map_err(mlua::Error::external)?;
          let key_str = String::from_utf8_lossy(&key);
          if key_str.contains(":idx:") {
            continue;
          }
          let data: serde_json::Value = serde_json::from_slice(&value_bytes).map_err(mlua::Error::external)?;
          if let Some(field_val) = data.get(&field) {
            if json_gt(field_val, &min_value) {
              if skipped < offset.unwrap_or(0) {
                skipped += 1;
                continue;
              }
              if results.len() >= limit {
                break;
              }
              results.push(lua.to_value(&data)?);
            }
          }
        }

        Ok(mlua::Value::Table(lua.create_sequence_from(results)?))
      },
    );

    methods.add_method(
      "all",
      |lua, this, (limit, offset): (Option<usize>, Option<usize>)| {
        let prefix = format!("{}:", this.table_name);
        let mut results = Vec::new();
        let mut skipped = 0;
        let limit = limit.unwrap_or(usize::MAX);

        for item in this.db.scan_prefix(prefix.as_bytes()) {
          let (key, value) = item.map_err(mlua::Error::external)?;
          let key_str = String::from_utf8_lossy(&key);
          if key_str.contains(":idx:") {
            continue;
          }

          if skipped < offset.unwrap_or(0) {
            skipped += 1;
            continue;
          }

          if results.len() >= limit {
            break;
          }

          let data: serde_json::Value =
            serde_json::from_slice(&value).map_err(mlua::Error::external)?;
          results.push(lua.to_value(&data)?);
        }

        Ok(mlua::Value::Table(lua.create_sequence_from(results)?))
      },
    );

    methods.add_method(
      "matches",
      |lua,
       this,
       (field, pattern, limit, offset): (String, String, Option<usize>, Option<usize>)| {
        let regex = Regex::new(&pattern).map_err(mlua::Error::external)?;
        let prefix = format!("{}:", this.table_name);
        let mut results = Vec::new();
        let mut skipped = 0;
        let limit = limit.unwrap_or(usize::MAX);

        for item in this.db.scan_prefix(prefix.as_bytes()) {
          let (key, value) = item.map_err(mlua::Error::external)?;
          let key_str = String::from_utf8_lossy(&key);
          if key_str.contains(":idx:") {
            continue;
          }

          let data: serde_json::Value =
            serde_json::from_slice(&value).map_err(mlua::Error::external)?;

          if let Some(field_val) = data.get(&field) {
            let val = field_val.to_string();
            let s = field_val.as_str().unwrap_or(&val);
            if regex.is_match(s) {
              if skipped < offset.unwrap_or(0) {
                skipped += 1;
                continue;
              }
              if results.len() >= limit {
                break;
              }
              results.push(lua.to_value(&data)?);
            }
          }
        }

        Ok(mlua::Value::Table(lua.create_sequence_from(results)?))
      },
    );

    methods.add_method("length", |_, this, _: ()| {
      let prefix = format!("{}:", this.table_name);
      let mut count = 0;
      for item in this.db.scan_prefix(prefix.as_bytes()) {
        let (key, _) = item.map_err(mlua::Error::external)?;
        let key_str = String::from_utf8_lossy(&key);
        if !key_str.contains(":idx:") {
          count += 1;
        }
      }
      Ok(count)
    });

    methods.add_method(
      "update",
      |_, this, (key, new_value): (String, mlua::Value)| {
        let old_bytes = this.db.get(&key).map_err(mlua::Error::external)?;
        if old_bytes.is_none() {
          return Err(mlua::Error::external("key not found"));
        }
        let old_json: serde_json::Value = serde_json::from_slice(&old_bytes.unwrap()).unwrap();
        let new_json: serde_json::Value =
          serde_json::from_slice(&serde_json::to_vec(&new_value).map_err(mlua::Error::external)?)
            .unwrap();

        for field in &this.indexed_fields {
          let old_val = old_json.get(field);
          let new_val = new_json.get(field);

          if old_val != new_val {
            if let Some(old_val) = old_val {
              let old_index_key = format!("{}:idx:{}:{}", this.table_name, field, old_val);
              if let Some(bytes) = this.db.get(&old_index_key).unwrap() {
                let mut ids: Vec<String> = serde_json::from_slice(&bytes).unwrap();
                ids.retain(|x| x != &key);
                this
                  .db
                  .insert(&old_index_key, serde_json::to_vec(&ids).unwrap())
                  .unwrap();
              }
            }

            if let Some(new_val) = new_val {
              let new_index_key = format!("{}:idx:{}:{}", this.table_name, field, new_val);
              let mut ids: Vec<String> = if let Some(bytes) = this.db.get(&new_index_key).unwrap() {
                serde_json::from_slice(&bytes).unwrap()
              } else {
                Vec::new()
              };
              ids.push(key.clone());
              this
                .db
                .insert(&new_index_key, serde_json::to_vec(&ids).unwrap())
                .unwrap();
            }
          }
        }

        let new_bytes = serde_json::to_vec(&new_json).unwrap();
        this.db.insert(&key, new_bytes).unwrap();
        this.db.flush().unwrap();

        Ok(())
      },
    );

    methods.add_method("remove", |_, this, key: String| {
      if let Some(bytes) = this.db.get(&key).unwrap() {
        let json_val: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        for field in &this.indexed_fields {
          if let Some(field_value) = json_val.get(field) {
            let index_key = format!("{}:idx:{}:{}", this.table_name, field, field_value);
            if let Some(index_bytes) = this.db.get(&index_key).unwrap() {
              let mut ids: Vec<String> = serde_json::from_slice(&index_bytes).unwrap();
              ids.retain(|x| x != &key);
              this
                .db
                .insert(&index_key, serde_json::to_vec(&ids).unwrap())
                .unwrap();
            }
          }
        }
        this.db.remove(&key).unwrap();
        this.db.flush().unwrap();
      }
      Ok(())
    });
  }
}

struct ServerRequest {
  req: Request,
  resp_tx: oneshot::Sender<Response>,
}

async fn axum_handler(State(req_tx): State<mpsc::Sender<ServerRequest>>, req: Request) -> Response {
  let (resp_tx, resp_rx) = oneshot::channel();
  let server_req = ServerRequest { req, resp_tx };

  if req_tx.send(server_req).await.is_err() {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      "Request handler has disconnected",
    )
      .into_response();
  }

  match resp_rx.await {
    Ok(resp) => resp,
    Err(e) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      format!("Request handler failed to respond: {e}"),
    )
      .into_response(),
  }
}

pub fn create_std_module(name: &str) -> STDModule {
  STDModule::new(name)
}

pub fn get_std_module(name: &str) -> Option<Arc<STDModule>> {
  STD_MODULES.read().unwrap().get(name).cloned()
}

pub fn init_std_modules() {
  create_std_module("kvdb")
    .add_function("open", |_, name: String| {
      Ok(LuluSledDB {
        db: sled::open(name).map_err(mlua::Error::external)?,
      })
    })
    .on_register(|_, db_mod| Ok(db_mod))
    .add_file(
      "kvdb.lua",
      std::fs::read_to_string("/home/makano/workspace/lulu/src/builtins/net/kvdb.lua").unwrap(),
    )
    .into();

  create_std_module("archive")
    .on_register(|lua, archive_mod| {
      let zip_mod = lua.create_table()?;
      zip_mod.set(
        "create",
        lua.create_function(|_, (archive_path, files): (String, Vec<String>)| {
          let file = File::create(&archive_path).map_err(|e| LuaError::external(e))?;
          let mut zip = ZipWriter::new(file);
          let options: FileOptions<ExtendedFileOptions> =
            FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

          for path in files {
            let mut f = File::open(&path).map_err(|e| LuaError::external(e))?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).map_err(|e| LuaError::external(e))?;
            zip
              .start_file(path.clone(), options.clone())
              .map_err(|e| LuaError::external(e))?;
            zip.write_all(&buf).map_err(|e| LuaError::external(e))?;
          }

          zip.finish().map_err(LuaError::external)?;
          Ok(())
        })?,
      )?;

      zip_mod.set(
        "extract",
        lua.create_function(|_, (archive_path, dest_dir): (String, String)| {
          let file = File::open(&archive_path).map_err(|e| LuaError::external(e))?;
          let mut archive = zip::ZipArchive::new(file).map_err(|e| LuaError::external(e))?;

          std::fs::create_dir_all(&dest_dir).ok();

          for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| LuaError::external(e))?;
            let out_path = std::path::Path::new(&dest_dir).join(file.name());

            if file.name().ends_with('/') {
              std::fs::create_dir_all(&out_path).ok();
            } else {
              if let Some(p) = out_path.parent() {
                std::fs::create_dir_all(p).ok();
              }
              let mut outfile = File::create(&out_path).map_err(|e| LuaError::external(e))?;
              std::io::copy(&mut file, &mut outfile).map_err(|e| LuaError::external(e))?;
            }
          }

          Ok(())
        })?,
      )?;

      use flate2::read::GzDecoder;
      use flate2::write::GzEncoder;
      let tar_mod = lua.create_table()?;

      tar_mod.set(
        "create",
        lua.create_function(|_, (archive_path, files): (String, Vec<String>)| {
          let tar_gz = File::create(&archive_path).map_err(|e| LuaError::external(e))?;
          let enc = GzEncoder::new(tar_gz, flate2::Compression::default());
          let mut tar = tar::Builder::new(enc);

          for path in files {
            tar.append_path(&path).map_err(|e| LuaError::external(e))?;
          }

          tar.into_inner().map_err(LuaError::external)?;
          Ok(())
        })?,
      )?;

      tar_mod.set(
        "extract",
        lua.create_function(|_, (archive_path, dest_dir): (String, String)| {
          let tar_gz = std::fs::File::open(&archive_path).map_err(|e| LuaError::external(e))?;
          let dec = GzDecoder::new(tar_gz);
          let mut archive = tar::Archive::new(dec);
          archive
            .unpack(std::path::Path::new(&dest_dir))
            .map_err(|e| LuaError::external(e))?;
          Ok(())
        })?,
      )?;

      archive_mod.set("tar", tar_mod)?;
      archive_mod.set("zip", zip_mod)?;
      Ok(archive_mod)
    })
    .into();

  create_std_module("serde")
    .on_register(|lua, serde_mod| {
      fn serde_value_to_lua(
        lua: &mlua::Lua,
        value: serde_json::Value,
      ) -> mlua::Result<mlua::Value> {
        use serde_json::Value::*;
        Ok(match value {
          Null => mlua::Value::Nil,
          Bool(b) => mlua::Value::Boolean(b),
          Number(n) => {
            if let Some(i) = n.as_i64() {
              mlua::Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
              mlua::Value::Number(f)
            } else {
              mlua::Value::Nil
            }
          }
          String(s) => mlua::Value::String(lua.create_string(&s)?),
          Array(arr) => {
            let tbl = lua.create_table()?;
            for (i, v) in arr.into_iter().enumerate() {
              tbl.set(i + 1, serde_value_to_lua(lua, v)?)?;
            }
            mlua::Value::Table(tbl)
          }
          Object(map) => {
            let tbl = lua.create_table()?;
            for (k, v) in map.into_iter() {
              tbl.set(k, serde_value_to_lua(lua, v)?)?;
            }
            mlua::Value::Table(tbl)
          }
        })
      }
      fn serde_into_json<V>(lua: &mlua::Lua, value: V) -> mlua::Result<mlua::Value>
      where
        V: serde::Serialize,
      {
        // Convert serde::Serialize -> serde_json::Value first
        let json_value = serde_json::to_value(value).map_err(mlua::Error::external)?;
        serde_value_to_lua(lua, json_value)
      }

      let json_mod = lua.create_table()?;
      json_mod.set(
        "decode",
        lua.create_function(|lua, text: String| {
          serde_value_to_lua(
            lua,
            serde_json::from_str::<serde_json::Value>(&text).map_err(mlua::Error::external)?,
          )
        })?,
      )?;
      json_mod.set(
        "encode",
        lua.create_function(|_, val: mlua::Table| {
          serde_json::to_string(&val).map_err(LuaError::external)
        })?,
      )?;
      serde_mod.set("json", json_mod)?;
      let yaml_mod = lua.create_table()?;
      yaml_mod.set(
        "decode",
        lua.create_function(|lua, text: String| {
          serde_into_json(
            lua,
            serde_yaml::from_str::<serde_yaml::Value>(&text).map_err(mlua::Error::external)?,
          )
        })?,
      )?;
      yaml_mod.set(
        "encode",
        lua.create_function(|_, val: mlua::Table| {
          serde_yaml::to_string(&val).map_err(LuaError::external)
        })?,
      )?;
      serde_mod.set("yaml", yaml_mod)?;

      Ok(serde_mod)
    })
    .into();

  create_std_module("net")
    .on_register(|lua, net_mod| {
      // HTTP client
      let http_mod = lua.create_table()?;
      let client = Client::builder()
        .user_agent("Lulu/1.0")
        .build()
        .map_err(LuaError::external)?;
      lua
        .globals()
        .set("__reqwest_client", lua.create_any_userdata(client)?)?;

      http_mod.set(
        "request",
        lua.create_async_function(|lua, req_table: mlua::Table| async move {
          let client = lua.globals().get::<mlua::AnyUserData>("__reqwest_client")?;
          let client = client.borrow::<Client>()?;

          let url: String = req_table.get("url")?;
          let method: Option<String> = req_table.get("method").ok();
          let body: Option<mlua::Value> = req_table.get("body").ok();
          let headers: Option<HashMap<String, String>> = req_table.get("headers").ok();

          let mut req = client.request(
            Method::from_bytes(method.unwrap_or_else(|| "GET".to_string()).as_bytes())
              .map_err(LuaError::external)?,
            &url,
          );

          if let Some(hmap) = headers {
            let mut hdrs = HeaderMap::new();
            for (k, v) in hmap {
              hdrs.insert(
                HeaderName::from_bytes(k.as_bytes()).map_err(mlua::Error::external)?,
                HeaderValue::from_str(&v).map_err(mlua::Error::external)?,
              );
            }
            req = req.headers(hdrs);
          }

          if let Some(body) = body {
            match body {
              mlua::Value::UserData(data) => {
                let data = data.borrow::<LuluByteArray>()?;
                req = req.body(data.bytes.clone());
              }
              mlua::Value::String(str) => req = req.body(str.to_str()?.to_string()),
              mlua::Value::Nil => {}
              _ => {
                eprintln!("Unsupported body type, only ByteArray and String is allowed.")
              }
            }
          }

          let resp = req.send().await.map_err(LuaError::external)?;
          let status = resp.status().as_u16();
          let res_headers: HashMap<String, String> = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
          let bytes = resp.bytes().await.map_err(LuaError::external)?;

          // Build Lua result
          let res = lua.create_table()?;
          res.set(
            "body",
            LuluByteArray {
              bytes: bytes.to_vec(),
            },
          )?;
          res.set("status", status)?;
          res.set("headers", res_headers)?;

          Ok(res)
        })?,
      )?;

      http_mod.set(
        "serve",
        lua.create_async_function(
          |lua, (addr, handler): (String, mlua::Function)| async move {
            let (req_tx, mut req_rx) = mpsc::channel::<ServerRequest>(32);
            let handler_key = lua.create_registry_value(handler)?;
            let lua = lua.clone();

            let app = Router::new()
              .fallback(any(axum_handler))
              .with_state(req_tx.clone());

            let socket_addr: SocketAddr = addr.parse().map_err(LuaError::external)?;
            let listener = tokio::net::TcpListener::bind(socket_addr)
              .await
              .map_err(LuaError::external)?;

            let listener = tokio::spawn(async move {
              if let Err(e) = axum::serve(listener, app).await {
                eprintln!("Server error: {}", e);
              }
            });

            tokio::spawn(async move {
              while let Some(server_req) = req_rx.recv().await {
                let handler = lua.registry_value::<mlua::Function>(&handler_key)?;
                let (parts, body) = server_req.req.into_parts();
                let body_bytes = axum::body::to_bytes(body, 1024 * 1024)
                  .await
                  .map_err(LuaError::external)?;

                let req_table = lua.create_table()?;
                req_table.set("method", parts.method.to_string())?;
                let uri = parts.uri.to_string();
                let host = parts
                  .headers
                  .get("host")
                  .and_then(|v| v.to_str().ok())
                  .unwrap_or("");
                req_table.set("host", host)?;
                req_table.set("uri", uri)?;

                let headers_table = lua.create_table()?;
                for (k, v) in parts.headers.iter() {
                  headers_table.set(k.to_string(), v.to_str().unwrap_or(""))?;
                }
                req_table.set("headers", headers_table)?;
                req_table.set(
                  "body",
                  LuluByteArray {
                    bytes: body_bytes.to_vec(),
                  },
                )?;

                let resp_handled = handler.call_async(req_table).await;

                match resp_handled.clone() {
                  Err(e) => {
                    eprintln!("{}", e);
                  }
                  _ => {}
                };

                let resp_table: mlua::Table = resp_handled?;
                let status: u16 = resp_table.get("status").unwrap_or(200);
                let body: mlua::Value = resp_table.get("body").unwrap_or(mlua::Value::Nil);
                let headers: HashMap<String, String> =
                  resp_table.get("headers").unwrap_or_default();

                let mut header_map = HeaderMap::new();
                for (k, v) in headers {
                  header_map.insert(
                    HeaderName::from_bytes(k.as_bytes()).map_err(LuaError::external)?,
                    HeaderValue::from_str(&v).map_err(LuaError::external)?,
                  );
                }

                let body_bytes = match body {
                  mlua::Value::String(s) => s.as_bytes().to_vec(),
                  mlua::Value::UserData(ud) => ud.borrow::<LuluByteArray>()?.bytes.clone(),
                  _ => Vec::new(),
                };

                let resp = Response::builder()
                  .status(StatusCode::from_u16(status).map_err(LuaError::external)?)
                  .header("x-powered-by", "Lulu")
                  .body(axum::body::Body::from(body_bytes))
                  .map_err(LuaError::external)?;

                let _ = server_req.resp_tx.send(resp);
              }
              Ok::<(), LuaError>(())
            });

            TOK_ASYNC_HANDLES.lock().unwrap().push(listener);
            Ok(())
          },
        )?,
      )?;

      net_mod.set("http", http_mod)?;

      // TCP
      let tcp_mod = lua.create_table()?;
      tcp_mod.set(
        "connect",
        lua.create_async_function(|_, addr: String| async move {
          let stream = TcpStream::connect(addr)
            .await
            .map_err(mlua::Error::external)?;
          Ok(LuluTcpStream::new(stream))
        })?,
      )?;
      tcp_mod.set(
        "listen",
        lua.create_async_function(|_, addr: String| async move {
          let listener = TcpListener::bind(addr)
            .await
            .map_err(mlua::Error::external)?;
          Ok(LuluTcpListener {
            listener: Arc::new(listener),
          })
        })?,
      )?;
      net_mod.set("tcp", tcp_mod)?;

      // UDP
      let udp_mod = lua.create_table()?;
      udp_mod.set(
        "bind",
        lua.create_async_function(|_, addr: String| async move {
          let socket = UdpSocket::bind(addr).await.map_err(mlua::Error::external)?;
          Ok(LuluUdpSocket::new(socket))
        })?,
      )?;
      net_mod.set("udp", udp_mod)?;

      // WebSocket
      let ws_mod = lua.create_table()?;
      ws_mod.set(
        "connect",
        lua.create_async_function(|_, url: String| async move {
          let (ws_stream, _) = connect_async(url).await.map_err(mlua::Error::external)?;
          Ok(LuluWebSocket::new(ws_stream))
        })?,
      )?;
      net_mod.set("websocket", ws_mod)?;

      Ok(net_mod)
    })
    .add_file(
      "net.lua",
      std::fs::read_to_string("/home/makano/workspace/lulu/src/builtins/net/net.lua").unwrap(),
    )
    .add_file(
      "http.lua",
      std::fs::read_to_string("/home/makano/workspace/lulu/src/builtins/net/http.lua").unwrap(),
    )
    .add_macro(
      "error_res",
      vec!["code".into(), "message".into()],
      "Response { status = $code, body = $message }",
    )
    .add_macro(
      "json_res",
      vec!["message".into()],
      "Response { status = 200, body = serde.json.encode($message) }",
    )
    .depend_on("serde".to_string())
    .into();

  create_std_module("threads")
    .on_register(|lua, threads_mod| {
      threads_mod.set(
        "spawn",
        lua.create_async_function(|lua, func: mlua::Function| async move {
          let handle = tokio::spawn(async move { func.call_async::<mlua::Value>(()).await });

          let handle = Arc::new(Mutex::new(Some(handle)));
          let handle_ref = handle.clone();

          TOK_ASYNC_HANDLES
            .lock()
            .unwrap()
            .push(tokio::spawn(async move {
              tokio::spawn(async move {
                let join_handle = {
                  let mut lock = handle.lock().unwrap();
                  lock.take()
                };

                if let Some(jh) = join_handle {
                  let _ = jh.await;
                }
              });
            }));

          Ok(lua.create_any_userdata(LuluThreadHandle { handle: handle_ref })?)
        })?,
      )?;

      // threads.join(task)
      threads_mod.set(
        "join",
        lua.create_async_function(|_, handle_ud: mlua::AnyUserData| async move {
          let handle_arc = {
            let handle = handle_ud.borrow::<LuluThreadHandle>()?;
            handle.handle.clone()
          };

          let join_handle_opt = {
            let mut opt = handle_arc.lock().unwrap();
            opt.take()
          };

          if let Some(join_handle) = join_handle_opt {
            let result = join_handle.await.map_err(LuaError::external)??;
            Ok(result)
          } else {
            Ok(mlua::Value::Nil)
          }
        })?,
      )?;

      threads_mod.set(
        "sleep",
        lua.create_async_function(|_, ms: u64| async move {
          tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
          Ok(())
        })?,
      )?;

      Ok(threads_mod)
    })
    .into();
}
