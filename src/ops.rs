use crate::lulu::Lulu;
use mlua::Lua;
use mlua::prelude::LuaError;
use regex::Regex;
// use std::sync::{Arc, Mutex};
use tokio::time;
// use std::rc::Rc;
// use std::cell::RefCell;

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

  let sleep_fn = lua.create_async_function({
    async move |_, secs: u64| -> mlua::Result<()> {
      time::sleep(time::Duration::from_secs(secs)).await;
      println!("Sleep over");
      Ok(())
    }
  })?;
  lua.globals().set("sleep", sleep_fn)?;

  lua.globals().set("get_mods", gmods)?;
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

  let ptr_free = lua.create_function(|lua, ptr: usize| {
    unsafe {
      drop(Box::from_raw(ptr as *mut mlua::RegistryKey));
    }
    lua.expire_registry_values();
    Ok(())
  })?;
  lua.globals().set("ptr_free", ptr_free)?;

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
