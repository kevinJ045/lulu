use crate::bundle::{bundle_lulu_or_exec, load_lulib, make_bin, run_bundle, write_bundle};
use crate::cli::{CacheCommand, Cli, Commands};
use crate::conf::conf_to_string;
use crate::lulu::{LuLib, Lulu, LuluModSource};
use crate::package_manager::PackageManager;
use crate::util::lua_to_bytecode;
use clap::Parser;
use mlua::Result;
use mlua::prelude::LuaError;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};

mod bundle;
mod cli;
pub mod compiler;
pub mod conf;
mod lml;
pub mod lulu;
mod ops;
mod package_manager;
mod project;
mod resolver;
mod util;

macro_rules! into_exec_command {
  ($lua:expr, $env:expr, (), $cmd:expr $(, $arg:expr)*) => {{
    let env_ref = $env.clone();
    $lua.create_function(move |_, ()| {
      let mut cmd = std::process::Command::new(std::env::current_exe()?);
      cmd.arg($cmd);
      $(
        cmd.arg($arg);
      )*
      let map = env_ref.lock().unwrap();
      for (k, v) in map.iter() {
        cmd.env(k, v);
      }
      cmd.status()?;
      Ok(())
    })?
  }};

  ($lua:expr, $env:expr, ($($arg_name:ident : $arg_type:ty),+), $cmd:expr $(, $arg:expr)*) => {{
    let env_ref = $env.clone();
    #[allow(unused_parens)]
    $lua.create_function(move |_, ($($arg_name),+): ($($arg_type),+)| {
      let mut cmd = std::process::Command::new(std::env::current_exe()?);
      cmd.arg($cmd);
      $(
        cmd.arg($arg);
      )*
      let map = env_ref.lock().unwrap();
      for (k, v) in map.iter() {
        cmd.env(k, v);
      }
      cmd.status()?;
      Ok(())
    })?
  }};
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
  if let Some(mods) = bundle::load_embedded_scripts() {
    do_error!(run_bundle(
      mods,
      std::env::args().collect(),
      Some(std::env::current_exe()?.parent().unwrap().to_path_buf())
    ));
    Ok(())
  } else {
    let cli = Cli::parse();

    match &cli.command {
      Commands::Run { file, args } => {
        do_error!(
          if file.extension().and_then(|s| s.to_str()) == Some("lulib") {
            let mods = load_lulib(file)?;
            run_bundle(
              mods,
              args.clone(),
              Some(file.parent().unwrap().to_path_buf()),
            )
          } else {
            let mut lulu = Lulu::new(
              Some(args.clone()),
              Some(file.parent().unwrap().to_path_buf()),
            );
            lulu.exec_entry_mod_path(file.clone())
          }
        );
        Ok(())
      }
      Commands::Test { file, test, args } => {
        let mut lulu = Lulu::new(
          Some(args.clone()),
          Some(file.parent().unwrap().to_path_buf()),
        );
        lulu.compiler.env = "test".to_string();
        lulu.compiler.current_test = test.clone();
        do_error!(lulu.exec_entry_mod_path(file.clone()));
        Ok(())
      }
      Commands::Bundle { file, output } => {
        let mut lulu = Lulu::new(None, None);
        bundle_lulu_or_exec(&mut lulu, file.clone(), output.clone())
      }
      Commands::Resolve { item } => {
        let pkg_manager = PackageManager::new().map_err(|e| mlua::Error::external(e))?;

        async {
          if item.starts_with("http") || item.starts_with("github:") {
            let path = std::path::PathBuf::from(".");

            match pkg_manager.install_package(item.as_str(), &path).await {
              Ok(_) => {}
              Err(e) => eprintln!("Failed to resolve dependency \"{}\": {}", item, e),
            };
          } else {
            let path = std::path::PathBuf::from(item);
            let conf_path = path.join("lulu.conf.lua");

            if let Ok(conf_string) = std::fs::read_to_string(conf_path.clone()) {
              let lua = mlua::Lua::new();
              let parent_path = path.clone();

              if let Ok(Some(dependencies)) =
                conf::load_lulu_conf_dependiencies(&lua, conf_string.clone())
              {
                let packages_to_install = dependencies;
                match pkg_manager
                  .install_packages(&packages_to_install, &parent_path)
                  .await
                {
                  Ok(_) => {}
                  Err(e) => eprintln!("Failed to resolve dependencies: {}", e),
                }
              } else {
                eprintln!("No dependencies found in {}", conf_path.display());
              }
            } else {
              eprintln!("Could not read configuration file: {}", conf_path.display());
            }
          }
        }
        .await;

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

          let env = Arc::new(Mutex::new(HashMap::<String, String>::new()));
          let lulu_arc = Arc::new(Mutex::new(Lulu::new(None, None)));

          let env_ref = env.clone();
          lua.globals().set(
            "set_env",
            lua.create_function(move |_, (name, value): (String, String)| {
              let mut map = env_ref.lock().unwrap();
              map.insert(name, value);
              Ok(())
            })?,
          )?;

          let bundle_path = path.clone();
          let bundle = into_exec_command!(lua, env, (file: String, output: String), "bundle", bundle_path.clone().join(file), bundle_path.join(output));

          // let bname = name.clone();
          // let bundle_main_path = path.clone();
          // let bundle_main_path = path.clone();
          // lua.globals().set("bundle", lua.create_function(move |_, file: String| {
          //     bundle_lulu_or_exec(&mut lulu, bundle_main_path.join(file).to_path_buf(), Path::new(&format!(".lib/{}.lulib", name.clone())).to_path_buf())
          //   })?)?;
          // let bundle_main_entry = into_exec_command!(lua, env, (file: String), "bundle", bundle_main_path.clone().join(file), bundle_main_path.join(format!(".lib/{}.lulib", bname.clone())));

          // let name = name.clone();
          // let bundle_main_path = path.clone();
          // let bundle_main_entry_exec = into_exec_command!(lua, env, (file: String), "bundle", bundle_main_path.clone().join(file), bundle_main_path.join(format!(".lib/{}", name.clone())));

          let ipath = path.clone();
          let larc = lulu_arc.clone();
          lua.globals().set(
            "include_bytes",
            lua.create_function(move |_, (name, file): (String, String)| {
              let file_path = ipath.join(file);
              let bytes = std::fs::read(file_path)?;
              let mut lulu = larc.lock().unwrap();
              lulu.add_mod_from_bytecode(format!("bytes://{}", name), bytes, None);
              Ok(())
            })?,
          )?;
          
          let larc = lulu_arc.clone();
          lua.globals().set(
            "set_cfg_env",
            lua.create_function(move |_, (key, value): (String, String)| {
              let mut lulu = larc.lock().unwrap();
              lulu.compiler.defs.insert(key, value);
              Ok(())
            })?,
          )?;

          let bname = name.clone();
          let bundle_main_path = path.clone();
          let larc = lulu_arc.clone();
          lua.globals().set(
            "bundle_main",
            lua.create_function(move |_, (file, lulib): (String, Option<bool>)| {
              let is_lulib = if let Some(lulib) = lulib {
                lulib
              } else {
                false
              };
              let mut lulu = larc.lock().unwrap();
              bundle_lulu_or_exec(
                &mut lulu,
                bundle_main_path.join(file).to_path_buf(),
                Path::new(&format!(
                  ".lib/{}{}",
                  bname.clone(),
                  if is_lulib { ".lulib" } else { "" }
                ))
                .to_path_buf(),
              )
            })?,
          )?;

          let build_path = path.clone();
          let build =
            into_exec_command!(lua, env, (file: String), "build", build_path.clone().join(file));

          let resolve_path = path.clone();
          let resolve_dependencies =
            into_exec_command!(lua, env, (), "resolve", resolve_path.clone());

          let exists_path = path.clone();
          let exists_func =
            lua.create_function(move |_, name: String| Ok(exists_path.join(name).exists()))?;

          lua
            .globals()
            .set("resolve_dependencies", resolve_dependencies)?;

          lua.globals().set("bundle", bundle)?;
          // lua.globals().set("bundle_main", bundle_main_entry)?;
          // lua
          //   .globals()
          //   .set("bundle_main_exec", bundle_main_entry_exec)?;
          lua.globals().set("build", build)?;
          lua.globals().set("exists", exists_func)?;

          do_error!(build_fn_lua.call::<()>(()));
        }
        Ok(())
      }
      Commands::Compile { file: _, output: _ } => Ok(()),
      Commands::Update { packages, project } => {
        let pkg_manager = PackageManager::new().map_err(|e| {
          eprintln!("Failed to initialize package manager: {}", e);
          mlua::Error::external(e)
        })?;

        async {
          for package in packages {
            if let Err(e) = pkg_manager.clear_package_cache(package) {
              eprintln!("Warning: Failed to clear cache for {}: {}", package, e);
            }
          }

          match pkg_manager.install_packages(packages, project).await {
            Ok(_) => {}
            Err(e) => {
              eprintln!("Package update failed: {}", e);
            }
          }
        }
        .await;

        Ok(())
      }
      Commands::New {
        name,
        git,
        lib,
        ignore,
      } => {
        project::new(name.clone(), *git, *ignore, *lib);
        Ok(())
      }
      Commands::Cache { cache_command } => {
        let pkg_manager = PackageManager::new().map_err(|e| {
          eprintln!("Failed to initialize package manager: {}", e);
          mlua::Error::external(e)
        })?;

        match cache_command {
          CacheCommand::Clear => match pkg_manager.clear_cache() {
            Ok(()) => println!("Package cache cleared successfully"),
            Err(e) => eprintln!("Failed to clear cache: {}", e),
          },
          CacheCommand::List => match pkg_manager.list_cached_packages() {
            Ok(packages) => {
              if packages.is_empty() {
                println!("No cached packages found");
              } else {
                println!("Cached packages:");
                for package in packages {
                  println!("  - {}", package);
                }
              }
            }
            Err(e) => eprintln!("Failed to list cached packages: {}", e),
          },
          CacheCommand::Remove { package_url } => {
            match pkg_manager.clear_package_cache(package_url) {
              Ok(()) => println!("Package cache cleared for: {}", package_url),
              Err(e) => eprintln!("Failed to clear package cache: {}", e),
            }
          }
        }

        Ok(())
      }
    }
  }
}
