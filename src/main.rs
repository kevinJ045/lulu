use crate::bundle::{load_lulib, make_bin, run_bundle, write_bundle};
use crate::cli::{Cli, Commands};
use crate::conf::conf_to_string;
use crate::lulu::{LuLib, Lulu, LuluModSource};
use crate::util::lua_to_bytecode;
use clap::Parser;
use mlua::Result;
use mlua::prelude::LuaError;
use std::collections::HashMap;
use std::fs::File;

mod bundle;
mod cli;
pub mod compiler;
pub mod conf;
pub mod lulu;
mod ops;
mod resolver;
mod util;

macro_rules! into_exec_command {
  ($lua:expr, (), $cmd:expr $(, $arg:expr)*) => {{
    $lua.create_function(move |_, ()| {
      let mut cmd = std::process::Command::new(std::env::current_exe()?);
      cmd.arg($cmd);
      $(
        cmd.arg($arg);
      )*
      cmd.status()?;
      Ok(())
    })?
  }};

  ($lua:expr, ($($arg_name:ident : $arg_type:ty),+), $cmd:expr $(, $arg:expr)*) => {{
    #[allow(unused_parens)]
    $lua.create_function(move |_, ($($arg_name),+): ($($arg_type),+)| {
      let mut cmd = std::process::Command::new(std::env::current_exe()?);
      cmd.arg($cmd);
      $(
        cmd.arg($arg);
      )*
      cmd.status()?;
      Ok(())
    })?
  }};
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
  if let Some(mods) = bundle::load_embedded_scripts() {
    println!("running");
    do_error!(run_bundle(mods, std::env::args().collect()));
    Ok(())
  } else {
    let cli = Cli::parse();

    match &cli.command {
      Commands::Run { file, args } => {
        do_error!(
          if file.extension().and_then(|s| s.to_str()) == Some("lulib") {
            let mods = load_lulib(file)?;
            run_bundle(mods, args.clone())
          } else {
            let mut lulu = Lulu::new(Some(args.clone()));
            lulu.exec_entry_mod_path(file.clone())
          }
        );
        Ok(())
      }
      Commands::Bundle { file, output } => {
        let mut lulu = Lulu::new(None);

        lulu.entry_mod_path(file.clone())?;

        let mut combined_bytes = HashMap::<String, LuLib>::new();

        for lmod in &lulu.mods {
          let conf = if let Some(conf) = lmod.conf.clone() {
            let conft = conf_to_string(&conf)?;
            Some(lua_to_bytecode(&lulu.lua, conft.as_str())?)
          } else {
            None
          };
          match &lmod.source {
            LuluModSource::Code(code) => {
              combined_bytes.insert(
                lmod.name.clone(),
                LuLib {
                  bytes: lua_to_bytecode(&lulu.lua, compiler::compile(code).as_str())?,
                  conf,
                },
              );
            }
            LuluModSource::Bytecode(bytes) => {
              combined_bytes.insert(
                lmod.name.clone(),
                LuLib {
                  bytes: bytes.clone(),
                  conf,
                },
              );
            }
          }
        }

        if output.extension().and_then(|s| s.to_str()) == Some("lulib") {
          let mut f = File::create(output)?;
          write_bundle(&mut f, combined_bytes)?;
        } else {
          make_bin(output, combined_bytes)?;
        }
        Ok(())
      }
      Commands::Resolve { item } => {
        if item.starts_with("http") || item.starts_with("github:") {
          let path = std::path::PathBuf::from(".");

          match resolver::fetch_dependency(item.as_str(), &path).await {
            Err(e) => eprintln!("Failed to resolve dependency \"{}\":\n{}", item, e),
            _ => {}
          };
        } else {
          let path = std::path::PathBuf::from(item);
          let conf_path = path.join("lulu.conf.lua");
          let conf_string = std::fs::read_to_string(conf_path.clone())?;
          let lua = mlua::Lua::new();
          let parent_path = path.clone();
          let dependencies = conf::load_lulu_conf_dependiencies(&lua, conf_string.clone())?;
          if let Some(dependencies) = dependencies {
            for dependency in dependencies {
              match resolver::fetch_dependency(dependency.as_str(), &parent_path.clone()).await {
                Err(e) => eprintln!("Failed to resolve dependency \"{}\":\n{}", dependency, e),
                _ => {}
              };
            }
          }
        }
        Ok(())
      }
      Commands::Build { path } => {
        let conf_path = path.join("lulu.conf.lua");

        if !conf_path.exists() {
          eprintln!("Path has no lulu.conf.lua");
          return Ok(());
        }

        let conf_string = std::fs::read_to_string(conf_path.clone())?;
        let lua = mlua::Lua::new();

        if let Some(build_fn_lua) = conf::load_lulu_conf_builder(&lua, conf_string.clone())? {
          let main = conf::load_lulu_conf_code(&lua, conf::CodeType::Code(conf_string))?;
          let name = main
            .manifest
            .unwrap_or(lua.create_table()?)
            .get::<String>("name")?;

          let bundle_path = path.clone();
          let bundle = into_exec_command!(lua, (file: String, output: String), "bundle", bundle_path.clone().join(file), bundle_path.join(output));

          let bname = name.clone();
          let bundle_main_path = path.clone();
          let bundle_main_entry = into_exec_command!(lua, (file: String), "bundle", bundle_main_path.clone().join(file), bundle_main_path.join(format!(".lib/{}.lulib", bname.clone())));

          let name = name.clone();
          let bundle_main_path = path.clone();
          let bundle_main_entry_exec = into_exec_command!(lua, (file: String), "bundle", bundle_main_path.clone().join(file), bundle_main_path.join(format!(".lib/{}", name.clone())));

          let build_path = path.clone();
          let build =
            into_exec_command!(lua, (file: String), "build", build_path.clone().join(file));

          let resolve_path = path.clone();
          let resolve_dependencies = into_exec_command!(lua, (), "resolve", resolve_path.clone());

          let exists_path = path.clone();
          let exists_func =
            lua.create_function(move |_, name: String| Ok(exists_path.join(name).exists()))?;

          lua
            .globals()
            .set("resolve_dependencies", resolve_dependencies)?;

          lua.globals().set("bundle", bundle)?;
          lua.globals().set("bundle_main", bundle_main_entry)?;
          lua
            .globals()
            .set("bundle_main_exec", bundle_main_entry_exec)?;
          lua.globals().set("build", build)?;
          lua.globals().set("exists", exists_func)?;

          do_error!(build_fn_lua.call::<()>(()));
        }
        Ok(())
      }
      Commands::Compile { file: _, output: _ } => Ok(()),
    }
  }
}
