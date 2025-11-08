use clap::{Arg, ArgAction, Command};
use mlua::prelude::*;
use std::sync::{Arc, Mutex};

use crate::ops::std::create_std_module;

#[derive(Clone)]
pub struct LuaClapCommand {
  pub command: Arc<Mutex<Command>>,
}

impl LuaClapCommand {
  pub fn new(name: String, version: Option<String>, about: Option<String>) -> Self {
    let name = leak_static(name);
    let mut cmd = Command::new(name);
    if let Some(v) = version {
      let v = leak_static(v);
      cmd = cmd.version(v);
    }
    if let Some(a) = about {
      cmd = cmd.about(a);
    }
    Self {
      command: Arc::new(Mutex::new(cmd)),
    }
  }
}

#[derive(Clone)]
pub struct LuaClapSubcommand {
  pub command: Arc<Mutex<Command>>,
}

impl LuaClapSubcommand {
  pub fn new(name: String, version: Option<String>, about: Option<String>) -> Self {
    let name = leak_static(name);
    let mut cmd = Command::new(name);
    if let Some(v) = version {
      let v = leak_static(v);
      cmd = cmd.version(v);
    }
    if let Some(a) = about {
      cmd = cmd.about(a);
    }
    Self {
      command: Arc::new(Mutex::new(cmd)),
    }
  }
}

fn leak_static(s: String) -> &'static str {
  Box::leak(s.into_boxed_str())
}

impl mlua::UserData for LuaClapSubcommand {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    // chainable arg on subcommand
    methods.add_method(
      "arg",
      |_, this, (name, opts): (String, Option<mlua::Table>)| {
        let mut cmd = this.command.lock().unwrap();
        let name_static = leak_static(name);
        let mut arg = Arg::new(name_static);

        if let Some(table) = opts {
          if let Ok(short) = table.get::<String>("short") {
            let s = short.trim_start_matches('-');
            if let Some(ch) = s.chars().next() {
              arg = arg.short(ch);
            }
          }
          if let Ok(long) = table.get::<String>("long") {
            let long_static: &'static str =
              Box::leak(long.trim_start_matches('-').to_string().into_boxed_str());
            arg = arg.long(long_static);
          }
          if let Ok(help) = table.get::<String>("help") {
            arg = arg.help(help);
          }
          if let Ok(default) = table.get::<String>("default") {
            let default = leak_static(default);
            arg = arg.default_value(default);
          }
          if let Ok(required) = table.get::<bool>("required") {
            arg = arg.required(required);
          }
          if let Ok(num_args) = table.get::<usize>("num_args") {
            arg = arg
              .num_args(1..num_args)
              .value_parser(clap::value_parser!(String));
          }
          if let Ok(trailing) = table.get::<bool>("trailing") {
            if trailing {
              arg = arg
                .trailing_var_arg(true)
                .num_args(1..)
                .value_parser(clap::value_parser!(String));
            }
          }
        }

        *cmd = cmd.clone().arg(arg);
        Ok(this.clone())
      },
    );

    // chainable flag on subcommand
    methods.add_method(
      "flag",
      |_, this, (name, opts): (String, Option<mlua::Table>)| {
        let mut cmd = this.command.lock().unwrap();
        let name_static = leak_static(name);
        let mut arg = Arg::new(name_static).action(ArgAction::SetTrue);

        if let Some(table) = opts {
          if let Ok(short) = table.get::<String>("short") {
            let s = short.trim_start_matches('-');
            if let Some(ch) = s.chars().next() {
              arg = arg.short(ch);
            }
          }
          if let Ok(long) = table.get::<String>("long") {
            let long_static: &'static str =
              Box::leak(long.trim_start_matches('-').to_string().into_boxed_str());
            arg = arg.long(long_static);
          }
          if let Ok(help) = table.get::<String>("help") {
            arg = arg.help(help);
          }
        }

        *cmd = cmd.clone().arg(arg);
        Ok(this.clone())
      },
    );
  }
}

impl mlua::UserData for LuaClapCommand {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    // arg on command
    methods.add_method(
      "arg",
      |_, this, (name, opts): (String, Option<mlua::Table>)| {
        let mut cmd = this.command.lock().unwrap();
        let name_static = leak_static(name);
        let mut arg = Arg::new(name_static);

        if let Some(table) = opts {
          if let Ok(short) = table.get::<String>("short") {
            let s = short.trim_start_matches('-');
            if let Some(ch) = s.chars().next() {
              arg = arg.short(ch);
            }
          }
          if let Ok(long) = table.get::<String>("long") {
            let long_static: &'static str =
              Box::leak(long.trim_start_matches('-').to_string().into_boxed_str());
            arg = arg.long(long_static);
          }
          if let Ok(help) = table.get::<String>("help") {
            arg = arg.help(help);
          }
          if let Ok(default) = table.get::<String>("default") {
            let default = leak_static(default);
            arg = arg.default_value(default);
          }
          if let Ok(required) = table.get::<bool>("required") {
            arg = arg.required(required);
          }
          if let Ok(num_args) = table.get::<usize>("num_args") {
            arg = arg
              .num_args(1..num_args)
              .value_parser(clap::value_parser!(String));
          }
          if let Ok(trailing) = table.get::<bool>("trailing") {
            if trailing {
              arg = arg
                .trailing_var_arg(true)
                .num_args(1..)
                .value_parser(clap::value_parser!(String));
            }
          }
        }

        *cmd = cmd.clone().arg(arg);
        Ok(this.clone())
      },
    );

    // flag on command
    methods.add_method(
      "flag",
      |_, this, (name, opts): (String, Option<mlua::Table>)| {
        let mut cmd = this.command.lock().unwrap();
        let name_static = leak_static(name);
        let mut arg = Arg::new(name_static).action(ArgAction::SetTrue);

        if let Some(table) = opts {
          if let Ok(short) = table.get::<String>("short") {
            let s = short.trim_start_matches('-');
            if let Some(ch) = s.chars().next() {
              arg = arg.short(ch);
            }
          }
          if let Ok(long) = table.get::<String>("long") {
            let long_static: &'static str =
              Box::leak(long.trim_start_matches('-').to_string().into_boxed_str());
            arg = arg.long(long_static);
          }
          if let Ok(help) = table.get::<String>("help") {
            arg = arg.help(help);
          }
        }

        *cmd = cmd.clone().arg(arg);
        Ok(this.clone())
      },
    );

    // attach a Subcommand userdata to this Command
    methods.add_method("subcommand", |_, this, sub_ud: mlua::AnyUserData| {
      // ensure sub_ud is a Subcommand, not a Command
      let sub = sub_ud.borrow::<LuaClapSubcommand>()?;
      let sub_cmd = sub.command.lock().unwrap().clone();

      let mut cmd = this.command.lock().unwrap();
      *cmd = cmd.clone().subcommand(sub_cmd);
      Ok(this.clone())
    });

    // parse -> returns table with nested subcommand tables keyed by subcommand name
    methods.add_method(
      "parse",
      |lua, this, (mut arg_vec, name): (Vec<String>, Option<String>)| {
        let cmd = this.command.lock().unwrap().clone();
        arg_vec.insert(0, name.unwrap_or(format!("lulu_command")));

        let matches = cmd
          .try_get_matches_from(arg_vec)
          .map_err(LuaError::external)?;

        fn extract<'a>(lua: &'a Lua, m: &clap::ArgMatches) -> mlua::Result<mlua::Table> {
          let tbl = lua.create_table()?;

          for id in m.ids() {
            let key = id.as_str();
            if key.ends_with("*") {
              if let Ok(Some(iter)) = m.try_get_many::<String>(key) {
                let arr = lua.create_table()?;
                for (i, val) in iter.enumerate() {
                  arr.set(i + 1, val.clone())?;
                }
                tbl.set(key.trim_end_matches("*"), arr)?;
              }
              continue;
            }
            if let Ok(Some(v)) = m.try_get_one::<String>(key) {
              tbl.set(key, v.clone())?;
            } else if let Ok(Some(iter)) = m.try_get_many::<String>(key) {
              let arr = lua.create_table()?;
              for (i, val) in iter.enumerate() {
                arr.set(i + 1, val.clone())?;
              }
              tbl.set(key, arr)?;
            } else if m.get_flag(key) {
              tbl.set(key, true)?;
            } else {
              tbl.set(key, mlua::Value::Nil)?;
            }
          }

          if let Some((sub_name, sub_m)) = m.subcommand() {
            let sub_tbl = extract(lua, sub_m)?;
            tbl.set(sub_name.to_string(), sub_tbl)?;
          }

          Ok(tbl)
        }

        extract(lua, &matches)
      },
    );
  }
}

pub fn into_module() {
  create_std_module("clap")
    .add_function("Command", |_, opts: mlua::Table| {
      let name: String = opts.get("name")?;
      let version: Option<String> = opts.get("version").ok();
      let about: Option<String> = opts.get("about").ok();
      Ok(LuaClapCommand::new(name, version, about))
    })
    .add_function("Subcommand", |_, opts: mlua::Table| {
      let name: String = opts.get("name")?;
      let version: Option<String> = opts.get("version").ok();
      let about: Option<String> = opts.get("about").ok();
      Ok(LuaClapSubcommand::new(name, version, about))
    })
    .on_register(|_, clap_mod| Ok(clap_mod))
    .into();
}
