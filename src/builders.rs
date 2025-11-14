use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, RwLock};

pub trait BuilderTrait: Send + Sync {
  fn build(&self, path: &PathBuf, args: &[String]) -> mlua::Result<()>;
}

pub struct CargoBuilder;
impl BuilderTrait for CargoBuilder {
  fn build(&self, path: &PathBuf, args: &[String]) -> mlua::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(path).args(args);
    let status = cmd.status().map_err(mlua::Error::external)?;
    if status.success() {
      Ok(())
    } else {
      Err(mlua::Error::external(format!("Cargo failed: {:?}", status)))
    }
  }
}

pub struct MakeBuilder;
impl BuilderTrait for MakeBuilder {
  fn build(&self, path: &PathBuf, args: &[String]) -> mlua::Result<()> {
    let mut cmd = Command::new("make");
    cmd.current_dir(path).args(args);
    let status = cmd.status().map_err(mlua::Error::external)?;
    if status.success() {
      Ok(())
    } else {
      Err(mlua::Error::external(format!("Make failed: {:?}", status)))
    }
  }
}

pub struct CMakeBuilder;
impl BuilderTrait for CMakeBuilder {
  fn build(&self, path: &PathBuf, args: &[String]) -> mlua::Result<()> {
    // i think i'll do with: cmake . && cmake --build .
    // feel free to fix
    let mut configure = Command::new("cmake");
    configure.current_dir(path).args(args);
    let status = configure.status().map_err(mlua::Error::external)?;
    if !status.success() {
      return Err(mlua::Error::external(format!(
        "CMake configure failed: {:?}",
        status
      )));
    }

    let mut build = Command::new("cmake");
    build.current_dir(path).arg("--build").arg(".");
    let status = build.status().map_err(mlua::Error::external)?;
    if status.success() {
      Ok(())
    } else {
      Err(mlua::Error::external(format!(
        "CMake build failed: {:?}",
        status
      )))
    }
  }
}

pub struct GCCBuilder;
impl BuilderTrait for GCCBuilder {
  fn build(&self, path: &PathBuf, args: &[String]) -> mlua::Result<()> {
    let mut cmd = Command::new("gcc");
    cmd.current_dir(path).args(args);
    let status = cmd.status().map_err(mlua::Error::external)?;
    if status.success() {
      Ok(())
    } else {
      Err(mlua::Error::external(format!("GCC failed: {:?}", status)))
    }
  }
}

lazy_static::lazy_static! {
  pub static ref BUILDERS: RwLock<HashMap<String, Arc<dyn BuilderTrait>>> =
      RwLock::new(HashMap::new());
}

pub fn register_default_builders() {
  let mut map = BUILDERS.write().unwrap();

  map.insert("cargo".into(), Arc::new(CargoBuilder));
  map.insert("make".into(), Arc::new(MakeBuilder));
  map.insert("cmake".into(), Arc::new(CMakeBuilder));
  map.insert("gcc".into(), Arc::new(GCCBuilder));
}

pub fn build_path(
  builder: impl Into<String>,
  path: impl Into<PathBuf>,
  args: impl Into<Vec<String>>,
) -> mlua::Result<()> {
  let builder_name = builder.into();

  let builder = BUILDERS
    .read()
    .unwrap()
    .get(&builder_name)
    .cloned()
    .ok_or_else(|| mlua::Error::external(format!("Builder '{}' not found", builder_name)))?;

  builder.build(&path.into(), &args.into())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_builders() {
    register_default_builders();

    let _ = build_path(
      "cargo",
      "./my_rust_project",
      &["build".into(), "--release".into()],
    );

    let _ = build_path("make", "./my_c_project", &[]);

    let _ = build_path("cmake", "./my_cmake_project", &[".".into()]);

    let _ = build_path(
      "gcc",
      "./my_c_project",
      &["-o".into(), "myprog".into(), "main.c".into()],
    );
  }
}
