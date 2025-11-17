// use crate::ops::std::create_std_module;
use mlua::{Error as LuaError, Lua, Result, UserData, UserDataMethods, Value as LuaValue};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug, Clone)]
pub enum LuluArcValue {
  String(String),
  Int32(i32),
  Int64(i64),
  Float64(f64),
  Multi(Box<[LuluArcValue]>),
  Map(HashMap<String, LuluArcValue>),
}

#[derive(Debug, Clone)]
pub enum LuluInner {
  Plain(Arc<LuluArcValue>),
  Mutex(Arc<Mutex<LuluArcValue>>),
  RwLock(Arc<RwLock<LuluArcValue>>),
}

#[derive(Clone)]
pub struct LuluArc {
  inner: LuluInner,
}

impl LuluArc {
  pub fn new_plain(v: LuluArcValue) -> Self {
    Self {
      inner: LuluInner::Plain(Arc::new(v)),
    }
  }

  pub fn new_mutex(v: LuluArcValue) -> Self {
    Self {
      inner: LuluInner::Mutex(Arc::new(Mutex::new(v))),
    }
  }

  pub fn new_rwlock(v: LuluArcValue) -> Self {
    Self {
      inner: LuluInner::RwLock(Arc::new(RwLock::new(v))),
    }
  }

  fn read<'lua>(&self) -> Result<LuluArcValue> {
    match &self.inner {
      LuluInner::Plain(a) => Ok((**a).clone()),
      LuluInner::Mutex(m) => {
        let g = m.lock().map_err(|_| LuaError::external("poisoned mutex"))?;
        Ok(g.clone())
      }
      LuluInner::RwLock(rw) => {
        let g = rw
          .read()
          .map_err(|_| LuaError::external("poisoned rwlock"))?;
        Ok(g.clone())
      }
    }
  }

  fn type_inner<'lua>(&self) -> Result<String> {
    Ok(get_val_type(self.read()?))
  }

  fn write(&self, new_val: LuluArcValue) -> Result<()> {
    match &self.inner {
      LuluInner::Plain(_) => Err(LuaError::external("value is immutable")),
      LuluInner::Mutex(m) => {
        let mut g = m.lock().map_err(|_| LuaError::external("poisoned mutex"))?;
        *g = new_val;
        Ok(())
      }
      LuluInner::RwLock(rw) => {
        let mut g = rw
          .write()
          .map_err(|_| LuaError::external("poisoned rwlock"))?;
        *g = new_val;
        Ok(())
      }
    }
  }
}

fn convert_table(t: &mlua::Table) -> mlua::Result<LuluArcValue> {
  let maxn = t.len()?;

  let mut is_array = true;
  for pair in t.clone().pairs::<LuaValue, LuaValue>() {
    let (key, _) = pair?;
    match key {
      LuaValue::Integer(i) if i >= 1 && i <= maxn => {}
      _ => {
        is_array = false;
        break;
      }
    }
  }

  if is_array {
    let mut vec = Vec::with_capacity(maxn as usize);
    for i in 1..=maxn {
      let entry = t.get::<LuaValue>(i)?;
      vec.push(lua_to_lulu(&entry)?);
    }
    return Ok(LuluArcValue::Multi(vec.into_boxed_slice()));
  }

  let mut map = HashMap::new();

  for pair in t.clone().pairs::<LuaValue, LuaValue>() {
    let (key, value) = pair?;

    let key_str = match key {
      LuaValue::String(s) => s.to_str()?.to_string(),
      LuaValue::Integer(i) => i.to_string(),
      LuaValue::Number(n) => n.to_string(),
      _ => {
        return Err(mlua::Error::RuntimeError(
          "Unsupported table key type".into(),
        ));
      }
    };

    map.insert(key_str, lua_to_lulu(&value)?);
  }

  Ok(LuluArcValue::Map(map))
}

fn get_val_type(v: LuluArcValue) -> String {
  match v {
    LuluArcValue::String(_) => "string".to_string(),
    LuluArcValue::Int32(_) => "i32".to_string(),
    LuluArcValue::Int64(_) => "i64".to_string(),
    LuluArcValue::Float64(_) => "float".to_string(),
    LuluArcValue::Multi(_) => "multi".to_string(),
    LuluArcValue::Map(_) => "map".to_string(),
  }
}

pub fn lulu_to_lua(lua: &Lua, v: LuluArcValue) -> Result<LuaValue> {
  match v {
    LuluArcValue::String(s) => Ok(LuaValue::String(lua.create_string(&s)?)),
    LuluArcValue::Int32(i) => Ok(LuaValue::Integer(i as i64)),
    LuluArcValue::Int64(i) => Ok(LuaValue::Integer(i)),
    LuluArcValue::Float64(f) => Ok(LuaValue::Number(f)),
    LuluArcValue::Multi(slice) => {
      let table = lua.create_table()?;
      for (i, item) in slice.into_vec().into_iter().enumerate() {
        table.set(i + 1, lulu_to_lua(lua, item)?)?;
      }
      Ok(LuaValue::Table(table))
    }
    LuluArcValue::Map(map) => {
      let tbl = lua.create_table()?;
      for (k, val) in map.iter() {
        tbl.set(k.as_str(), lulu_to_lua(lua, val.clone())?)?;
      }
      Ok(LuaValue::Table(tbl))
    }
  }
}

pub fn lua_to_lulu(v: &LuaValue) -> Result<LuluArcValue> {
  match v {
    LuaValue::Nil => Err(LuaError::external("nil not supported")),
    LuaValue::Boolean(b) => Ok(LuluArcValue::Int32(if *b { 1 } else { 0 })),
    LuaValue::Integer(i) => Ok(LuluArcValue::Int64(*i)),
    LuaValue::Number(n) => Ok(LuluArcValue::Float64(*n)),
    LuaValue::String(s) => Ok(LuluArcValue::String(s.to_str()?.to_owned())),
    LuaValue::Table(t) => convert_table(t),
    LuaValue::UserData(ud) => {
      if let Ok(lulu_arc) = ud.borrow::<LuluArc>() {
        Ok(lulu_arc.read()?.clone())
      } else {
        Err(LuaError::external(
          "unsupported userdata type for conversion",
        ))
      }
    }
    _ => Err(LuaError::external(
      "unsupported Lua value type for conversion",
    )),
  }
}

impl UserData for LuluArc {
  fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
    methods.add_method("get", |lua, this, ()| {
      let val = this.read()?;
      lulu_to_lua(lua, val)
    });

    methods.add_method("set", |_lua, this, input: LuaValue| {
      let new = lua_to_lulu(&input)?;
      this.write(new)?;
      Ok(())
    });

    methods.add_meta_method(mlua::MetaMethod::Call, |lua, this, other: LuaValue| {
      match other {
        LuaValue::Nil => {},
        _ => {
          let new = lua_to_lulu(&other)?;
          this.write(new)?;
        } 
      }
      let val = this.read()?;
      lulu_to_lua(lua, val)
    });

    methods.add_meta_method(mlua::MetaMethod::Add, |_lua, this, other: LuaValue| {
      let a = this.read()?;
      let b = lua_to_lulu(&other)?;
      match (a, b) {
        (LuluArcValue::Int32(x), LuluArcValue::Int32(y)) => {
          Ok(mlua::Value::Number((x + y).into()))
        }
        (LuluArcValue::Int64(x), LuluArcValue::Int64(y)) => {
          Ok(mlua::Value::Number((x as f64 + y as f64).into()))
        }
        (LuluArcValue::Float64(x), LuluArcValue::Float64(y)) => {
          Ok(mlua::Value::Number((x + y).into()))
        }
        _ => Err(mlua::Error::external("unsupported types for addition")),
      }
    });

    methods.add_meta_method(mlua::MetaMethod::Sub, |_lua, this, other: LuaValue| {
      let a = this.read()?;
      let b = lua_to_lulu(&other)?;
      match (a, b) {
        (LuluArcValue::Int32(x), LuluArcValue::Int32(y)) => {
          Ok(mlua::Value::Number((x - y).into()))
        }
        (LuluArcValue::Int64(x), LuluArcValue::Int64(y)) => {
          Ok(mlua::Value::Number((x as f64 - y as f64).into()))
        }
        (LuluArcValue::Float64(x), LuluArcValue::Float64(y)) => {
          Ok(mlua::Value::Number((x - y).into()))
        }
        _ => Err(mlua::Error::external("unsupported types for subtraction")),
      }
    });

    methods.add_meta_method(mlua::MetaMethod::Div, |_lua, this, other: LuaValue| {
      let a = this.read()?;
      let b = lua_to_lulu(&other)?;
      match (a, b) {
        (LuluArcValue::Int32(x), LuluArcValue::Int32(y)) => {
          Ok(mlua::Value::Number((x / y).into()))
        }
        (LuluArcValue::Int64(x), LuluArcValue::Int64(y)) => {
          Ok(mlua::Value::Number((x as f64 / y as f64).into()))
        }
        (LuluArcValue::Float64(x), LuluArcValue::Float64(y)) => {
          Ok(mlua::Value::Number((x / y).into()))
        }
        _ => Err(mlua::Error::external("unsupported types for division")),
      }
    });

    methods.add_meta_method(mlua::MetaMethod::Mul, |_lua, this, other: LuaValue| {
      let a = this.read()?;
      let b = lua_to_lulu(&other)?;
      match (a, b) {
        (LuluArcValue::Int32(x), LuluArcValue::Int32(y)) => {
          Ok(mlua::Value::Number((x * y).into()))
        }
        (LuluArcValue::Int64(x), LuluArcValue::Int64(y)) => {
          Ok(mlua::Value::Number((x as f64 * y as f64).into()))
        }
        (LuluArcValue::Float64(x), LuluArcValue::Float64(y)) => {
          Ok(mlua::Value::Number((x * y).into()))
        }
        _ => Err(mlua::Error::external("unsupported types for multiplication")),
      }
    });

    methods.add_method("type", |_lua, this, ()| Ok(this.type_inner()?));

    methods.add_method("kind", |_lua, this, ()| {
      let s = match &this.inner {
        LuluInner::Plain(_) => "plain",
        LuluInner::Mutex(_) => "mutex",
        LuluInner::RwLock(_) => "rwlock",
      };
      Ok(s.to_string())
    });

    methods.add_method("tostring", |_lua, this, ()| {
      let v = this.read()?;
      Ok(format!("{:?}", v))
    });

    methods.add_method("clone_handle", |_lua, this, ()| Ok(this.clone()));
  }
}

// pub fn into_module() {
//   create_std_module("rust")
//     .add_function("arc", |_, v: LuaValue| {
//       Ok(LuluArc::new_plain(lua_to_lulu(&v)?))
//     })
//     .add_function("mutex", |_, v: LuaValue| {
//       Ok(LuluArc::new_mutex(lua_to_lulu(&v)?))
//     })
//     .add_function("rwlock", |_, v: LuaValue| {
//       Ok(LuluArc::new_rwlock(lua_to_lulu(&v)?))
//     })
//     .on_register(|_, rust_mod| Ok(rust_mod))
//     .into();
// }
