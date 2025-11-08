use crate::ops::std::create_std_module;
use mlua::Error as LuaError;

use std::fs::File;
use std::io::{Read, Write};
use zip::write::ExtendedFileOptions;
use zip::{ZipWriter, write::FileOptions};

pub fn into_module(){

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
}