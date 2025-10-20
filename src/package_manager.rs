use anyhow::{Context, Result, anyhow};
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;
use zip::ZipArchive;

use crate::conf::{FetchField, load_lulu_conf, load_lulu_fetch_field};
use crate::resolver::{GitHubDependency, create_dirs, parse_github_dep};

#[derive(Debug, Clone)]
pub struct PackageInfo {
  pub name: String,
  pub version: Option<String>,
  #[allow(unused)]
  pub url: String,
  #[allow(unused)]
  pub cache_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PackageManager {
  cache_dir: PathBuf,
}

impl PackageManager {
  pub fn new() -> Result<Self> {
    let cache_dir = Self::get_cache_directory()?;
    fs::create_dir_all(&cache_dir)?;

    Ok(PackageManager { cache_dir })
  }

  fn get_cache_directory() -> Result<PathBuf> {
    let base = if cfg!(windows) {
      std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
          std::env::var("USERPROFILE")
            .map(|p| PathBuf::from(p).join("AppData/Roaming"))
            .unwrap_or_else(|_| PathBuf::from("C:/temp"))
        })
    } else {
      std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
          std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".cache"))
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
        })
    };

    Ok(base.join("lulu"))
  }

  fn cache_key(&self, url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
  }

  pub fn get_package_cache_path(&self, url: &str) -> PathBuf {
    self.cache_dir.join(self.cache_key(url))
  }

  pub fn is_cached(&self, url: &str) -> bool {
    let cache_path = self.get_package_cache_path(url);
    cache_path.exists() && cache_path.join("lulu.conf.lua").exists()
  }

  pub async fn install_package(&self, url: &str, project_path: &Path) -> Result<PackageInfo> {
    let cache_path = self.get_package_cache_path(url);

    if !self.is_cached(url) {
      self.fetch_package(url, &cache_path).await?;
    }

    let package_info = self.get_package_info(&cache_path, url)?;
    self.build_package(&cache_path).await?;
    self
      .copy_package_artifacts(&cache_path, project_path, &package_info)
      .await?;

    Ok(package_info)
  }

  pub async fn download_file(&self, url: &str) -> Result<PathBuf> {
    let cache_path = self.get_package_cache_path(url);
    fs::create_dir_all(cache_path.clone())?;

    let download_needed = if cache_path.exists() {
      fs::read_dir(&cache_path)?.next().is_none()
    } else {
      true
    };

    if download_needed {
      self.download_url(url, &cache_path).await?;
    }

    Ok(cache_path)
  }

  pub async fn download_url(&self, url: &str, cache_path: &Path) -> Result<()> {
    Ok(if url.ends_with(".lulib") {
      create_dirs(cache_path)?;
      self.download_lulib_package(url, cache_path, None).await?;
    } else if url.ends_with(".zip") {
      self.download_and_extract_zip(url, cache_path).await?;
    } else if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
      self.download_and_extract_tar_gz(url, cache_path).await?;
    } else {
      self.download_rogue_file(url, cache_path).await?;
    })
  }

  pub async fn fetch_package(&self, url: &str, cache_path: &Path) -> Result<()> {
    fs::create_dir_all(cache_path)?;

    if url.starts_with("github:") {
      self.handle_github_repo(url, cache_path).await?;
    } else if url.starts_with("http://") || url.starts_with("https://") {
      if url.ends_with(".git") {
        self.clone_git_repo(url, cache_path).await?;
      } else {
        self.download_url(url, cache_path).await?
      }
    } else {
      return Err(anyhow!("Unsupported package source: {}", url));
    }

    Ok(())
  }

  async fn handle_github_repo(&self, github_url: &str, cache_path: &Path) -> Result<()> {
    let github_dep = parse_github_dep(github_url)
      .ok_or_else(|| anyhow!("Invalid GitHub URL format: {}", github_url))?;

    let lua_conf_url = github_dep.raw_url("lulu.conf.lua");

    let response = reqwest::get(&lua_conf_url).await;
    match response {
      Ok(resp) if resp.status().is_success() => {
        let conf_content = resp.text().await?;
        let lua = mlua::Lua::new();

        if let Ok(Some(fetch)) = load_lulu_fetch_field(&lua, conf_content.clone()) {
          match fetch {
            FetchField::Code => self.clone_github_repo_code(&github_dep, cache_path).await?,
            FetchField::Lulib { url, include } => {
              create_dirs(cache_path)?;
              self
                .download_lulib_package(&url, cache_path, include)
                .await?;

              let conf_path = cache_path.join("lulu.conf.lua");
              fs::write(conf_path, conf_content)?;
            }
          }
        } else {
          println!("Repository has no fetch field, cloning to prepare build");
          self.clone_github_repo_code(&github_dep, cache_path).await?
        }
      }
      _ => {
        println!("Could not fetch lulu.conf.lua from GitHub, falling back to cloning repository");
        self.clone_github_repo_code(&github_dep, cache_path).await?
      }
    }

    Ok(())
  }

  async fn download_lulib_package(
    &self,
    url: &str,
    cache_path: &Path,
    include: Option<HashMap<String, Vec<String>>>,
  ) -> Result<()> {
    // Extract package name from URL
    let pathname = PathBuf::from(url.replace("http://", "").replace("https://", ""));
    let name = pathname
      .file_stem()
      .and_then(|s| s.to_str())
      .unwrap_or("package");

    let lulib_path = cache_path.join(format!(".lib/lulib/{}.lulib", name));
    fs::create_dir_all(lulib_path.parent().unwrap())?;

    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    fs::write(&lulib_path, &bytes)?;

    if let Some(include_map) = include {
      let current_platform = self.get_current_platform();
      if let Some(files) = include_map.get(current_platform) {
        let platform_dir = cache_path.join(current_platform);
        fs::create_dir_all(&platform_dir)?;

        for file_url in files {
          let filename = Path::new(file_url)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
          let dest = platform_dir.join(filename);

          if let Ok(response) = reqwest::get(file_url).await {
            if let Ok(bytes) = response.bytes().await {
              let _ = fs::write(dest, &bytes);
            }
          }
        }
      }
    }

    Ok(())
  }

  async fn clone_github_repo_code(
    &self,
    github_dep: &GitHubDependency,
    cache_path: &Path,
  ) -> Result<()> {
    let git_url = format!(
      "https://github.com/{}/{}.git",
      github_dep.username, github_dep.repo
    );

    self.clone_git_repo(&git_url, cache_path).await?;

    if let Some(branch) = &github_dep.branch {
      self.git_checkout(cache_path, branch)?;
    } else if let Some(commit) = &github_dep.commit {
      self.git_checkout(cache_path, commit)?;
    }

    if let Some(path) = &github_dep.path {
      let source_path = cache_path.join(path);
      if source_path.exists() {
        let temp_path = cache_path.parent().unwrap().join(format!(
          "{}_temp",
          cache_path.file_name().unwrap().to_string_lossy()
        ));
        fs::rename(&source_path, &temp_path)?;
        fs::remove_dir_all(cache_path)?;
        fs::rename(&temp_path, cache_path)?;
      }
    }

    Ok(())
  }

  async fn clone_git_repo(&self, git_url: &str, cache_path: &Path) -> Result<()> {
    let output = Command::new("git")
      .args(&["clone", "--depth", "1", git_url])
      .arg(cache_path)
      .output()
      .context("Failed to execute git clone")?;

    if !output.status.success() {
      return Err(anyhow!(
        "Git clone failed: {}",
        String::from_utf8_lossy(&output.stderr)
      ));
    }

    Ok(())
  }

  fn git_checkout(&self, repo_path: &Path, ref_name: &str) -> Result<()> {
    let output = Command::new("git")
      .current_dir(repo_path)
      .args(&["checkout", ref_name])
      .output()
      .context("Failed to execute git checkout")?;

    if !output.status.success() {
      return Err(anyhow!(
        "Git checkout failed: {}",
        String::from_utf8_lossy(&output.stderr)
      ));
    }

    Ok(())
  }

  async fn download_and_extract_zip(&self, url: &str, cache_path: &Path) -> Result<()> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;

    let temp_file = cache_path.join("download.zip");
    fs::write(&temp_file, &bytes)?;

    let file = File::open(&temp_file)?;
    let mut archive = ZipArchive::new(BufReader::new(file))?;

    let temp_extract = cache_path.join("extract_temp");
    fs::create_dir_all(&temp_extract)?;

    archive.extract(&temp_extract)?;

    let entries: Vec<_> = fs::read_dir(&temp_extract)?.collect::<Result<Vec<_>, _>>()?;

    if entries.len() == 1 && entries[0].file_type()?.is_dir() {
      let root_dir = entries[0].path();
      self.move_directory_contents(&root_dir, cache_path)?;
    } else {
      self.move_directory_contents(&temp_extract, cache_path)?;
    }

    fs::remove_dir_all(&temp_extract).ok();
    fs::remove_file(&temp_file).ok();

    Ok(())
  }

  async fn download_and_extract_tar_gz(&self, url: &str, cache_path: &Path) -> Result<()> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;

    let decoder = GzDecoder::new(&bytes[..]);
    let mut archive = Archive::new(decoder);

    let temp_extract = cache_path.join("extract_temp");
    fs::create_dir_all(&temp_extract)?;

    archive.unpack(&temp_extract)?;

    let entries: Vec<_> = fs::read_dir(&temp_extract)?.collect::<Result<Vec<_>, _>>()?;

    if entries.len() == 1 && entries[0].file_type()?.is_dir() {
      let root_dir = entries[0].path();
      self.move_directory_contents(&root_dir, cache_path)?;
    } else {
      self.move_directory_contents(&temp_extract, cache_path)?;
    }

    fs::remove_dir_all(&temp_extract).ok();

    Ok(())
  }

  async fn download_rogue_file(&self, url: &str, cache_path: &Path) -> Result<()> {
    let parsed_url = reqwest::Url::parse(url)?;

    let file_name = parsed_url
      .path_segments()
      .and_then(|segments| segments.last())
      .filter(|s| !s.is_empty())
      .ok_or("Could not extract file name from URL")
      .unwrap_or("file");

    let file_path = cache_path.join(file_name);

    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;

    fs::write(file_path, bytes)?;

    Ok(())
  }

  fn move_directory_contents(&self, source: &Path, dest: &Path) -> Result<()> {
    for entry in fs::read_dir(source)? {
      let entry = entry?;
      let dest_path = dest.join(entry.file_name());

      if entry.file_type()?.is_dir() {
        fs::create_dir_all(&dest_path)?;
        self.move_directory_contents(&entry.path(), &dest_path)?;
      } else {
        fs::copy(&entry.path(), &dest_path)?;
      }
    }
    Ok(())
  }

  pub fn get_package_info(&self, cache_path: &Path, url: &str) -> Result<PackageInfo> {
    let conf_path = cache_path.join("lulu.conf.lua");

    if !conf_path.exists() {
      return Err(anyhow!("Package does not contain lulu.conf.lua"));
    }

    let lua = mlua::Lua::new();
    let conf = load_lulu_conf(&lua, conf_path)?;

    let name = if let Some(manifest) = &conf.manifest {
      manifest
        .get::<String>("name")
        .unwrap_or_else(|_| "unknown".to_string())
    } else {
      "unknown".to_string()
    };

    let version = if let Some(manifest) = &conf.manifest {
      manifest.get::<String>("version").ok()
    } else {
      None
    };

    Ok(PackageInfo {
      name,
      version,
      url: url.to_string(),
      cache_path: cache_path.to_path_buf(),
    })
  }

  pub async fn build_package(&self, cache_path: &Path) -> Result<()> {
    let conf_path = cache_path.join("lulu.conf.lua");

    if !conf_path.exists() {
      return Ok(());
    }

    fs::create_dir_all(cache_path.join(".lib/lulib"))?;
    fs::create_dir_all(cache_path.join(".lib/dylib"))?;

    let output = Command::new(std::env::current_exe()?)
      .current_dir(cache_path)
      .args(&["build", "."])
      .output()
      .context("Failed to build package")?;

    if !output.status.success() {
      eprintln!("Build output: {}", String::from_utf8_lossy(&output.stdout));
      eprintln!("Build errors: {}", String::from_utf8_lossy(&output.stderr));
      return Err(anyhow!("Package build failed"));
    }

    Ok(())
  }

  async fn copy_package_artifacts(
    &self,
    cache_path: &Path,
    project_path: &Path,
    _package_info: &PackageInfo,
  ) -> Result<()> {
    let project_lib_dir = project_path.join(".lib");
    let project_lulib_dir = project_lib_dir.join("lulib");
    let project_dylib_dir = project_lib_dir.join("dylib");

    fs::create_dir_all(&project_lulib_dir)?;
    fs::create_dir_all(&project_dylib_dir)?;

    let cache_lulib_dir = cache_path.join(".lib");
    if cache_lulib_dir.exists() {
      for entry in fs::read_dir(&cache_lulib_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
          && entry.path().extension().and_then(|s| s.to_str()) == Some("lulib")
        {
          let dest_path = project_lulib_dir.join(entry.file_name());

          if !dest_path.exists() {
            fs::copy(&entry.path(), &dest_path)?;
          }
        }
      }
    }

    let current_platform = self.get_current_platform();
    let dylib_extension = self.get_dylib_extension();

    let cache_dylib_dir = cache_path.join(".lib/dylib");
    if cache_dylib_dir.exists() {
      for entry in fs::read_dir(&cache_dylib_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
          let file_name = entry.file_name().to_string_lossy().to_string();

          if file_name.ends_with(&format!(".{}", dylib_extension)) {
            let dest_path = project_dylib_dir.join(entry.file_name());

            if !dest_path.exists() {
              fs::copy(&entry.path(), &dest_path)?;
            }
          }
        }
      }
    }

    let platform_dylib_dir = cache_path.join(&current_platform);
    if platform_dylib_dir.exists() {
      for entry in fs::read_dir(&platform_dylib_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
          let file_name = entry.file_name().to_string_lossy().to_string();

          if file_name.ends_with(&format!(".{}", dylib_extension)) {
            let dest_path = project_dylib_dir.join(entry.file_name());

            if !dest_path.exists() {
              fs::copy(&entry.path(), &dest_path)?;
            }
          }
        }
      }
    }

    Ok(())
  }

  fn get_current_platform(&self) -> &'static str {
    if cfg!(target_os = "linux") {
      "linux"
    } else if cfg!(target_os = "windows") {
      "windows"
    } else if cfg!(target_os = "macos") {
      "macos"
    } else {
      "unknown"
    }
  }

  fn get_dylib_extension(&self) -> &'static str {
    crate::resolver::current_platform()
  }

  pub async fn install_packages(
    &self,
    urls: &[String],
    project_path: &Path,
  ) -> Result<Vec<PackageInfo>> {
    let mut installed_packages = Vec::new();

    for url in urls {
      match self.install_package(url, project_path).await {
        Ok(package_info) => {
          installed_packages.push(package_info);
        }
        Err(e) => {
          eprintln!("Failed to install package '{}': {}", url, e);
        }
      }
    }

    Ok(installed_packages)
  }

  pub fn clear_cache(&self) -> Result<()> {
    if self.cache_dir.exists() {
      fs::remove_dir_all(&self.cache_dir)?;
    }
    fs::create_dir_all(&self.cache_dir)?;
    Ok(())
  }

  pub fn clear_package_cache(&self, url: &str) -> Result<()> {
    let cache_path = self.get_package_cache_path(url);
    if cache_path.exists() {
      fs::remove_dir_all(&cache_path)?;
    }
    Ok(())
  }

  pub fn list_cached_packages(&self) -> Result<Vec<String>> {
    let mut packages = Vec::new();

    if !self.cache_dir.exists() {
      return Ok(packages);
    }

    for entry in fs::read_dir(&self.cache_dir)? {
      let entry = entry?;
      if entry.file_type()?.is_dir() {
        let conf_path = entry.path().join("lulu.conf.lua");
        if conf_path.exists() {
          if let Ok(package_info) = self.get_package_info(&entry.path(), "unknown") {
            packages.push(format!(
              "{} ({})",
              package_info.name,
              package_info.version.unwrap_or("unknown".to_string())
            ));
          }
        }
      }
    }

    Ok(packages)
  }
}
