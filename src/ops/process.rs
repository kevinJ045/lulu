use mlua::prelude::*;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Stdio};
use std::sync::{Arc, Mutex};

pub fn split_command(s: &str) -> Vec<String> {
  let mut parts = Vec::new();
  let mut cur = String::new();
  let mut chars = s.chars().peekable();
  let mut in_single = false;
  let mut in_double = false;

  while let Some(ch) = chars.next() {
    match ch {
      '\\' => {
        if let Some(next) = chars.next() {
          cur.push(next);
        }
      }
      '\'' if !in_double => {
        in_single = !in_single;
      }
      '"' if !in_single => {
        in_double = !in_double;
      }
      c if c.is_whitespace() && !in_single && !in_double => {
        if !cur.is_empty() {
          parts.push(cur.clone());
          cur.clear();
        }
      }
      c => cur.push(c),
    }
  }

  if !cur.is_empty() {
    parts.push(cur);
  }

  parts
}

pub fn register_exec(lua: &Lua) -> mlua::Result<()> {
  lua.globals().set(
    "exec",
    lua.create_function(|lua, (command, inherit): (String, Option<bool>)| {
      let parts = split_command(&command);
      if parts.is_empty() {
        return Err(mlua::Error::external("empty command"));
      }

      let program = &parts[0];
      let args = &parts[1..];

      let inherit = inherit.unwrap_or(false);

      if inherit {
        let status = std::process::Command::new(program)
          .args(args)
          .stdin(std::process::Stdio::inherit())
          .stdout(std::process::Stdio::inherit())
          .stderr(std::process::Stdio::inherit())
          .status()
          .map_err(mlua::Error::external)?;

        let result = lua.create_table()?;
        result.set("status", status.code().unwrap_or(-1))?;
        result.set("success", status.success())?;

        Ok(mlua::Value::Table(result))
      } else {
        let output = std::process::Command::new(program)
          .args(args)
          .output()
          .map_err(mlua::Error::external)?;

        let result = lua.create_table()?;
        result.set(
          "stdout",
          String::from_utf8_lossy(&output.stdout).to_string(),
        )?;
        result.set(
          "stderr",
          String::from_utf8_lossy(&output.stderr).to_string(),
        )?;
        result.set("status", output.status.code().unwrap_or(-1))?;
        result.set("success", output.status.success())?;

        Ok(mlua::Value::Table(result))
      }
    })?,
  )?;

  lua.globals().set(
    "spawn",
    lua.create_function(|_, command: String| spawn_process_with_buffer(&command))?,
  )?;

  Ok(())
}

pub struct ProcessHandle {
  stdin: Arc<Mutex<Option<ChildStdin>>>,
  lines: Arc<Mutex<Vec<String>>>,
  child: Arc<Mutex<Child>>,
}

impl mlua::UserData for ProcessHandle {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_method_mut("write", |_, this, input: String| {
      if let Some(stdin) = &mut *this.stdin.lock().unwrap() {
        stdin
          .write_all(input.as_bytes())
          .map_err(mlua::Error::external)?;
        stdin.flush().map_err(mlua::Error::external)?;
      }
      Ok(())
    });

    methods.add_method("read", |lua, this, ()| {
      let mut lines = this.lines.lock().unwrap();
      if lines.is_empty() {
        return Ok(mlua::Value::Nil);
      }
      let line = lines.remove(0);
      Ok(mlua::Value::String(lua.create_string(&line)?))
    });

    methods.add_method_mut("wait", |lua, this, ()| {
      let mut child = this.child.lock().unwrap();
      let status = child.wait().map_err(mlua::Error::external)?;
      let result = lua.create_table()?;
      result.set("status", status.code().unwrap_or(-1))?;
      result.set("success", status.success())?;
      Ok(result)
    });

    methods.add_method("wait_nonblocking", |lua, this, ()| {
      let mut child = this.child.lock().unwrap();
      match child.try_wait() {
        Ok(Some(status)) => {
          let result = lua.create_table()?;
          result.set("status", status.code().unwrap_or(-1))?;
          result.set("success", status.success())?;
          Ok(mlua::Value::Table(result))
        }
        Ok(Option::None) => Ok(mlua::Value::Nil),
        Err(e) => Err(mlua::Error::external(e)),
      }
    });

    methods.add_method_mut("close_stdin", |_, this, ()| {
      *this.stdin.lock().unwrap() = None;
      Ok(())
    });
  }
}

pub fn spawn_process_with_buffer(command: &str) -> mlua::Result<ProcessHandle> {
  let mut parts = split_command(command);
  let program = parts.remove(0);

  let mut child = std::process::Command::new(program)
    .args(parts)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::inherit())
    .spawn()
    .map_err(mlua::Error::external)?;

  let stdout = child.stdout.take().unwrap();
  let lines = Arc::new(Mutex::new(Vec::new()));
  let lines_clone = Arc::clone(&lines);

  std::thread::spawn(move || {
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
      if let Ok(line) = line {
        {
          let mut buffer = lines_clone.lock().unwrap();
          buffer.push(line.clone());
        }
      }
    }
  });

  Ok(ProcessHandle {
    stdin: Arc::new(Mutex::new(child.stdin.take())),
    lines,
    child: Arc::new(Mutex::new(child)),
  })
}
