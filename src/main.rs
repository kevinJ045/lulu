use clap::{Parser, Subcommand};
use colored::*;
use mlua::{Lua, UserData, Value};
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

fn table_to_lua_string(table: &mlua::Table) -> mlua::Result<String> {
  let mut parts = Vec::new();
  for pair in table.pairs::<mlua::Value, mlua::Value>() {
    let (k, v) = pair?;
    let key = match k {
      mlua::Value::String(s) => s.to_str()?.to_string(),
      mlua::Value::Integer(i) => "".to_string(),
      _ => "<unsupported>".to_string(),
    };
    let val = match v {
      mlua::Value::String(s) => format!(r#""{}""#, s.to_str()?),
      mlua::Value::Integer(i) => i.to_string(),
      mlua::Value::Boolean(b) => b.to_string(),
      mlua::Value::Table(b) => table_to_lua_string(&b)?,
      _ => "<unsupported>".to_string(),
    };
    parts.push(if key.len() > 0 { format!("{} = {}", key, val) } else {format!("{}", val)  });
  }
  Ok(format!("{{ {} }}", parts.join(", ")))
}

pub fn conf_to_string(conf: &LuluConf) -> mlua::Result<String> {
  let mut out = String::from("return {\n");

  if let Some(manifest) = &conf.manifest {
    out.push_str("  manifest = ");
    out.push_str(&table_to_lua_string(manifest)?);
    out.push_str(",\n");
  }

  if let Some(mods) = &conf.mods {
    out.push_str("  mods = { ");
    for (k, v) in mods {
      out.push_str(&format!(r#"{} = "{}","#, k, v));
    }
    out.push_str(" },\n");
  }

  out.push('}');
  Ok(out)
}

#[derive(Debug, Clone)]
pub struct LuLib {
  pub bytes: Vec<u8>,
  pub conf: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub enum LuluModSource {
  Path(String),
  Code(String),
  Bytecode(Vec<u8>),
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

  globals.set("manifest", mlua::Value::Nil)?;
  globals.set("mods", mlua::Value::Nil)?;

  Ok(LuluConf { manifest, mods })
}

pub fn load_lulu_conf_from_bytecode(lua: &Lua, bytecode: Vec<u8>) -> mlua::Result<LuluConf> {

  let table = lua.load(&bytecode).set_name("lulu.conf.lua").eval::<mlua::Table>()?;
  let manifest: Option<mlua::Table> = table.get("manifest").ok();
  let mods = table
    .get::<HashMap<String, String>>("mods")
    .map(|a| Some(a))
    .unwrap_or_else(|_| None);

  table.set("manifest", mlua::Value::Nil)?;
  table.set("mods", mlua::Value::Nil)?;

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

  let dump: mlua::String = lua.load("return string.dump(...)").call(func)?;

  Ok(dump.as_bytes().to_vec())
}

pub fn write_bundle<W: Write>(
  writer: &mut W,
  bytes: HashMap<String, LuLib>,
) -> std::io::Result<()> {
  let mut total_size: u64 = 0;

  let mut unique_confs: Vec<Vec<u8>> = Vec::new();
  let mut conf_map: HashMap<Vec<u8>, u32> = HashMap::new();

  for (_name, lib) in &bytes {
    if let Some(conf) = &lib.conf {
      if !conf_map.contains_key(conf) {
        let idx = unique_confs.len() as u32;
        unique_confs.push(conf.clone());
        conf_map.insert(conf.clone(), idx);
      }
    }
  }

  for (name, lib) in &bytes {
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len() as u32;
    let data_len = lib.bytes.len() as u64;
    let conf_idx = lib
      .conf
      .as_ref()
      .and_then(|c| conf_map.get(c).copied())
      .unwrap_or(u32::MAX);

    writer.write_all(&name_len.to_le_bytes())?;
    writer.write_all(name_bytes)?;
    writer.write_all(&data_len.to_le_bytes())?;
    writer.write_all(&lib.bytes)?;
    writer.write_all(&conf_idx.to_le_bytes())?;

    total_size += 4 + name_bytes.len() as u64 + 8 + data_len + 4;
  }

  let conf_count = unique_confs.len() as u64;
  writer.write_all(&conf_count.to_le_bytes())?;
  total_size += 8;

  for conf in &unique_confs {
    let len = conf.len() as u64;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(conf)?;
    total_size += 8 + len;
  }

  let module_count = bytes.len() as u64;

  writer.write_all(&total_size.to_le_bytes())?;
  writer.write_all(&module_count.to_le_bytes())?;
  writer.write_all(b"LUL!")?;

  Ok(())
}

pub fn write_bin(output: &PathBuf, bytes: HashMap<String, LuLib>) -> std::io::Result<()> {
  let mut bin = OpenOptions::new().append(true).open(output)?;
  write_bundle(&mut bin, bytes)
}

pub fn make_bin(output: &PathBuf, bytes: HashMap<String, LuLib>) -> std::io::Result<()> {
  let exe_path = std::env::current_exe()?;
  std::fs::copy(&exe_path, output)?;
  write_bin(output, bytes)?;
  Ok(())
}

fn load_bundle_from_reader<R: Read + Seek>(
  reader: &mut R,
) -> std::io::Result<HashMap<String, LuLib>> {
  // Read footer
  reader.seek(SeekFrom::End(-20))?;
  let mut footer = [0u8; 20];
  reader.read_exact(&mut footer)?;

  if &footer[16..20] != b"LUL!" {
    return Err(std::io::Error::new(
      std::io::ErrorKind::InvalidData,
      "Invalid bundle magic",
    ));
  }

  let total_size = u64::from_le_bytes(footer[0..8].try_into().unwrap());
  let module_count = u64::from_le_bytes(footer[8..16].try_into().unwrap());

  reader.seek(SeekFrom::End(-(20 + total_size as i64)))?;

  let mut modules = HashMap::new();

  let mut module_meta: Vec<(String, Vec<u8>, u32)> = Vec::new();
  for _ in 0..module_count {
    let mut name_len_buf = [0u8; 4];
    reader.read_exact(&mut name_len_buf)?;
    let name_len = u32::from_le_bytes(name_len_buf) as usize;

    let mut name_bytes = vec![0u8; name_len];
    reader.read_exact(&mut name_bytes)?;
    let name = String::from_utf8(name_bytes)
      .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let mut data_len_buf = [0u8; 8];
    reader.read_exact(&mut data_len_buf)?;
    let data_len = u64::from_le_bytes(data_len_buf) as usize;

    let mut data = vec![0u8; data_len];
    reader.read_exact(&mut data)?;

    let mut conf_idx_buf = [0u8; 4];
    reader.read_exact(&mut conf_idx_buf)?;
    let conf_idx = u32::from_le_bytes(conf_idx_buf);

    module_meta.push((name, data, conf_idx));
  }

  // Read conf section
  let mut conf_count_buf = [0u8; 8];
  reader.read_exact(&mut conf_count_buf)?;
  let conf_count = u64::from_le_bytes(conf_count_buf);

  let mut confs: Vec<Vec<u8>> = Vec::new();
  for _ in 0..conf_count {
    let mut len_buf = [0u8; 8];
    reader.read_exact(&mut len_buf)?;
    let len = u64::from_le_bytes(len_buf) as usize;

    let mut conf = vec![0u8; len];
    reader.read_exact(&mut conf)?;
    confs.push(conf);
  }

  // Assemble modules
  for (name, data, conf_idx) in module_meta {
    let conf = if conf_idx != u32::MAX {
      Some(confs[conf_idx as usize].clone())
    } else {
      None
    };
    modules.insert(name, LuLib { bytes: data, conf });
  }

  Ok(modules)
}

pub fn load_lulib(path: &Path) -> std::io::Result<HashMap<String, LuLib>> {
  let mut f = File::open(path)?;
  load_bundle_from_reader(&mut f)
}

pub fn load_embedded_scripts() -> Option<HashMap<String, LuLib>> {
  let exe = std::env::current_exe().ok()?;
  let mut f = File::open(&exe).ok()?;
  load_bundle_from_reader(&mut f).ok()
}

fn run_bundle(mods: HashMap<String, LuLib>, args: Vec<String>) -> mlua::Result<()> {
  let mut lulu = Lulu::new(Some(args));

  for (name, data) in mods.iter() {
    let conf = if let Some(confbytes) = data.conf.clone() {
      Some(load_lulu_conf_from_bytecode(&lulu.lua, confbytes)?)
    } else {
      None
    };
    lulu.add_mod_from_bytecode(name.clone(), data.bytes.clone(), conf);
  }

  lulu.preload_mods()?;
  lulu.exec_mod("main")?;
  Ok(())
}

fn main() -> mlua::Result<()> {
  if let Some(mods) = load_embedded_scripts() {
    run_bundle(mods, std::env::args().collect())?;
  } else {
    let cli = Cli::parse();

    match &cli.command {
      Commands::Run { file, args } => {
        if file.extension().and_then(|s| s.to_str()) == Some("lulib") {
          let mods = load_lulib(file)?;
          run_bundle(mods, args.clone())?;
        } else {
          let mut lulu = Lulu::new(Some(args.clone()));
          lulu.exec_entry_mod_path(file.clone())?;
        }
      }
      Commands::Bundle { file, output } => {
        let mut lulu = Lulu::new(None);

        // let conf_path = find_lulu_conf(file.clone());

        lulu.entry_mod_path(file.clone())?;

        let mut combined_bytes = HashMap::<String, LuLib>::new();

        // if let Some(path) = conf_path {
        //   let conf_content = std::fs::read_to_string(&path)?;
        //   combined_bytes.insert(
        //     "lulu.conf.lua".to_string(),
        //     lua_to_bytecode(&lulu.lua, &conf_content)?,
        //   );
        // }

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
                  bytes: lua_to_bytecode(&lulu.lua, code.as_str())?,
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
            _ => {}
          }
        }

        if output.extension().and_then(|s| s.to_str()) == Some("lulib") {
          let mut f = File::create(output)?;
          write_bundle(&mut f, combined_bytes)?;
        } else {
          make_bin(output, combined_bytes)?;
        }
      }
      Commands::Compile { file, output } => {}
    }
  }

  Ok(())
}
