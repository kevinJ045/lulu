use clap::{Parser, Subcommand};
use colored::*;
use mlua::{Lua, UserData};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread;

#[derive(Parser)]
#[command(name = "rew")]
#[command(version = env!("CARGO_PKG_VERSION"),)]
#[command(about = "A Rust-based Rew runtime using deno_core")]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  Run {
    #[arg(name = "FILE")]
    file: PathBuf,

    #[arg(trailing_var_arg = true)]
    args: Vec<String>,
  },
  Bundle {
    #[arg(name = "FILE")]
    file: PathBuf,

    #[arg(name = "OUTPUT_FILE")]
    output: PathBuf,
  },
  Compile {
    #[arg(name = "FILE")]
    file: PathBuf,

    #[arg(name = "OUTPUT_FILE")]
    output: PathBuf,
  },
  // Exec {
  //   #[arg(name = "CODE")]
  //   code: String,

  //   #[arg(trailing_var_arg = true)]
  //   args: Vec<String>,
  // },
  // Test {
  //   #[arg(name = "FILE")]
  //   file: PathBuf,

  //   #[arg(short, long, help = "Specify the tests to run")]
  //   test: Option<String>,
  // },
}

#[derive(Debug, Clone)]
pub struct LuluConf {
  pub manifest: Option<mlua::Table>,
  pub mods: Option<HashMap<String, String>>,
}

impl UserData for LuluConf {
  fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
    fields.add_field_method_get("mods", |_, this| Ok(this.mods.clone()));
    fields.add_field_method_get("manifest", |_, this| Ok(this.manifest.clone()));
  }
}

#[derive(Debug, Clone)]
pub enum LuluModSource {
  Path(String),      // Load from a file
  Code(String),      // Raw Lua source code
  Bytecode(Vec<u8>), // Precompiled bytecode
}

#[derive(Debug, Clone)]
pub struct LuluMod {
  pub name: String,
  pub source: LuluModSource,
  pub conf: Option<LuluConf>,
}

#[derive(Debug, Clone)]
pub struct Lulu {
  pub mods: Vec<LuluMod>,
  pub lua: Lua,
  pub args: Vec<String>,
}

impl Lulu {
  pub fn new(args: Option<Vec<String>>) -> Lulu {
    let mods = Vec::new();
    let lua = Lua::new();

    return Lulu {
      mods,
      lua,
      args: args.unwrap_or(Vec::new()),
    };
  }

  pub fn preload_mods(&self) -> mlua::Result<()> {
    let mods = self.mods.clone();

    let f = self.lua.create_function(move |_, ()| {
      let mut embedded_scripts: Vec<String> = Vec::new();

      for lmod in &mods {
        embedded_scripts.push(lmod.name.clone());
      }

      Ok(embedded_scripts)
    })?;

    let lulu_rc = Rc::new(RefCell::new(self.clone()));
    let l = self.lua.create_function({
      let lulu_rc = lulu_rc.clone();
      move |_, name: String| -> mlua::Result<mlua::Value> {
        Ok(lulu_rc.borrow_mut().exec_mod(&name)?)
      }
    })?;

    self.lua.globals().set("get_mods", f)?;
    self.lua.globals().set("exec_mod", l)?;
    self.lua.globals().set("args", self.args.clone())?;

    self
      .lua
      .load(
        r#"
          local embedded = get_mods()
          package.preload = package.preload or {}
          for key, name in pairs(embedded) do
            package.preload[name] = function()
              return exec_mod(name)
            end
          end
        "#,
      )
      // .set(mlua::Table (embedded_scripts_clone))
      .exec()?;

    Ok(())
  }

  pub fn add_mod(&mut self, lmod: LuluMod) {
    self.mods.push(lmod);
  }

  pub fn add_mod_from_code(&mut self, name: String, code: String, conf: Option<LuluConf>) {
    self.add_mod(LuluMod {
      name,
      source: LuluModSource::Code(code),
      conf,
    });
  }

  pub fn add_mod_from_bytecode(&mut self, name: String, bytecode: Vec<u8>, conf: Option<LuluConf>) {
    self.add_mod(LuluMod {
      name,
      source: LuluModSource::Bytecode(bytecode),
      conf,
    });
  }

  pub fn add_mod_from_file(
    &mut self,
    name: String,
    path: PathBuf,
    conf: Option<LuluConf>,
  ) -> mlua::Result<()> {
    let raw = std::fs::read(&path)?;

    let source = match std::str::from_utf8(&raw) {
      Ok(code) => LuluModSource::Code(code.to_string()),
      Err(_) => LuluModSource::Bytecode(raw),
    };

    self.add_mod(LuluMod { name, source, conf });
    Ok(())
  }

  pub fn exec_mod(&self, name: &str) -> mlua::Result<mlua::Value> {
    let lmod = self
      .mods
      .iter()
      .find(|m| m.name == name)
      .ok_or_else(|| mlua::Error::RuntimeError(format!("Module {} not found", name)))?;

    let chunk = match &lmod.source {
      LuluModSource::Code(code) => self.lua.load(code),
      LuluModSource::Bytecode(bytes) => self.lua.load(&bytes[..]),
      LuluModSource::Path(path) => {
        let code = std::fs::read_to_string(path)?;
        self.lua.load(code)
      }
    };

    let env = if let Some(env) = chunk.environment() {
      env.clone()
    } else {
      let env = self.lua.create_table()?;

      let mt = self.lua.create_table()?;
      mt.set("__index", self.lua.globals())?;
      env.set_metatable(Some(mt))?;

      env
    };

    let lmod_table = self.lua.create_table()?;

    if let Some(conf) = lmod.conf.clone() {
      let p = self.lua.create_userdata::<LuluConf>(conf)?;

      lmod_table.set("conf", p)?;
    }
    lmod_table.set("name", name)?;

    env.set("mod", lmod_table)?;

    let chunk = chunk.set_environment(env);

    chunk.eval()
  }

  pub fn entry_mod_path(&mut self, path: PathBuf) -> mlua::Result<()> {
    let conf = if let Some(root_path) = find_lulu_conf(path.clone()) {
      let c = load_lulu_conf(&self.lua, root_path.clone())?;
      // mods is a hashmap of <string, string> as <name, file path>
      if let Some(mods) = c.mods.clone() {
        for (name, modpath) in mods {
          self.add_mod_from_file(
            String::from(name),
            root_path.parent().unwrap().join(modpath),
            Some(c.clone()),
          )?;
        }
      }
      Some(c)
    } else {
      None
    };

    self.add_mod_from_file(String::from("main"), path.clone(), conf)?;

    self.preload_mods()?;

    Ok(())
  }

  pub fn exec_entry_mod_path(&mut self, path: PathBuf) -> mlua::Result<()> {
    self.entry_mod_path(path)?;
    self.exec_mod("main")?;

    Ok(())
  }
}

pub fn load_lulu_conf(lua: &Lua, path: PathBuf) -> mlua::Result<LuluConf> {
  let code = std::fs::read_to_string(path)?;
  lua.load(&code).set_name("lulu.conf.lua").exec()?;

  let globals = lua.globals();
  let manifest: Option<mlua::Table> = globals.get("manifest").ok();
  let mods = globals
    .get::<HashMap<String, String>>("mods")
    .map(|a| Some(a))
    .unwrap_or_else(|_| None);

  Ok(LuluConf { manifest, mods })
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

pub fn lua_to_bytecode(lua: &Lua, code: &str) -> mlua::Result<Vec<u8>> {
  let func: mlua::Function = lua.load(code).into_function()?;

  let dump: String = lua.load("return string.dump(...)").call(func)?;

  Ok(dump.into_bytes())
}

pub fn write_bin(output: &PathBuf, bytes: HashMap<String, Vec<u8>>) -> std::io::Result<()> {
  let mut bin = OpenOptions::new().append(true).open(output)?;

  let mut total_size: u64 = 0;

  for (name, data) in &bytes {
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len() as u32;
    let data_len = data.len() as u64;

    bin.write_all(&name_len.to_le_bytes())?;
    bin.write_all(name_bytes)?;
    bin.write_all(&data_len.to_le_bytes())?;
    bin.write_all(data)?;

    total_size += 4 + name_bytes.len() as u64 + 8 + data_len;
  }

  let module_count = bytes.len() as u64;

  bin.write_all(&total_size.to_le_bytes())?;
  bin.write_all(&module_count.to_le_bytes())?;
  bin.write_all(b"LUL!")?;

  Ok(())
}

pub fn make_bin(output: &PathBuf, bytes: HashMap<String, Vec<u8>>) -> std::io::Result<()> {
  let exe_path = std::env::current_exe()?;
  std::fs::copy(&exe_path, output)?;
  write_bin(output, bytes);
  Ok(())
}

pub fn load_embedded_scripts() -> Option<HashMap<String, Vec<u8>>> {
  let exe = std::env::current_exe().ok()?;
  let mut f = File::open(&exe).ok()?;

  f.seek(SeekFrom::End(-20)).ok()?;
  let mut footer = [0u8; 20];
  f.read_exact(&mut footer).ok()?;

  if &footer[16..20] != b"LUL!" {
    return None;
  }

  let total_size = u64::from_le_bytes(footer[0..8].try_into().unwrap());
  let module_count = u64::from_le_bytes(footer[8..16].try_into().unwrap());

  let mut modules = HashMap::new();

  f.seek(SeekFrom::End(-(20 + total_size as i64))).ok()?;

  for _ in 0..module_count {
    let mut name_len_buf = [0u8; 4];
    f.read_exact(&mut name_len_buf).ok()?;
    let name_len = u32::from_le_bytes(name_len_buf) as usize;

    let mut name_bytes = vec![0u8; name_len];
    f.read_exact(&mut name_bytes).ok()?;
    let name = String::from_utf8(name_bytes).ok()?;

    let mut data_len_buf = [0u8; 8];
    f.read_exact(&mut data_len_buf).ok()?;
    let data_len = u64::from_le_bytes(data_len_buf) as usize;

    let mut data = vec![0u8; data_len];
    f.read_exact(&mut data).ok()?;

    modules.insert(name, data);
  }

  Some(modules)
}

fn main() -> mlua::Result<()> {
  if let Some(mods) = load_embedded_scripts() {
    let mut lulu = Lulu::new(Some(std::env::args().collect()));
    for (name, data) in mods.iter() {
      if name == "lulu.conf.lua" {
        continue;
      }
      lulu.add_mod_from_bytecode(name.clone(), data.clone(), None);
    }

    lulu.preload_mods()?;
    lulu.exec_mod("main")?;
  } else {
    let cli = Cli::parse();

    match &cli.command {
      Commands::Run { file, args } => {
        let mut lulu = Lulu::new(Some(args.clone()));
        lulu.exec_entry_mod_path(file.clone())?;
      }
      Commands::Bundle { file, output } => {
        let mut lulu = Lulu::new(None);
        lulu.entry_mod_path(file.clone())?;

        let mut combined_bytes = HashMap::<String, Vec<u8>>::new();

        for lmod in &lulu.mods {
          match &lmod.source {
            LuluModSource::Code(code) => {
              combined_bytes.insert(
                lmod.name.clone(),
                lua_to_bytecode(&lulu.lua, code.as_str())?,
              );
            }
            LuluModSource::Bytecode(bytes) => {
              combined_bytes.insert(lmod.name.clone(), bytes.clone());
            }
            _ => {}
          }
        }

        make_bin(output, combined_bytes)?;
      }
      Commands::Compile { file, output } => {}
    }
  }

  Ok(())
}
