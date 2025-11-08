use crate::lulibs::bytes::LuluByteArray;
use crate::lulibs::collections::{LuluHashMap, LuluHashSet};
use crate::ops::process::register_exec;
use crate::ops::std::create_std;
use crate::package_manager::PackageManager;
use crate::{core::Lulu, core::LuluModSource, ops::std::get_std_module};
use mlua::Lua;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::sync::{Arc, Mutex};
use tokio::time;

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
    lua.create_async_function(move |lua, (url, only_reg): (String, Option<bool>)| {
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
                .ok_or_else(|| mlua::Error::RuntimeError(format!("No init was found")))?;

              if let Some(only_reg) = only_reg {
                if only_reg {
                  return Ok(mlua::Value::String(lua.create_string(modname)?));
                } else {
                  let tbl = lua.create_table()?;
                  tbl.set("__into", modname.clone().split("/").collect::<Vec<_>>()[0])?;
                  tbl.set("__value", lulu_clone.exec_mod(modname.as_str())?)?;

                  return Ok(mlua::Value::Table(tbl));
                }
              }
              return Ok(lulu_clone.exec_mod(modname.as_str())?);
            }
          }
        }

        eprintln!(
          "Dynamic requirement \"{}\" did not provide a proper lulib.",
          url
        );
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
        local res = nil
        local f = coroutine.create(function(...)
          res = require_cached_async(...)
          return false
        end)
        local done = true
        while done do
          done = coroutine.resume(f, ...)
        end
        return res
      })
      .into_function()?,
  )?;
  lua.globals().set(
    "sync_call",
    lua
      .load(mlua::chunk! {
        local res = nil
        local args = {...}
        local fn = args[1]
        table.remove(args, 1)
        local f = coroutine.create(function(...)
          res = fn(...)
          return false
        end)
        local done = true
        while done do
          done = coroutine.resume(f, unpack(args))
        end
        return res
      })
      .into_function()?,
  )?;

  create_std(lua)?;

  Ok(())
}

pub fn register_consts(lua: &Lua) -> mlua::Result<()> {
  lua.globals().set("CURRENT_OS", std::env::consts::OS)?;

  lua.globals().set("CURRENT_ARCH", std::env::consts::ARCH)?;

  lua
    .globals()
    .set("CURRENT_FAMILY", std::env::consts::FAMILY)?;

  lua.globals().set("LULU_VER", env!("CARGO_PKG_VERSION"))?;

  Ok(())
}
