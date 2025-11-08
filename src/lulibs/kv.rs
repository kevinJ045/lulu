use mlua::prelude::*;
use serde_json;
use regex::Regex;

use crate::ops::std::create_std_module;

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


pub fn into_module(){

  create_std_module("kvdb")
    .add_function("open", |_, name: String| {
      Ok(LuluSledDB {
        db: sled::open(name).map_err(mlua::Error::external)?,
      })
    })
    .on_register(|_, db_mod| Ok(db_mod))
    .add_file("kvdb.lua", include_str!("../builtins/net/kvdb.lua"))
    .into();
  
}