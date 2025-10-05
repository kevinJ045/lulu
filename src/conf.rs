use mlua::{Lua, UserData};
// use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LuluConf {
  pub manifest: Option<mlua::Table>,
  pub mods: Option<HashMap<String, String>>,
  pub include: Option<Vec<String>>,
  pub macros: Option<String>,
}

impl UserData for LuluConf {
  fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
    fields.add_field_method_get("mods", |_, this| Ok(this.mods.clone()));
    fields.add_field_method_get("manifest", |_, this| Ok(this.manifest.clone()));
  }
}

#[derive(Debug)]
pub enum FetchField {
  Code,
  Lulib {
    url: String,
    include: Option<HashMap<String, Vec<String>>>,
  },
}

fn table_to_lua_string(table: &mlua::Table) -> mlua::Result<String> {
  let mut parts = Vec::new();
  for pair in table.pairs::<mlua::Value, mlua::Value>() {
    let (k, v) = pair?;
    let key = match k {
      mlua::Value::String(s) => s.to_str()?.to_string(),
      mlua::Value::Integer(_) => "".to_string(),
      _ => "<unsupported>".to_string(),
    };
    let val = match v {
      mlua::Value::String(s) => format!(r#""{}""#, s.to_str()?),
      mlua::Value::Integer(i) => i.to_string(),
      mlua::Value::Boolean(b) => b.to_string(),
      mlua::Value::Table(b) => table_to_lua_string(&b)?,
      _ => "<unsupported>".to_string(),
    };
    parts.push(if !key.is_empty() {
      format!("{} = {}", key, val)
    } else {
      val
    });
  }
  Ok(format!("{{ {} }}", parts.join(", ")))
}

pub fn conf_to_string(conf: &LuluConf) -> mlua::Result<String> {
  let mut out = String::from(
    "return {
",
  );

  if let Some(manifest) = &conf.manifest {
    out.push_str("  manifest = ");
    out.push_str(&table_to_lua_string(manifest)?);
    out.push_str(
      ",
",
    );
  }

  if let Some(mods) = &conf.mods {
    out.push_str("  mods = { ");
    for (k, v) in mods {
      out.push_str(&format!(r#"{} = "{}","#, k, v));
    }
    out.push_str(
      " },
",
    );
  }
  
  if let Some(macros) = &conf.macros {
    out.push_str(format!("  macros = [[{}]]\n", macros).as_str());
  }

  out.push('}');
  Ok(out)
}

pub fn load_lulu_fetch_field(lua: &Lua, code: String) -> mlua::Result<Option<FetchField>> {
  lua.load(&code).set_name("lulu.conf.lua").exec()?;

  let globals = lua.globals();
  let fetch_val: Option<mlua::Value> = globals.get("fetch")?;

  if let Some(fetch) = fetch_val {
    match fetch {
      mlua::Value::Table(table) => {
        let lulib: Option<String> = table.get("lulib").ok();
        let include: Option<HashMap<String, Vec<String>>> =
          if let Ok(include_table) = table.get::<mlua::Table>("include") {
            let mut map = HashMap::new();
            for pair in include_table.pairs::<String, mlua::Value>() {
              let (key, val) = pair?;
              if let mlua::Value::Table(inner) = val {
                let mut vec = Vec::new();
                for v in inner.sequence_values::<String>() {
                  vec.push(v?);
                }
                map.insert(key, vec);
              }
            }
            Some(map)
          } else {
            None
          };

        return Ok(Some(FetchField::Lulib {
          url: lulib.unwrap(),
          include: include,
        }));
      }
      mlua::Value::String(s) if s.to_str()? == "code" => {
        return Ok(Some(FetchField::Code));
      }
      _ => return Ok(None),
    }
  }

  Ok(None)
}

pub fn load_lulu_conf_dependiencies(lua: &Lua, code: String) -> mlua::Result<Option<Vec<String>>> {
  lua.load(&code).set_name("lulu.conf.lua").exec()?;

  let globals = lua.globals();
  let dependencies: Option<Vec<String>> = globals.get("dependencies")?;

  Ok(dependencies)
}

pub fn load_lulu_conf_builder(lua: &Lua, code: String) -> mlua::Result<Option<mlua::Function>> {
  lua.load(&code).set_name("lulu.conf.lua").exec()?;

  let globals = lua.globals();
  let build: Option<mlua::Function> = globals.get("build")?;

  Ok(build)
}

pub enum CodeType {
  Bytes(Vec<u8>),
  Code(String),
}

pub fn load_lulu_conf_code(lua: &Lua, code: CodeType) -> mlua::Result<LuluConf> {
  let globals = match code {
    CodeType::Bytes(code) => lua
      .load(&code)
      .set_name("lulu.conf.lua")
      .eval::<mlua::Table>()?,
    CodeType::Code(code) => {
      lua.load(&code).set_name("lulu.conf.lua").exec()?;
      lua.globals()
    }
  };
  let manifest: Option<mlua::Table> = globals.get("manifest").ok();
  let mods = globals
    .get::<HashMap<String, String>>("mods")
    .map(Some)
    .unwrap_or(None);
  let include = globals
    .get::<Vec<String>>("include")
    .map(Some)
    .unwrap_or(None);
  let macros = globals
    .get::<String>("macros")
    .map(Some)
    .unwrap_or(None);
  
  globals.set("manifest", mlua::Value::Nil)?;
  globals.set("mods", mlua::Value::Nil)?;
  globals.set("macros", mlua::Value::Nil)?;
  globals.set("include", mlua::Value::Nil)?;
  
  Ok(LuluConf {
    manifest,
    mods,
    include,
    macros
  })
}

pub fn load_lulu_conf(lua: &Lua, path: PathBuf) -> mlua::Result<LuluConf> {
  let code = std::fs::read_to_string(path)?;
  load_lulu_conf_code(lua, CodeType::Code(
    crate::compiler::wrap_macros(code.as_str())
  ))
}

pub fn load_lulu_conf_from_bytecode(lua: &Lua, bytecode: Vec<u8>) -> mlua::Result<LuluConf> {
  load_lulu_conf_code(lua, CodeType::Bytes(bytecode))
}

pub fn find_lulu_conf(start: PathBuf) -> Option<PathBuf> {
  let mut dir = start;

  loop {
    let candidate = dir.join("lulu.conf.lua");
    if candidate.exists() {
      return Some(candidate);
    }

    match dir.parent() {
      Some(parent) => dir = parent.to_path_buf(),
      _ => break,
    }
  }

  None
}
