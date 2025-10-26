use crate::bundle::{bundle_lulu_or_exec, load_lulib, run_bundle, set_exec_path};
use crate::cli::{CacheCommand, Cli, Commands};
use crate::conf::load_lulu_conf;
use crate::lulu::Lulu;
use crate::ops::{register_consts, TOK_ASYNC_HANDLES};
use crate::package_manager::PackageManager;
use clap::Parser;
use mlua::Result;
use mlua::prelude::LuaError;
use std::collections::HashMap;
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
  crate::ops::init_std_modules();
  if let Some(mods) = bundle::load_embedded_scripts() {
    handle_error!(
      run_bundle(
        mods,
        &mut Lulu::new(
          Some(std::env::args().skip(1).collect()),
          Some(std::env::current_exe()?.parent().unwrap().to_path_buf())
        )
      )
      .await
    );
  } else {
    let cli = Cli::parse();

    match &cli.command {
      Commands::Run { file, args, build } => {
        handle_error!(if *build {
          let lua = mlua::Lua::new();
          let conf = load_lulu_conf(&lua, file.join("lulu.conf.lua"))?;
          let name = conf.manifest.unwrap().get::<String>("name")?;
          std::process::Command::new(std::env::current_exe()?)
            .arg("build")
            .arg(file.clone())
            .status()?;
          let runpath = if file.join(format!(".lib/{name}.lulib")).exists() {
            file.join(format!(".lib/{name}.lulib"))
          } else {
            file.join(".lib").join(name)
          };

          if runpath.ends_with(".lulib") {
            let mods = load_lulib(&runpath)?;
            run_bundle(mods, &mut Lulu::new(Some(args.clone()), Some(runpath))).await?;
          } else {
            std::process::Command::new(runpath).args(args).status()?;
          }
          Ok(())
        } else if file.extension().and_then(|s| s.to_str()) == Some("lulib") {
          let mods = load_lulib(file)?;
          run_bundle(
            mods,
            &mut Lulu::new(
              Some(args.clone()),
              Some(file.parent().unwrap().to_path_buf()),
            ),
          )
          .await
        } else if file.is_dir() {
          let mut lulu = Lulu::new(Some(args.clone()), Some(file.to_path_buf()));
          let filepath = if file.join("init.lua").exists() {
            file.join("init.lua")
          } else {
            file.join("main.lua")
          };
          lulu.exec_entry_mod_path(filepath.clone()).await
        } else {
          let mut lulu = Lulu::new(
            Some(args.clone()),
            Some(file.parent().unwrap().to_path_buf()),
          );
          lulu.exec_entry_mod_path(file.clone()).await
        });
      }
      Commands::Compile { file } => {
        let path = std::fs::canonicalize(file)?;
        let mut lulu = Lulu::new(None, Some(path.clone().parent().unwrap().to_path_buf()));
        println!("{}", lulu.compile(path.clone())?);
      }
      Commands::Test { file, test, args } => {
        let mut lulu = Lulu::new(
          Some(args.clone()),
          Some(file.parent().unwrap().to_path_buf()),
        );
        lulu.compiler.env = "test".to_string();
        lulu.compiler.current_test = test.clone();
        handle_error!(lulu.exec_entry_mod_path(file.clone()).await);
      }
      Commands::Bundle { file, output } => {
        let mut lulu = Lulu::new(None, None);
        bundle_lulu_or_exec(&mut lulu, file.clone(), output.clone())?;
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
      }
      Commands::Build { path } => {
        let conf_path = path.join("lulu.conf.lua");

        if !conf_path.exists() {
          eprintln!("Path has no lulu.conf.lua");
          return Ok(());
        }

        let conf_string = std::fs::read_to_string(conf_path.clone())?;
        let lua = mlua::Lua::new();

        register_consts(&lua)?;

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

          lua.globals().set(
            "download_file_async",
            lua.create_async_function(async move |_, url: String| {
              PackageManager::new()
                .map_err(|e| {
                  eprintln!("Failed to initialize package manager: {}", e);
                  mlua::Error::external(e)
                })?
                .clone()
                .download_file(&url)
                .await
                .map_err(|e| {
                  eprintln!("Failed to download file: {}", e);
                  mlua::Error::external(e)
                })
            })?,
          )?;

          lua.globals().set(
            "download_file",
            lua
              .load(mlua::chunk! {
                local f = coroutine.create(function(...)
                  download_file_async(...)
                  return false
                end)
                local done = true
                while done do
                  done = coroutine.resume(f, ...)
                end
              })
              .into_function()?,
          )?;

          lua.globals().set(
            "set_stub",
            lua.create_function(move |_, path: String| {
              set_exec_path(path);
              Ok(())
            })?,
          )?;

          let stubs_fn =
            lua.create_async_function(async move |_, stubs: HashMap<String, String>| {
              let current_os = std::env::consts::OS;

              let url = if let Some(url) = stubs.get(current_os) {
                Ok(url)
              } else if let Some(url) =
                stubs.get(&format!("{}-{}", current_os, std::env::consts::ARCH))
              {
                Ok(url)
              } else {
                Err(mlua::Error::external(format!(
                  "No stub found for OS: {}",
                  current_os
                )))
              }?;

              let path = if url.starts_with("http") {
                let cache_path = PackageManager::new()
                  .map_err(|e| {
                    eprintln!("Failed to initialize package manager: {}", e);
                    mlua::Error::external(e)
                  })?
                  .download_file(url)
                  .await
                  .map_err(|e| {
                    eprintln!("Failed to download file: {}", e);
                    mlua::Error::external(e)
                  })?;

                let file_name = url
                  .split('/')
                  .last()
                  .ok_or_else(|| mlua::Error::external("Invalid URL: missing file name"))?;

                let file_path = cache_path.join(file_name);

                #[cfg(unix)]
                {
                  use std::os::unix::fs::PermissionsExt;
                  let mut perms = std::fs::metadata(file_path.clone())?.permissions();
                  perms.set_mode(perms.mode() | 0o111);
                  std::fs::set_permissions(file_path.clone(), perms)?;
                }

                file_path
              } else {
                Path::new(url).to_path_buf()
              };

              set_exec_path(path);
              Ok(())
            })?;

          lua.globals().set("stubs_async", stubs_fn)?;

          lua.globals().set(
            "stubs",
            lua
              .load(mlua::chunk! {
                local f = coroutine.create(function(...)
                  stubs_async(...)
                  return false
                end)
                local done = true
                while done do
                  done = coroutine.resume(f, ...)
                end
              })
              .into_function()?,
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

          handle_error!(build_fn_lua.call::<()>(()));
        }
      }
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

      }
      Commands::New {
        name,
        git,
        lib,
        ignore,
      } => {
        project::new(name.clone(), *git, *ignore, *lib);
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
      }
    }
  }

  let handles = std::mem::take(&mut *TOK_ASYNC_HANDLES.lock().unwrap());
  for handle in handles {
    let _ = handle.await;
  }
  Ok(())
}
