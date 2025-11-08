use crate::lulu::{Lulu, LuluModSource};
use crate::package_manager::PackageManager;
use mlua::prelude::LuaError;
use mlua::{Lua, LuaSerdeExt};
use rand::rngs::StdRng;
use rand::{Rng, RngCore, SeedableRng};
use regex::Regex;
use reqwest::Method;
use reqwest::{
  Client,
  header::{HeaderMap, HeaderName, HeaderValue},
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::{BufRead, BufReader};
use std::io::{Read, Write};
use std::process::{Child, ChildStdin, Stdio};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tokio::time;
use zip::write::ExtendedFileOptions;
use zip::{ZipWriter, write::FileOptions};















pub fn register_ops(lua: &Lua, lulu: &Lulu) -> mlua::Result<()> {
  let mods = lulu.mods.clone();

  let gmods = lua.create_function(move |_, ()| {
    let mut embedded_scripts: Vec<String> = Vec::new();

    for lmod in &mods {
      embedded_scripts.push(lmod.name.clone());
    }

    Ok(embedded_scripts)
  })?;

  let execmod = lua.create_function({
    let lulu_rc = lulu.clone();
    move |_, name: String| {
      let lulu = &lulu_rc;
      lulu.exec_mod(&name)
    }
  })?;

  lua.globals().set(
    "request_env_load",
    lua.create_function({
      let lulu_rc = lulu.clone();
      let imported = Arc::new(Mutex::new(Vec::new()));
      move |lua, (env, name): (String, Option<String>)| {
        if let Some(module) = get_std_module(&env) {
          let mut imports = imported.lock().unwrap();
          let name = if let Some(name) = name {
            name.clone()
          } else {
            env.clone()
          };
          if imports.contains(&name) {
            return Ok(mlua::Value::Boolean(true));
          }
          module.register(lua)?;
          imports.push(name);
          if !module.deps.is_empty() {
            let tbl = lua.create_table()?;
            tbl.set("__include", module.deps.clone())?;
            return Ok(mlua::Value::Table(tbl));
          }
          return Ok(mlua::Value::Boolean(true));
        }
        if let Ok(modules) = lua
          .globals()
          .get::<mlua::Table>("package")?
          .get::<HashMap<String, mlua::Value>>("preload")
        {
          if let Some(mn) = if let Some(_) = modules.get(&env) {
            Some(env.clone())
          } else if let Some(_) = modules.get(&format!("{}/init", env)) {
            Some(format!("{}/init", env))
          } else {
            None
          } {
            let lulu = &lulu_rc;
            let module = lulu.exec_mod(&mn)?;

            let tbl = lua.create_table()?;
            tbl.set("__into", name)?;
            tbl.set("__value", module)?;

            return Ok(mlua::Value::Table(tbl));
          }
        }
        Ok(mlua::Value::Boolean(false))
      }
    })?,
  )?;

  let bytes_from_mods = {
    let lulu_rc = lulu.clone();
    lua.create_function(move |_, name: String| {
      let lulu = &lulu_rc;
      if let Some(module) = lulu.mods.iter().find(|m| {
        m.name
          == (if name.starts_with("bytes://") {
            name.clone()
          } else {
            format!("bytes://{}", name)
          })
      }) {
        let bytes = match &module.source {
          LuluModSource::Bytecode(bytes) => bytes.clone(),
          LuluModSource::Code(code) => code.as_bytes().to_vec(),
        };
        Ok(LuluByteArray { bytes })
      } else {
        Err(mlua::Error::RuntimeError(format!(
          "Module '{}' not found",
          name
        )))
      }
    })
  }?;

  let sleep_fn = lua.create_async_function(async move |_, secs: u64| -> mlua::Result<()> {
    time::sleep(time::Duration::from_secs(secs)).await;
    Ok(())
  })?;
  lua.globals().set("sleep", sleep_fn)?;

  lua.globals().set("__get_mods__", gmods)?;
  lua.globals().set("bytes_from", bytes_from_mods)?;
  lua.globals().set("exec_mod", execmod)?;
  lua.globals().set("argv", lulu.args.clone())?;

  let ptr_of = lua.create_function(|lua, value: mlua::Value| {
    let ptr = lua.create_registry_value(value)?;
    Ok(Box::into_raw(Box::new(ptr)) as usize)
  })?;
  lua.globals().set("ptr_of", ptr_of)?;

  let ptr_deref = lua.create_function(|lua, ptr: usize| {
    let value_ptr = ptr as *mut mlua::RegistryKey;
    if value_ptr.is_null() {
      return Ok(mlua::Value::Nil);
    }
    let key = unsafe { &*value_ptr };
    let value = lua.registry_value::<mlua::Value>(&key)?;
    Ok(value)
  })?;
  lua.globals().set("ptr_deref", ptr_deref)?;

  let ptr_set = lua.create_function(|lua, (ptr, new_val): (usize, mlua::Value)| {
    let value_ptr = ptr as *mut mlua::RegistryKey;
    if value_ptr.is_null() {
      return Err(mlua::Error::RuntimeError("Null pointer".to_string()));
    }
    let key = unsafe { &mut *value_ptr };
    lua.replace_registry_value(key, new_val.clone())?;
    Ok(new_val)
  })?;
  lua.globals().set("ptr_set", ptr_set)?;

  lua.globals().set(
    "reads",
    lua.create_function(|_, path: String| Ok(fs::read_to_string(path)?))?,
  )?;

  lua.globals().set(
    "exists",
    lua.create_function(|_, path: String| Ok(std::path::Path::new(&path).exists()))?,
  )?;

  lua.globals().set(
    "mkdir",
    lua.create_function(|_, path: String| {
      fs::create_dir_all(&path)?;
      Ok(())
    })?,
  )?;

  lua.globals().set(
    "cp",
    lua.create_function(|_, (src, dest): (String, String)| {
      fs::copy(&src, &dest)?;
      Ok(())
    })?,
  )?;

  lua.globals().set(
    "rename",
    lua.create_function(|_, (old, new): (String, String)| {
      fs::rename(&old, &new)?;
      Ok(())
    })?,
  )?;

  lua.globals().set(
    "mv",
    lua.create_function(|_, (src, dest): (String, String)| {
      fs::copy(&src, &dest)?;
      fs::remove_file(&src)?;
      Ok(())
    })?,
  )?;

  lua.globals().set(
    "rm",
    lua.create_function(|_, path: String| {
      let p = std::path::Path::new(&path);
      if p.is_dir() {
        fs::remove_dir_all(p)?;
      } else if p.is_file() {
        fs::remove_file(p)?;
      }
      Ok(())
    })?,
  )?;

  lua.globals().set(
    "read",
    lua.create_function(|_, path: String| {
      let mut file = fs::File::open(&path)?;
      let mut buffer = Vec::new();
      file.read_to_end(&mut buffer)?;
      Ok(LuluByteArray { bytes: buffer })
    })?,
  )?;

  register_exec(lua)?;

  lua.globals().set(
    "exit",
    lua.create_function(|_, code: Option<i32>| {
      std::process::exit(code.unwrap_or(0));
      #[allow(unreachable_code)]
      Ok(())
    })?,
  )?;

  lua.globals().set(
    "foreach",
    lua.create_function(|lua, items: Vec<mlua::Value>| {
      Ok(lua.create_function(move |_, func: mlua::Function| {
        let mut mapped: Vec<mlua::Value> = Vec::new();
        for i in items.clone() {
          mapped.push(func.call(i)?)
        }
        Ok(mapped)
      }))
    })?,
  )?;

  lua.globals().set(
    "range",
    lua.create_function(|_, (x, y, z): (usize, usize, Option<usize>)| {
      let nums: Vec<usize> = (x..=y).step_by(z.unwrap_or(1) as usize).collect();
      Ok(nums)
    })?,
  )?;

  let lulu_clone = lulu.clone();

  lua.globals().set(
    "require_cached_async",
    lua.create_async_function(move |_, url: String| {
      let mut lulu_clone = lulu_clone.clone();
      async move {
        let pkg_manager = PackageManager::new().map_err(mlua::Error::external)?;

        let cache_path = pkg_manager.get_package_cache_path(&url);
        if !pkg_manager.is_cached(&url) {
          pkg_manager
            .fetch_package(&url, &cache_path)
            .await
            .map_err(mlua::Error::external)?;
          pkg_manager
            .build_package(&cache_path)
            .await
            .map_err(mlua::Error::external)?;
        }

        let cache_lulib_dir = cache_path.join(".lib");
        if cache_lulib_dir.exists() {
          for entry in std::fs::read_dir(&cache_lulib_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file()
              && entry.path().extension().and_then(|s| s.to_str()) == Some("lulib")
            {
              let mods = crate::bundle::load_lulib(&entry.path())?;
              crate::bundle::reg_bundle_nods(&mut lulu_clone, mods.clone())?;

              let (modname, _) = mods
                .iter()
                .find(|(m, _)| m.ends_with("init"))
                .ok_or_else(|| mlua::Error::RuntimeError(format!("No main was found")))?;

              return Ok(lulu_clone.exec_mod(modname.as_str())?);
            }
          }
        }

        Ok(mlua::Value::Nil)
      }
    })?,
  )?;

  let set_ctor = lua.create_function(|_, ()| {
    Ok(LuluHashSet {
      items: HashSet::new(),
    })
  })?;
  lua.globals().set("HashSet", set_ctor)?;

  let map_ctor = lua.create_function(|_, ()| {
    Ok(LuluHashMap {
      items: HashMap::new(),
    })
  })?;
  lua.globals().set("HashMap", map_ctor)?;
  lua.globals().set(
    "ByteArray",
    lua.create_function(|_, bytes: Vec<u8>| Ok(LuluByteArray { bytes }))?,
  )?;
  lua.globals().set(
    "exec_sandboxed",
    lua.create_function(
      |lua, (code, name, env): (String, Option<String>, Option<mlua::Table>)| {
        let mut chunk = lua.load(code);

        if let Some(name) = name {
          chunk = chunk.set_name(name);
        }

        if let Some(env) = env {
          chunk = chunk.set_environment(env);
        } else {
          chunk = chunk.set_environment(lua.create_table()?);
        }

        chunk.eval::<mlua::Value>()
      },
    )?,
  )?;

  lua.globals().set(
    "setup_downloader",
    lua.create_function(|lua, options: Option<mlua::Table>| {
      let mut pm = PackageManager::new().map_err(|e| {
        eprintln!("Failed to initialize package manager: {}", e);
        mlua::Error::external(e)
      })?;
      if let Some(options) = options {
        if let Ok(format) = options.get::<String>("format") {
          pm.downloader.format = format;
        }

        if let Ok(download_text) = options.get::<String>("download_text") {
          pm.downloader.download_text = download_text;
        }

        if let Ok(progress_bar_size) = options.get::<usize>("progressbar_size") {
          pm.downloader.progress_bar_size = progress_bar_size;
        }

        if let Ok(progress_bar_colors) = options.get::<Vec<u8>>("progressbar_colors") {
          pm.downloader.progress_bar_colors = (
            (
              progress_bar_colors[0],
              progress_bar_colors[1],
              progress_bar_colors[2],
            ),
            (
              progress_bar_colors[3],
              progress_bar_colors[4],
              progress_bar_colors[5],
            ),
          );
        }
      }
      lua
        .globals()
        .set("__lulu_pac_man", lua.create_any_userdata(pm)?)?;
      Ok(())
    })?,
  )?;

  lua.globals().set(
    "download_file",
    lua.create_async_function(async |lua, url: String| {
      let pm = lua.globals().get::<mlua::AnyUserData>("__lulu_pac_man")?;
      let pm = pm.borrow::<PackageManager>()?;
      pm.download_file(&url).await.map_err(|e| {
        eprintln!("Failed to download file: {}", e);
        mlua::Error::external(e)
      })
    })?,
  )?;
  lua.globals().set(
    "download_uncached",
    lua.create_async_function(async |lua, (url, path): (String, String)| {
      let pm = lua.globals().get::<mlua::AnyUserData>("__lulu_pac_man")?;
      let pm = pm.borrow::<PackageManager>()?;
      pm.download_url(&url, &std::path::Path::new(&path))
        .await
        .map_err(|e| {
          eprintln!("Failed to download file: {}", e);
          mlua::Error::external(e)
        })
    })?,
  )?;
  lua.globals().set(
    "require_cached",
    lua
      .load(mlua::chunk! {
        local f = coroutine.create(function(...)
          require_cached_async(...)
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
    "sync_call",
    lua
      .load(mlua::chunk! {
        local args = {...}
        local fn = args[1]
        table.remove(args, 1)
        local f = coroutine.create(function(...)
          fn(...)
          return false
        end)
        local done = true
        while done do
          done = coroutine.resume(f, unpack(args))
        end
      })
      .into_function()?,
  )?;

  create_std(lua)?;

  Ok(())
}

pub fn register_consts(lua: &Lua) -> mlua::Result<()> {
  lua.globals().set("CURRENT_OS", std::env::consts::OS)?;

  lua.globals().set("CURRENT_ARCH", std::env::consts::ARCH)?;

  lua
    .globals()
    .set("CURRENT_FAMILY", std::env::consts::FAMILY)?;

  lua.globals().set("LULU_VER", env!("CARGO_PKG_VERSION"))?;

  Ok(())
}












pub fn create_std_module(name: &str) -> STDModule {
  STDModule::new(name)
}

pub fn get_std_module(name: &str) -> Option<Arc<STDModule>> {
  STD_MODULES.read().unwrap().get(name).cloned()
}

pub fn init_std_modules() {
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

  create_std_module("sys")
    .add_function("battery", |lua, _: ()| {
      use battery::Manager;

      let manager = Manager::new().map_err(mlua::Error::external)?;
      let mut batteries = manager.batteries().map_err(mlua::Error::external)?;
      if let Some(Ok(bat)) = batteries.next() {
        let tbl = lua.create_table()?;

        tbl.set("state", format!("{:?}", bat.state()))?;
        tbl.set("percentage", bat.state_of_charge().value * 100.0)?;
        tbl.set("energy", bat.energy().value)?;
        tbl.set("energy_full", bat.energy_full().value)?;
        tbl.set("energy_full_design", bat.energy_full_design().value)?;
        tbl.set("energy_rate", bat.energy_rate().value)?;
        tbl.set("voltage", bat.voltage().value)?;
        tbl.set("temperature", bat.temperature().map(|t| t.value))?;
        tbl.set("cycle_count", bat.cycle_count())?;
        tbl.set("time_to_full", bat.time_to_full().map(|t| t.value))?;
        tbl.set("time_to_empty", bat.time_to_empty().map(|t| t.value))?;

        Ok(Some(tbl))
      } else {
        Ok(None)
      }
    })
    .on_register(|lua, sys| {
      let get_proc = |lua: &mlua::Lua, (pid, proc): (&sysinfo::Pid, &sysinfo::Process)| {
        let pid_val = pid.as_u32();
        let name = proc.name();
        let exe = proc.exe();
        let cmd = proc.cmd().to_vec();
        let memory = proc.memory();
        let cpu_usage = proc.cpu_usage();

        let process = lua.create_table().unwrap();

        process.set(
          "kill",
          lua.create_function(move |_, ()| {
            let sys = sysinfo::System::new_all();
            if let Some(proc) = sys.process(sysinfo::Pid::from_u32(pid_val)) {
              proc.kill();
            }
            Ok(())
          })?,
        )?;

        process.set(
          "kill_with",
          lua.create_function(move |_, signal: String| {
            let sys = sysinfo::System::new_all();
            if let Some(proc) = sys.process(sysinfo::Pid::from_u32(pid_val)) {
              use sysinfo::Signal::*;
              let sig = match signal.as_str() {
                "Hangup" => Hangup,
                "Interrupt" => Interrupt,
                "Quit" => Quit,
                "Illegal" => Illegal,
                "Trap" => Trap,
                "Abort" => Abort,
                "IOT" => IOT,
                "Bus" => Bus,
                "FloatingPointException" => FloatingPointException,
                "Kill" => Kill,
                "User1" => User1,
                "Segv" => Segv,
                "User2" => User2,
                "Pipe" => Pipe,
                "Alarm" => Alarm,
                "Term" => Term,
                "Child" => Child,
                "Continue" => Continue,
                "Stop" => Stop,
                "TSTP" => TSTP,
                "TTIN" => TTIN,
                "TTOU" => TTOU,
                "Urgent" => Urgent,
                "XCPU" => XCPU,
                "XFSZ" => XFSZ,
                "VirtualAlarm" => VirtualAlarm,
                "Profiling" => Profiling,
                "Winch" => Winch,
                "IO" => IO,
                "Poll" => Poll,
                "Power" => Power,
                "Sys" => Sys,
                _ => Kill,
              };
              proc.kill_with(sig);
            }
            Ok(())
          })?,
        )?;

        process.set(
          "exists",
          lua.create_function(move |_, ()| {
            let sys = sysinfo::System::new_all();
            Ok(sys.process(sysinfo::Pid::from_u32(pid_val)).is_some())
          })?,
        )?;

        process.set("pid", pid_val)?;
        process.set("name", name)?;
        process.set("exe", exe)?;
        process.set("cmd", cmd)?;
        process.set("memory", memory)?;
        process.set("cpu_usage", cpu_usage)?;

        Ok::<mlua::Table, mlua::Error>(process)
      };

      sys.set(
        "cpus",
        lua.create_function(move |lua, ()| {
          Ok(
            sysinfo::System::new_all()
              .cpus()
              .into_iter()
              .map(|c| {
                let cpu = lua.create_table().unwrap();

                cpu.set("name", c.name()).unwrap();
                cpu.set("usage", c.cpu_usage()).unwrap();
                cpu.set("vendor_id", c.vendor_id()).unwrap();
                cpu.set("brand", c.brand()).unwrap();
                cpu.set("frequency", c.frequency()).unwrap();

                cpu
              })
              .collect::<Vec<_>>(),
          )
        })?,
      )?;

      sys.set(
        "processes",
        lua.create_function(move |lua, ()| {
          let procs = lua.create_table().unwrap();
          sysinfo::System::new_all()
            .processes()
            .into_iter()
            .for_each(|(pid, proc)| {
              let process = get_proc(lua, (pid, proc)).unwrap();
              procs.set(pid.as_u32(), process).unwrap();
            });
          Ok(procs)
        })?,
      )?;

      sys.set(
        "process",
        lua.create_function(move |lua, pid: usize| {
          let sys = sysinfo::System::new_all();
          let pid = sysinfo::Pid::from(pid);
          let proc = sys.process(pid).unwrap();
          Ok(get_proc(lua, (&pid, proc))?)
        })?,
      )?;

      sys.set(
        "global_cpu_usage",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().global_cpu_usage()))?,
      )?;

      sys.set(
        "total_memory",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().total_memory()))?,
      )?;
      sys.set(
        "free_memory",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().free_memory()))?,
      )?;
      sys.set(
        "available_memory",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().available_memory()))?,
      )?;
      sys.set(
        "used_memory",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().used_memory()))?,
      )?;
      sys.set(
        "total_swap",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().total_swap()))?,
      )?;
      sys.set(
        "free_swap",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().free_swap()))?,
      )?;
      sys.set(
        "used_swap",
        lua.create_function(move |_, ()| Ok(sysinfo::System::new_all().used_swap()))?,
      )?;

      sys.set(
        "uptime",
        lua.create_function(move |_, ()| Ok(sysinfo::System::uptime()))?,
      )?;
      sys.set(
        "boot_time",
        lua.create_function(move |_, ()| Ok(sysinfo::System::boot_time()))?,
      )?;
      sys.set(
        "load_average",
        lua.create_function(move |_, ()| {
          let lavg = sysinfo::System::load_average();
          Ok(vec![lavg.one, lavg.five, lavg.fifteen])
        })?,
      )?;
      sys.set(
        "name",
        lua.create_function(move |_, ()| Ok(sysinfo::System::name()))?,
      )?;
      sys.set(
        "kernel_version",
        lua.create_function(move |_, ()| Ok(sysinfo::System::kernel_version()))?,
      )?;
      sys.set(
        "os_version",
        lua.create_function(move |_, ()| Ok(sysinfo::System::os_version()))?,
      )?;
      sys.set(
        "long_os_version",
        lua.create_function(move |_, ()| Ok(sysinfo::System::long_os_version()))?,
      )?;
      sys.set(
        "distribution_id",
        lua.create_function(move |_, ()| Ok(sysinfo::System::distribution_id()))?,
      )?;
      sys.set(
        "distribution_id_like",
        lua.create_function(move |_, ()| Ok(sysinfo::System::distribution_id_like()))?,
      )?;
      sys.set(
        "kernel_long_version",
        lua.create_function(move |_, ()| Ok(sysinfo::System::kernel_long_version()))?,
      )?;
      sys.set(
        "host_name",
        lua.create_function(move |_, ()| Ok(sysinfo::System::host_name()))?,
      )?;
      sys.set(
        "cpu_arch",
        lua.create_function(move |_, ()| Ok(sysinfo::System::cpu_arch()))?,
      )?;
      sys.set(
        "physical_core_count",
        lua.create_function(move |_, ()| Ok(sysinfo::System::physical_core_count()))?,
      )?;

      Ok(sys)
    })
    .into();

  create_std_module("kvdb")
    .add_function("open", |_, name: String| {
      Ok(LuluSledDB {
        db: sled::open(name).map_err(mlua::Error::external)?,
      })
    })
    .on_register(|_, db_mod| Ok(db_mod))
    .add_file("kvdb.lua", include_str!("builtins/net/kvdb.lua"))
    .into();

  create_std_module("archive")
    .on_register(|lua, archive_mod| {
      let zip_mod = lua.create_table()?;
      zip_mod.set(
        "create",
        lua.create_function(|_, (archive_path, files): (String, Vec<String>)| {
          let file = File::create(&archive_path).map_err(|e| LuaError::external(e))?;
          let mut zip = ZipWriter::new(file);
          let options: FileOptions<ExtendedFileOptions> =
            FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

          for path in files {
            let mut f = File::open(&path).map_err(|e| LuaError::external(e))?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).map_err(|e| LuaError::external(e))?;
            zip
              .start_file(path.clone(), options.clone())
              .map_err(|e| LuaError::external(e))?;
            zip.write_all(&buf).map_err(|e| LuaError::external(e))?;
          }

          zip.finish().map_err(LuaError::external)?;
          Ok(())
        })?,
      )?;

      zip_mod.set(
        "extract",
        lua.create_function(|_, (archive_path, dest_dir): (String, String)| {
          let file = File::open(&archive_path).map_err(|e| LuaError::external(e))?;
          let mut archive = zip::ZipArchive::new(file).map_err(|e| LuaError::external(e))?;

          std::fs::create_dir_all(&dest_dir).ok();

          for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| LuaError::external(e))?;
            let out_path = std::path::Path::new(&dest_dir).join(file.name());

            if file.name().ends_with('/') {
              std::fs::create_dir_all(&out_path).ok();
            } else {
              if let Some(p) = out_path.parent() {
                std::fs::create_dir_all(p).ok();
              }
              let mut outfile = File::create(&out_path).map_err(|e| LuaError::external(e))?;
              std::io::copy(&mut file, &mut outfile).map_err(|e| LuaError::external(e))?;
            }
          }

          Ok(())
        })?,
      )?;

      use flate2::read::GzDecoder;
      use flate2::write::GzEncoder;
      let tar_mod = lua.create_table()?;

      tar_mod.set(
        "create",
        lua.create_function(|_, (archive_path, files): (String, Vec<String>)| {
          let tar_gz = File::create(&archive_path).map_err(|e| LuaError::external(e))?;
          let enc = GzEncoder::new(tar_gz, flate2::Compression::default());
          let mut tar = tar::Builder::new(enc);

          for path in files {
            tar.append_path(&path).map_err(|e| LuaError::external(e))?;
          }

          tar.into_inner().map_err(LuaError::external)?;
          Ok(())
        })?,
      )?;

      tar_mod.set(
        "extract",
        lua.create_function(|_, (archive_path, dest_dir): (String, String)| {
          let tar_gz = std::fs::File::open(&archive_path).map_err(|e| LuaError::external(e))?;
          let dec = GzDecoder::new(tar_gz);
          let mut archive = tar::Archive::new(dec);
          archive
            .unpack(std::path::Path::new(&dest_dir))
            .map_err(|e| LuaError::external(e))?;
          Ok(())
        })?,
      )?;

      archive_mod.set("tar", tar_mod)?;
      archive_mod.set("zip", zip_mod)?;
      Ok(archive_mod)
    })
    .into();

  create_std_module("serde")
    .on_register(|lua, serde_mod| {
      fn serde_value_to_lua(
        lua: &mlua::Lua,
        value: serde_json::Value,
      ) -> mlua::Result<mlua::Value> {
        use serde_json::Value::*;
        Ok(match value {
          Null => mlua::Value::Nil,
          Bool(b) => mlua::Value::Boolean(b),
          Number(n) => {
            if let Some(i) = n.as_i64() {
              mlua::Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
              mlua::Value::Number(f)
            } else {
              mlua::Value::Nil
            }
          }
          String(s) => mlua::Value::String(lua.create_string(&s)?),
          Array(arr) => {
            let tbl = lua.create_table()?;
            for (i, v) in arr.into_iter().enumerate() {
              tbl.set(i + 1, serde_value_to_lua(lua, v)?)?;
            }
            mlua::Value::Table(tbl)
          }
          Object(map) => {
            let tbl = lua.create_table()?;
            for (k, v) in map.into_iter() {
              tbl.set(k, serde_value_to_lua(lua, v)?)?;
            }
            mlua::Value::Table(tbl)
          }
        })
      }
      fn serde_into_json<V>(lua: &mlua::Lua, value: V) -> mlua::Result<mlua::Value>
      where
        V: serde::Serialize,
      {
        // Convert serde::Serialize -> serde_json::Value first
        let json_value = serde_json::to_value(value).map_err(mlua::Error::external)?;
        serde_value_to_lua(lua, json_value)
      }

      let json_mod = lua.create_table()?;
      json_mod.set(
        "decode",
        lua.create_function(|lua, text: String| {
          serde_value_to_lua(
            lua,
            serde_json::from_str::<serde_json::Value>(&text).map_err(mlua::Error::external)?,
          )
        })?,
      )?;
      json_mod.set(
        "encode",
        lua.create_function(|_, val: mlua::Table| {
          serde_json::to_string(&val).map_err(LuaError::external)
        })?,
      )?;
      serde_mod.set("json", json_mod)?;
      let yaml_mod = lua.create_table()?;
      yaml_mod.set(
        "decode",
        lua.create_function(|lua, text: String| {
          serde_into_json(
            lua,
            serde_yaml::from_str::<serde_yaml::Value>(&text).map_err(mlua::Error::external)?,
          )
        })?,
      )?;
      yaml_mod.set(
        "encode",
        lua.create_function(|_, val: mlua::Table| {
          serde_yaml::to_string(&val).map_err(LuaError::external)
        })?,
      )?;
      serde_mod.set("yaml", yaml_mod)?;

      Ok(serde_mod)
    })
    .into();

  create_std_module("net")
    .on_register(|lua, net_mod| {
      // HTTP client
      let http_mod = lua.create_table()?;
      let client = Client::builder()
        .user_agent("Lulu/1.0")
        .build()
        .map_err(LuaError::external)?;
      lua
        .globals()
        .set("__reqwest_client", lua.create_any_userdata(client)?)?;

      http_mod.set(
        "request",
        lua.create_async_function(|lua, req_table: mlua::Table| async move {
          let client = lua.globals().get::<mlua::AnyUserData>("__reqwest_client")?;
          let client = client.borrow::<Client>()?;

          let url: String = req_table.get("url")?;
          let method: Option<String> = req_table.get("method").ok();
          let body: Option<mlua::Value> = req_table.get("body").ok();
          let headers: Option<HashMap<String, String>> = req_table.get("headers").ok();

          let mut req = client.request(
            Method::from_bytes(method.unwrap_or_else(|| "GET".to_string()).as_bytes())
              .map_err(LuaError::external)?,
            &url,
          );

          if let Some(hmap) = headers {
            let mut hdrs = HeaderMap::new();
            for (k, v) in hmap {
              hdrs.insert(
                HeaderName::from_bytes(k.as_bytes()).map_err(mlua::Error::external)?,
                HeaderValue::from_str(&v).map_err(mlua::Error::external)?,
              );
            }
            req = req.headers(hdrs);
          }

          if let Some(body) = body {
            match body {
              mlua::Value::UserData(data) => {
                let data = data.borrow::<LuluByteArray>()?;
                req = req.body(data.bytes.clone());
              }
              mlua::Value::String(str) => req = req.body(str.to_str()?.to_string()),
              mlua::Value::Nil => {}
              _ => {
                eprintln!("Unsupported body type, only ByteArray and String is allowed.")
              }
            }
          }

          let resp = req.send().await.map_err(LuaError::external)?;
          let status = resp.status().as_u16();
          let res_headers: HashMap<String, String> = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
          let bytes = resp.bytes().await.map_err(LuaError::external)?;

          // Build Lua result
          let res = lua.create_table()?;
          res.set(
            "body",
            LuluByteArray {
              bytes: bytes.to_vec(),
            },
          )?;
          res.set("status", status)?;
          res.set("headers", res_headers)?;

          Ok(res)
        })?,
      )?;

      http_mod.set(
        "serve",
        lua.create_async_function(
          |lua, (addr, handler): (String, mlua::Function)| async move {
            let (req_tx, mut req_rx) = mpsc::channel::<ServerRequest>(32);
            let handler_key = lua.create_registry_value(handler)?;
            let lua = lua.clone();

            let app = Router::new()
              .fallback(any(axum_handler))
              .with_state(req_tx.clone());

            let socket_addr: SocketAddr = addr.parse().map_err(LuaError::external)?;
            let listener = tokio::net::TcpListener::bind(socket_addr)
              .await
              .map_err(LuaError::external)?;

            let listener = tokio::spawn(async move {
              if let Err(e) = axum::serve(listener, app).await {
                eprintln!("Server error: {}", e);
              }
            });

            tokio::spawn(async move {
              while let Some(server_req) = req_rx.recv().await {
                let handler = lua.registry_value::<mlua::Function>(&handler_key)?;
                let (parts, body) = server_req.req.into_parts();
                let body_bytes = axum::body::to_bytes(body, 1024 * 1024)
                  .await
                  .map_err(LuaError::external)?;

                let req_table = lua.create_table()?;
                req_table.set("method", parts.method.to_string())?;
                let uri = parts.uri.to_string();
                let host = parts
                  .headers
                  .get("host")
                  .and_then(|v| v.to_str().ok())
                  .unwrap_or("");
                req_table.set("host", host)?;
                req_table.set("uri", uri)?;

                let headers_table = lua.create_table()?;
                for (k, v) in parts.headers.iter() {
                  headers_table.set(k.to_string(), v.to_str().unwrap_or(""))?;
                }
                req_table.set("headers", headers_table)?;
                req_table.set(
                  "body",
                  LuluByteArray {
                    bytes: body_bytes.to_vec(),
                  },
                )?;

                let resp_handled = handler.call_async(req_table).await;

                match resp_handled.clone() {
                  Err(e) => {
                    eprintln!("{}", e);
                  }
                  _ => {}
                };

                let resp_table: mlua::Table = resp_handled?;
                let status: u16 = resp_table.get("status").unwrap_or(200);
                let body: mlua::Value = resp_table.get("body").unwrap_or(mlua::Value::Nil);
                let headers: HashMap<String, String> =
                  resp_table.get("headers").unwrap_or_default();

                let mut header_map = HeaderMap::new();
                for (k, v) in headers {
                  header_map.insert(
                    HeaderName::from_bytes(k.as_bytes()).map_err(LuaError::external)?,
                    HeaderValue::from_str(&v).map_err(LuaError::external)?,
                  );
                }

                let body_bytes = match body {
                  mlua::Value::String(s) => s.as_bytes().to_vec(),
                  mlua::Value::UserData(ud) => ud.borrow::<LuluByteArray>()?.bytes.clone(),
                  _ => Vec::new(),
                };

                let resp = Response::builder()
                  .status(StatusCode::from_u16(status).map_err(LuaError::external)?)
                  .header("x-powered-by", "Lulu")
                  .body(axum::body::Body::from(body_bytes))
                  .map_err(LuaError::external)?;

                let _ = server_req.resp_tx.send(resp);
              }
              Ok::<(), LuaError>(())
            });

            TOK_ASYNC_HANDLES.lock().unwrap().push(listener);
            Ok(())
          },
        )?,
      )?;

      net_mod.set("http", http_mod)?;

      // TCP
      let tcp_mod = lua.create_table()?;
      tcp_mod.set(
        "connect",
        lua.create_async_function(|_, addr: String| async move {
          let stream = TcpStream::connect(addr)
            .await
            .map_err(mlua::Error::external)?;
          Ok(LuluTcpStream::new(stream))
        })?,
      )?;
      tcp_mod.set(
        "listen",
        lua.create_async_function(|_, addr: String| async move {
          let listener = TcpListener::bind(addr)
            .await
            .map_err(mlua::Error::external)?;
          Ok(LuluTcpListener {
            listener: Arc::new(listener),
          })
        })?,
      )?;
      net_mod.set("tcp", tcp_mod)?;

      let udp_mod = lua.create_table()?;
      udp_mod.set(
        "bind",
        lua.create_async_function(|_, addr: String| async move {
          let socket = UdpSocket::bind(addr).await.map_err(mlua::Error::external)?;
          Ok(LuluUdpSocket::new(socket))
        })?,
      )?;
      net_mod.set("udp", udp_mod)?;

      let ws_mod = lua.create_table()?;
      ws_mod.set(
        "connect",
        lua.create_async_function(|_, url: String| async move {
          let (ws_stream, _) = connect_async(url).await.map_err(mlua::Error::external)?;
          Ok(LuluWebSocket::new(ws_stream))
        })?,
      )?;
      net_mod.set("websocket", ws_mod)?;

      Ok(net_mod)
    })
    .add_file("net.lua", include_str!("builtins/net/net.lua"))
    .add_file("http.lua", include_str!("builtins/net/http.lua"))
    .add_macro(
      "error_res",
      vec!["code".into(), "message".into()],
      "Response { status = $code, body = $message }",
    )
    .add_macro(
      "json_res",
      vec!["message".into()],
      "Response { status = 200, body = serde.json.encode($message) }",
    )
    .depend_on("serde".to_string())
    .into();

  create_std_module("threads")
    .on_register(|lua, threads_mod| {
      threads_mod.set(
        "spawn",
        lua.create_async_function(|lua, func: mlua::Function| async move {
          let handle = tokio::spawn(async move { func.call_async::<mlua::Value>(()).await });

          let handle = Arc::new(Mutex::new(Some(handle)));
          let handle_ref = handle.clone();

          TOK_ASYNC_HANDLES
            .lock()
            .unwrap()
            .push(tokio::spawn(async move {
              tokio::spawn(async move {
                let join_handle = {
                  let mut lock = handle.lock().unwrap();
                  lock.take()
                };

                if let Some(jh) = join_handle {
                  let _ = jh.await;
                }
              });
            }));

          Ok(lua.create_any_userdata(LuluThreadHandle { handle: handle_ref })?)
        })?,
      )?;

      // threads.join(task)
      threads_mod.set(
        "join",
        lua.create_async_function(|_, handle_ud: mlua::AnyUserData| async move {
          let handle_arc = {
            let handle = handle_ud.borrow::<LuluThreadHandle>()?;
            handle.handle.clone()
          };

          let join_handle_opt = {
            let mut opt = handle_arc.lock().unwrap();
            opt.take()
          };

          if let Some(join_handle) = join_handle_opt {
            let result = join_handle.await.map_err(LuaError::external)??;
            Ok(result)
          } else {
            Ok(mlua::Value::Nil)
          }
        })?,
      )?;

      threads_mod.set(
        "sleep",
        lua.create_async_function(|_, ms: u64| async move {
          tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
          Ok(())
        })?,
      )?;

      Ok(threads_mod)
    })
    .into();
}
