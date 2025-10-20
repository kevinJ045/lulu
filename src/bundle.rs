use crate::conf::{conf_to_string, load_lulu_conf_from_bytecode};
use crate::lulu::{LuLib, Lulu, LuluModSource};
use crate::util::lua_to_bytecode;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

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

static EXEC_PATH: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

fn get_exec_path() -> PathBuf {
  let mut guard = EXEC_PATH.lock().unwrap();

  if let Some(path) = &*guard {
    return path.clone();
  }

  let path = std::env::var("LULU_EXEC_PATH")
    .map(PathBuf::from)
    .unwrap_or_else(|_| std::env::current_exe().unwrap());

  *guard = Some(path.clone());
  path
}

pub fn set_exec_path<P: Into<PathBuf>>(path: P) {
  let mut guard = EXEC_PATH.lock().unwrap();
  *guard = Some(path.into());
}

pub fn make_bin(output: &PathBuf, bytes: HashMap<String, LuLib>) -> std::io::Result<()> {
  let exe_path = get_exec_path();
  std::fs::copy(&exe_path, output)?;
  write_bin(output, bytes)?;
  Ok(())
}

fn load_bundle_from_reader<R: Read + Seek>(
  reader: &mut R,
) -> std::io::Result<HashMap<String, LuLib>> {
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

pub fn reg_bundle_nods(lulu: &mut Lulu, mods: HashMap<String, LuLib>) -> mlua::Result<()> {
  for (name, data) in mods.iter() {
    let conf = if let Some(confbytes) = data.conf.clone() {
      let conf = load_lulu_conf_from_bytecode(&lulu.lua, confbytes)?;

      if let Some(macros) = conf.macros.clone() {
        lulu.compiler.compile(&macros, None, None);
      }

      Some(conf)
    } else {
      None
    };

    if !lulu.mods.iter().any(|m| m.name == *name) {
      lulu.add_mod_from_bytecode(name.clone(), data.bytes.clone(), conf);
    }
  }

  Ok(())
}

pub async fn run_bundle(
  mods: HashMap<String, LuLib>,
  lulu: &mut Lulu
) -> mlua::Result<()> {

  reg_bundle_nods(lulu, mods)?;

  lulu.preload_mods()?;

  let main_name = lulu.find_mod("main")?;
  lulu.exec_final(main_name.as_str()).await?;
  Ok(())
}

pub fn bundle_lulu_or_exec(lulu: &mut Lulu, file: PathBuf, output: PathBuf) -> mlua::Result<()> {
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
    }
  }

  if output.extension().and_then(|s| s.to_str()) == Some("lulib") {
    let mut f = File::create(output)?;
    write_bundle(&mut f, combined_bytes)?;
  } else {
    make_bin(&output, combined_bytes)?;
  }

  Ok(())
}
