use crate::lulu::{Lulu, LuluModSource};
use crate::package_manager::PackageManager;
use mlua::Lua;
use mlua::prelude::LuaError;
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
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::{Read, Write};
use tokio::time;
use zip::write::ExtendedFileOptions;
use zip::{ZipWriter, write::FileOptions};

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
  std.set("tar", tar_mod)?;
  std.set("zip", zip_mod)?;

  let net_mod = lua.create_table()?;
  let http_mod = lua.create_table()?;

  let client = Client::builder()
    .user_agent("Lulu/1.0")
    .build()
    .map_err(LuaError::external)?;
  let client_ref = lua.create_any_userdata(client)?;

  http_mod.set(
    "request",
    lua.create_async_function(
      |lua,
       (url, method, body, headers): (
        String,
        Option<String>,
        Option<String>,
        Option<HashMap<String, String>>,
      )| async move {
        let client = lua.globals().get::<mlua::AnyUserData>("__reqwest_client")?;
        let client = client.borrow::<Client>()?;
        let mut req = client.request(
          Method::from_bytes(method.unwrap_or("GET".to_string()).as_bytes())
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
        if let Some(b) = body {
          req = req.body(b);
        }
        let resp = req.send().await.map_err(LuaError::external)?;
        let status = resp.status().as_u16();
        let bytes = resp.bytes().await.map_err(LuaError::external)?;

        let res = lua.create_table_from([("body", bytes.to_vec())])?;
        res.set("status", status)?;
        Ok(res)
      },
    )?,
  )?;

  lua.globals().set("__reqwest_client", client_ref)?;
  net_mod.set("http", http_mod)?;
  std.set("net", net_mod)?;

  let serde_mod = lua.create_table()?;
  let json_mod = lua.create_table()?;
  json_mod.set(
    "decode",
    lua.create_function(|_, text: String| {
      serde_json::from_str::<serde_json::Value>(&text)
        .map(|v| format!("{:?}", v))
        .map_err(mlua::Error::external)
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
    lua.create_function(|_, text: String| {
      serde_yaml::from_str::<serde_yaml::Value>(&text)
        .map(|v| format!("{:?}", v))
        .map_err(mlua::Error::external)
    })?,
  )?;
  yaml_mod.set(
    "encode",
    lua.create_function(|_, val: mlua::Table| {
      serde_yaml::to_string(&val).map_err(LuaError::external)
    })?,
  )?;
  serde_mod.set("yaml", yaml_mod)?;
  std.set("serde", serde_mod)?;

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

  let bytes_from_mods = {
    let lulu_rc = lulu.clone();
    lua.create_function(move |_, name: String| {
      let lulu = &lulu_rc;
      if let Some(module) = lulu
        .mods
        .iter()
        .find(|m| m.name == format!("bytes://{}", name))
      {
        match &module.source {
          LuluModSource::Bytecode(bytes) => Ok(bytes.clone()),
          LuluModSource::Code(code) => Ok(code.as_bytes().to_vec()),
        }
      } else {
        Err(mlua::Error::RuntimeError(format!(
          "Module '{}' not found",
          name
        )))
      }
    })
  }?;

  let sleep_fn = lua.create_async_function({
    async move |_, secs: u64| -> mlua::Result<()> {
      time::sleep(time::Duration::from_secs(secs)).await;
      println!("Sleep over");
      Ok(())
    }
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
    lua.create_function(|lua, path: String| {
      let mut file = fs::File::open(&path)?;
      let mut buffer = Vec::new();
      file.read_to_end(&mut buffer)?;
      lua.create_string(&buffer)
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
    "require_cached",
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
              crate::bundle::reg_bundle_nods(&mut lulu_clone, mods)?;
            }
          }
        }

        let modname = lulu_clone.find_mod("init")?;

        println!("{}", modname);

        Ok(lulu_clone.exec_mod(modname.as_str()))
      }
    })?,
  )?;

  create_std(lua)?;

  Ok(())
}
