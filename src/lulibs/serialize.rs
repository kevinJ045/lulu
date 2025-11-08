use crate::ops::std::create_std_module;
use mlua::Error as LuaError;


pub fn into_module() {

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
}