use mlua::prelude::LuaError;
use mlua::{Lua};
use rand::rngs::StdRng;
use rand::{Rng, RngCore, SeedableRng};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::{Arc, RwLock};
use crate::ops::sys;

pub fn create_std(lua: &Lua) -> mlua::Result<()> {
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

lazy_static::lazy_static! {
  pub static ref STD_MODULES: RwLock<HashMap<String, Arc<STDModule>>> =
      RwLock::new(HashMap::new());
}

pub fn create_std_module(name: &str) -> STDModule {
  STDModule::new(name)
}

pub fn get_std_module(name: &str) -> Option<Arc<STDModule>> {
  STD_MODULES.read().unwrap().get(name).cloned()
}

pub fn init_std_modules() {
  sys::into_module();

  crate::lulibs::clap::into_module();

  crate::lulibs::serialize::into_module();

  crate::lulibs::kv::into_module();

  crate::lulibs::archive::into_module();

  crate::lulibs::net::into_module();

  crate::lulibs::threads::into_module();

  crate::lulibs::console::into_module();
}
