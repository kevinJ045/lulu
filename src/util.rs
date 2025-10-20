use mlua::Lua;
use std::path::Path;

#[macro_export]
macro_rules! handle_error {
  ($case:expr) => {
    match $case {
      Err(e) => {
        match e {
          LuaError::SyntaxError {
            message,
            incomplete_input: _,
          } => {
            eprintln!("SyntaxError: {}", message);
          }
          LuaError::RuntimeError(msg) => {
            eprintln!("RuntimeError: {}", msg);
          }
          LuaError::MemoryError(msg) => {
            eprintln!("MemoryError: {}", msg);
          }
          _ => {
            eprintln!("{}", e);
          }
        }
        std::process::exit(1);
      }
      _ => {}
    };
  };
}

pub fn lua_to_bytecode(lua: &Lua, code: &str) -> mlua::Result<Vec<u8>> {
  let func: mlua::Function = lua.load(code).into_function()?;

  let dump: mlua::String = lua.load("return string.dump(...)").call(func)?;

  Ok(dump.as_bytes().to_vec())
}

pub fn normalize_name(cpath: &str) -> String {
  let path = Path::new(cpath);
  let mut parts: Vec<String> = path
    .components()
    .filter_map(|c| c.as_os_str().to_str().map(String::from))
    .collect();

  if let Some(last) = parts.last_mut() {
    if let Some(stem) = Path::new(last).file_stem() {
      *last = stem.to_string_lossy().to_string();
    }
  }

  parts.join("-")
}