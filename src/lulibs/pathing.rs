use mlua::prelude::*;
use std::path::PathBuf;

use crate::ops::std::create_std_module;

#[derive(Clone)]
pub struct LuluPath {
  pub base: PathBuf,
}

impl LuaUserData for LuluPath {
  fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
    methods.add_method_mut("join", |_, this, sub: String| {
      let p = this.base.join(sub);
      Ok(LuluPath { base: p })
    });

    methods.add_method_mut("append", |_, this, sub: String| {
      this.base = this.base.join(sub);
      Ok(this.clone())
    });

    methods.add_method("to_string", |_, this, _: ()| {
      Ok(this.base.to_string_lossy().to_string())
    });

    methods.add_meta_method_mut(LuaMetaMethod::ToString, |_, this, _: ()| {
      Ok(this.base.to_string_lossy().to_string())
    });

    methods.add_method("exists", |_, this, _: ()| Ok(this.base.exists()));

    methods.add_method("is_file", |_, this, _: ()| Ok(this.base.is_file()));

    methods.add_method("is_dir", |_, this, _: ()| Ok(this.base.is_dir()));

    methods.add_method("filename", |lua, this, _: ()| {
      Ok(
        this
          .base
          .file_name()
          .and_then(|s| s.to_str())
          .map(|s| lua.create_string(s))
          .transpose()?,
      )
    });

    methods.add_method("extension", |lua, this, _: ()| {
      Ok(
        this
          .base
          .extension()
          .and_then(|s| s.to_str())
          .map(|s| lua.create_string(s))
          .transpose()?,
      )
    });

    methods.add_method("stem", |lua, this, _: ()| {
      Ok(
        this
          .base
          .file_stem()
          .and_then(|s| s.to_str())
          .map(|s| lua.create_string(s))
          .transpose()?,
      )
    });

    methods.add_method("parent", |lua, this, _: ()| {
      if let Some(parent) = this.base.parent() {
        let new_path = LuluPath {
          base: parent.to_path_buf(),
        };
        Ok(LuaValue::UserData(lua.create_userdata(new_path)?))
      } else {
        Ok(LuaValue::Nil)
      }
    });

    methods.add_method("components", |lua, this, _: ()| {
      let tbl = lua.create_table()?;
      for (i, comp) in this.base.components().enumerate() {
        let s = comp.as_os_str().to_string_lossy().to_string();
        tbl.set(i + 1, s)?;
      }
      Ok(tbl)
    });

    methods.add_method("list", |lua, this, _: ()| {
      let tbl = lua.create_table()?;

      if this.base.is_dir() {
        for entry in std::fs::read_dir(&this.base).map_err(mlua::Error::external)? {
          let entry = entry.map_err(mlua::Error::external)?;
          let path = LuluPath { base: entry.path() };
          tbl.push(lua.create_userdata(path)?)?;
        }
      }

      Ok(tbl)
    });

    methods.add_method("ensure_dir", |_, this, _: ()| {
      std::fs::create_dir_all(&this.base).map_err(mlua::Error::external)?;
      Ok(this.clone())
    });

    methods.add_method("ensure_file", |_, this, content: Option<String>| {
      if let Some(parent) = this.base.parent() {
        std::fs::create_dir_all(parent).map_err(mlua::Error::external)?;
      }

      if !this.base.exists() {
        if let Some(c) = content {
          std::fs::write(&this.base, c).map_err(mlua::Error::external)?;
        } else {
          std::fs::write(&this.base, "").map_err(mlua::Error::external)?;
        }
      }

      Ok(this.clone())
    });

    methods.add_method("ensure", |_lua, this, content: Option<String>| {
      if this.base.ends_with(std::path::MAIN_SEPARATOR.to_string()) {
        if !this.base.exists() {
          std::fs::create_dir_all(&this.base).map_err(mlua::Error::external)?;
        }
      } else {
        if let Some(parent) = this.base.parent() {
          std::fs::create_dir_all(parent).map_err(mlua::Error::external)?;
        }
        if let Some(c) = content {
          if !this.base.exists() {
            std::fs::write(&this.base, c).map_err(mlua::Error::external)?;
          }
        } else if !this.base.exists() {
          std::fs::create_dir_all(&this.base).map_err(mlua::Error::external)?;
        }
      }

      Ok(this.clone())
    });
  }
}

pub fn into_module() {
  create_std_module("pathing")
    .add_function("new", |lua, path: String| {
      let ud = lua.create_userdata(LuluPath {
        base: PathBuf::from(path),
      })?;
      Ok(ud)
    })
    .add_function("root", |lua, _: ()| {
      lua.create_userdata(LuluPath {
        base: std::env::current_dir().unwrap(),
      })
    })
    .add_function("temp", |lua, _: ()| {
      lua.create_userdata(LuluPath {
        base: std::env::temp_dir(),
      })
    })
    .add_function("appdata", |lua, _: ()| {
      let base = dirs::data_dir().unwrap_or(std::env::current_dir().unwrap());
      lua.create_userdata(LuluPath { base })
    })
    .add_function("cache", |lua, _: ()| {
      let base = dirs::cache_dir().unwrap_or(std::env::temp_dir());
      lua.create_userdata(LuluPath { base })
    })
    .add_function("program_files", |lua, _: ()| {
      let base = dirs::executable_dir().unwrap_or(std::env::current_dir().unwrap());
      lua.create_userdata(LuluPath { base })
    })
    .into();
}
