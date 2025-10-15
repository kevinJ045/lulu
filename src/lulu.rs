use crate::compiler::Compiler;
use crate::conf::{LuluConf, find_lulu_conf, load_lulu_conf};
use crate::ops;
use mlua::{Lua, chunk};
use std::path::PathBuf;

const STD_FILE: &str = include_str!("./builtins/std.lua");

#[derive(Debug, Clone)]
pub struct LuLib {
  pub bytes: Vec<u8>,
  pub conf: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub enum LuluModSource {
  Code(String),
  Bytecode(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct LuluMod {
  pub name: String,
  pub source: LuluModSource,
  pub conf: Option<LuluConf>,
}

#[derive(Debug, Clone)]
pub struct Lulu {
  pub mods: Vec<LuluMod>,
  pub lua: Lua,
  pub args: Vec<String>,
  pub current: Option<PathBuf>,
  pub compiler: Compiler,
  std: String
}

impl Lulu {
  pub fn new(args: Option<Vec<String>>, current: Option<PathBuf>) -> Lulu {
    let mods = Vec::new();
    let lua = unsafe { Lua::unsafe_new() };

    let mut compiler = Compiler::new(None);

    let std = compiler.compile(STD_FILE, None, None);
    // println!("{}", std);
    
    Lulu {
      mods,
      lua,
      args: args.unwrap_or_default(),
      current,
      compiler,
      std
    }
  }

  pub fn preload_mods(&mut self) -> mlua::Result<()> {

    let items = self.compiler.importmap.clone();

    for (name, (path_to_import, path_from, conf)) in &items {
      let path_from = if let Some(p) = path_from {
        p.clone()
      } else {
        "".to_string()
      };
      if path_from.is_empty() {
        continue;
      }
      let parent = std::path::Path::new(&path_from).parent().unwrap();
      let file_path = parent.join(path_to_import);

      if !file_path.exists() {
        panic!(
          "File \"{:?}\" does not exist. Imported from: \"{}\"",
          file_path, path_from
        );
      }

      if name.starts_with("bytes://") {
        let bytecode = std::fs::read(file_path)?;
        self.add_mod_from_bytecode(name.to_string(), bytecode, None);
      } else {
        self.add_mod_from_file(name.to_string(), file_path, conf.clone())?;
      }
    }

    ops::register_ops(&self.lua, self)?;

    self
      .lua
      .load(
        r#"
        local embedded = __get_mods__()
        package.preload = package.preload or {}
        require_native = require
        for key, name in pairs(embedded) do
          package.preload[name] = function()
            return exec_mod(name)
          end
        end
        "#
      )
      .exec()?;

    self
      .lua
      .load(
        self.std.clone()
      )
      .set_name("std")
      .exec()?;


    Ok(())
  }

  pub fn add_mod(&mut self, lmod: LuluMod) {
    self.mods.push(lmod);
  }

  pub fn add_mod_from_code(&mut self, name: String, code: String, conf: Option<LuluConf>) {
    self.add_mod(LuluMod {
      name,
      source: LuluModSource::Code(self.compiler.clone().compile(code.as_str(), None, None)),
      conf,
    });
  }

  pub fn add_mod_from_bytecode(&mut self, name: String, bytecode: Vec<u8>, conf: Option<LuluConf>) {
    self.add_mod(LuluMod {
      name,
      source: LuluModSource::Bytecode(bytecode),
      conf,
    });
  }

  pub fn add_mod_from_file(
    &mut self,
    name: String,
    path: PathBuf,
    conf: Option<LuluConf>,
  ) -> mlua::Result<()> {
    let raw = std::fs::read(&path)?;

    let source = match std::str::from_utf8(&raw) {
      Ok(code) => LuluModSource::Code(self.compiler.compile(
        code,
        Some(std::fs::canonicalize(path)?.to_string_lossy().to_string()),
        conf.clone(),
      )),
      Err(_) => LuluModSource::Bytecode(raw),
    };

    let modname = if let Some(n) = self.compiler.last_mod.clone() {
      self.compiler.last_mod = None;
      n
    } else {
      name.clone()
    };

    self.add_mod(LuluMod { name: modname, source, conf });
    Ok(())
  }

  pub fn exec_mod(&self, name: &str) -> mlua::Result<mlua::Value> {
    let lmod = self
      .mods
      .iter()
      .find(|m| m.name == name)
      .ok_or_else(|| mlua::Error::RuntimeError(format!("Module {} not found", name)))?;

    let chunk = match &lmod.source {
      LuluModSource::Code(code) => self.lua.load(code),
      LuluModSource::Bytecode(bytes) => self.lua.load(&bytes[..]),
    }
    .set_name(name);

    let env = if let Some(env) = chunk.environment() {
      env.clone()
    } else {
      let env = self.lua.create_table()?;

      let mt = self.lua.create_table()?;
      mt.set("__index", self.lua.globals())?;
      env.set_metatable(Some(mt))?;

      env
    };

    let lmod_table = self.lua.create_table()?;

    if let Some(conf) = lmod.conf.clone() {
      let p = self.lua.create_userdata::<LuluConf>(conf)?;

      lmod_table.set("conf", p)?;
    }
    lmod_table.set("name", name)?;

    env.set("mod", lmod_table)?;

    let req_chunk = self.lua.load(chunk! {
      local name = ({...})[1]
      if mod.conf then
        local modname = mod.conf.manifest.name .. "/" .. name;
        if package.preload[modname] then
          return require_native(modname)
        end
      end
      return require_native(name)
    });

    let req = req_chunk.set_environment(env.clone()).into_function()?;

    env.set("require", req)?;

    if let Some(current) = self.current.clone() {
      env.set("current_path", std::fs::canonicalize(current)?)?;
    } else {
      env.set("current_path", mlua::Value::Nil)?;
    }
    let current = self.current.clone();
    let lookup_dylib = self.lua.create_function(move |_, name: String| {
      let path = std::fs::canonicalize(current.clone().unwrap_or(PathBuf::from(".")))?;
      let lib_folder = path.join(".lib/dylib").join(name.clone());
      let dylib_here = path.join("dylib").join(name.clone());

      if lib_folder.exists() {
        Ok(lib_folder)
      } else if dylib_here.exists() {
        Ok(dylib_here)
      } else {
        Ok(name.into())
      }
    })?;

    env.set("lookup_dylib", lookup_dylib)?;


    let using = self.lua.load(chunk! {
      local args = { ... }
      args[1](getfenv(1))
    }).set_environment(env.clone()).into_function()?;

    env.set("using", using)?;

    let chunk = chunk.set_environment(env);

    chunk.eval()
  }

  pub fn entry_mod_path(&mut self, path: PathBuf) -> mlua::Result<String> {
    let mut mainname = "main".to_string();
    let conf = if let Some(root_path) = find_lulu_conf(path.clone()) {
      let c = load_lulu_conf(&self.lua, root_path.clone())?;
      let prefix = if let Some(manifest) = c.clone().manifest {
        if let Ok(n) = manifest.get::<mlua::Value>("name") {
          match n.to_string() {
            Ok(n) => format!("{}/", n),
            Err(_) => "".to_string(),
          }
        } else {
          "".to_string()
        }
      } else {
        "".to_string()
      };

      if let Some(mods) = c.mods.clone() {
        for (name, modpath) in mods {
          let mod_path = root_path.parent().unwrap().join(modpath);
          if mod_path == path {
            mainname = format!("{}{}", prefix, name.clone());
            continue;
          }
          self.add_mod_from_file(
            format!("{}{}", prefix, name.clone()),
            mod_path,
            Some(c.clone()),
          )?;
        }
      }

      if let Some(macros) = c.macros.clone() {
        self.compiler.compile(&macros, None, None);
      }

      if let Some(include) = c.include.clone() {
        for libpath in include {
          let lib_path = root_path
            .parent()
            .unwrap()
            .join(if libpath.starts_with("@") {
              format!(".lib/lulib/{}.lulib", libpath[1..].to_string())
            } else {
              libpath
            });
          let mods = crate::bundle::load_lulib(&lib_path)?;
          crate::bundle::reg_bundle_nods(self, mods)?;
        }
      }
      Some(c)
    } else {
      mainname = path.to_string_lossy().to_string();
      None
    };

    self.add_mod_from_file(mainname.clone(), path.clone(), conf)?;

    self.preload_mods()?;

    Ok(mainname)
  }

  pub fn find_mod(&mut self, name: &str) -> mlua::Result<String> {
    let lmod = self
      .mods
      .iter()
      .find(|m| m.name.ends_with(name))
      .ok_or_else(|| mlua::Error::RuntimeError(format!("No main was found")))?;

    Ok(lmod.name.clone())
  }

  pub fn exec_entry_mod_path(&mut self, path: PathBuf) -> mlua::Result<()> {
    let mainname = self.entry_mod_path(path)?;
    self.exec_mod(mainname.as_str())?;

    Ok(())
  }

  pub fn compile(&mut self, path: PathBuf) -> mlua::Result<String> {
    let mainname = self.entry_mod_path(path)?;

    let lmod = self
      .mods
      .iter()
      .find(|m| m.name == mainname)
      .ok_or_else(|| mlua::Error::RuntimeError(format!("Such module was not found")))?;

    match lmod.source.clone() {
      LuluModSource::Code(code) => Ok(code),
     _ => Err(mlua::Error::DeserializeError("Module string was not found".to_string()))
    }
  }
}
