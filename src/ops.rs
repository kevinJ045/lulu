use crate::lulu::{Lulu, LuluModSource};
use mlua::Lua;
use mlua::prelude::LuaError;
use regex::Regex;
use std::fs;
use std::io::Read;
use tokio::time;

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
      if let Some(module) = lulu.mods.iter().find(|m| m.name == format!("bytes://{}", name)) {
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
  lua.globals().set("regex_match", regex_match_all)?;

  Ok(())
}
