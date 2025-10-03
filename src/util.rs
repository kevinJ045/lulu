use mlua::Lua;

#[macro_export]
macro_rules! do_error {
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
      },
      _ => {}
    };
  };
}

pub fn lua_to_bytecode(lua: &Lua, code: &str) -> mlua::Result<Vec<u8>> {
  let func: mlua::Function = lua.load(code).into_function()?;

  let dump: mlua::String = lua.load("return string.dump(...)").call(func)?;

  Ok(dump.as_bytes().to_vec())
}
