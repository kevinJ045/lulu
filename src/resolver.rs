use anyhow::Result;
use regex::Regex;
use std::path::Path;

#[derive(Debug)]
pub struct GitHubDependency {
  pub username: String,
  pub repo: String,
  pub path: Option<String>,
  pub branch: Option<String>,
  pub commit: Option<String>,
}

impl GitHubDependency {
  pub fn raw_url(&self, file_path: &str) -> String {
    let revision = self
      .commit
      .as_deref()
      .or(self.branch.as_deref())
      .unwrap_or("main");
    let path = self.path.as_deref().unwrap_or("");
    let full_path = if path.is_empty() {
      file_path.to_string()
    } else {
      format!("{}/{}", path, file_path)
    };

    format!(
      "https://raw.githubusercontent.com/{}/{}/{}/{}",
      self.username, self.repo, revision, full_path
    )
  }
}

pub fn parse_github_dep(s: &str) -> Option<GitHubDependency> {
  let re = Regex::new(r"^github:(?P<username>[^/]+)/(?P<repo>[^/@#]+)(?P<path>(?:/[^@#]+)*)?(?:@(?P<branch>[^#]+))?(?:#(?P<commit>.+))?$").unwrap();

  let caps = re.captures(s)?;

  Some(GitHubDependency {
    username: caps.name("username")?.as_str().to_string(),
    repo: caps.name("repo")?.as_str().to_string(),
    path: caps.name("path").and_then(|m| {
      let p = m.as_str();
      if p.is_empty() {
        None
      } else {
        Some(p[1..].to_string())
      }
    }),
    branch: caps.name("branch").map(|m| m.as_str().to_string()),
    commit: caps.name("commit").map(|m| m.as_str().to_string()),
  })
}

// fn current_platform() -> &'static str {
//   if cfg!(target_os = "linux") {
//     "linux"
//   } else if cfg!(target_os = "windows") {
//     "windows"
//   } else if cfg!(target_os = "macos") {
//     "macos"
//   } else {
//     "unknown"
//   }
// }

// async fn download_file(url: &str, dest: &Path) -> Result<()> {
//   let response = reqwest::get(url).await?.error_for_status()?;
//   let bytes = response.bytes().await?;
//   fs::create_dir_all(dest.parent().unwrap())?;
//   fs::write(dest, &bytes)?;
//   Ok(())
// }

// async fn download_lulib(url: &str, lib_folder: &Path) -> Result<()> {
//   create_dirs(lib_folder)?;

//   let pathname = PathBuf::from(
//     url
//       .to_string()
//       .replace("http://", "")
//       .replace("https://", ""),
//   );
//   let name = pathname.file_stem().and_then(|s| s.to_str()).unwrap();
//   let lulib_path = lib_folder.join(format!(".lib/lulib/{}.lulib", name));
//   download_file(url, &lulib_path).await?;

//   let platform = current_platform();
//   let lib_ext = match platform {
//     "linux" => "so",
//     "windows" => "dll",
//     "macos" => "dylib",
//     _ => return Ok(()),
//   };
//   let platform_lib_url = format!(".lib/lulib/{}-{}.{}", name, platform, lib_ext);
//   let platform_dir = lib_folder.join(platform);
//   let platform_lib_path = platform_dir.join(format!("{}-{}.{}", name, platform, lib_ext));
//   let _ = download_file(&platform_lib_url, &platform_lib_path);

//   Ok(())
// }

// async fn extract_archive(url: &str, dest: &Path) -> Result<()> {
//   let tmp_path = dest.join("tmp_download");
//   download_file(url, &tmp_path).await?;

//   if url.ends_with(".zip") {
//     let file = File::open(&tmp_path)?;
//     let mut archive = zip::ZipArchive::new(file)?;
//     archive.extract(dest)?;
//   } else if url.ends_with(".tar.gz") || url.ends_with(".tgz") {

//   } else {
//     return Err(anyhow!("Archive format not supported yet"));
//   }

//   fs::remove_file(tmp_path)?;
//   Ok(())
// }

pub fn create_dirs(dest: &Path) -> Result<()> {
  std::fs::create_dir_all(dest.join(".lib/lulib"))?;
  std::fs::create_dir_all(dest.join(".lib/dylib"))?;
  Ok(())
}

// pub async fn check_github_repo(dep: GitHubDependency, dest: &Path) -> Result<Option<PathBuf>> {
//   let lua_conf_url = dep.raw_url("lulu.conf.lua");

//   let res = reqwest::get(lua_conf_url)
//     .await?
//     .error_for_status()?
//     .text()
//     .await?;

//   let fetch = crate::conf::load_lulu_fetch_field(&mlua::Lua::new(), res)?;

//   if let Some(fetch) = fetch {
//     create_dirs(dest)?;
//     process_fetch(fetch, dest).await?;
//   }

//   Ok(None)
// }

// async fn process_fetch(fetch: FetchField, lib_folder: &Path) -> Result<()> {
//   match fetch {
//     FetchField::Code => Ok(()),
//     FetchField::Lulib { url, include } => {
//       download_lulib(&url, lib_folder).await?;
//       if let Some(include) = include {
//         for (platform, files) in include {
//           if platform == current_platform() {
//             let platform_dir = lib_folder.join(platform);
//             for file_url in files {
//               let filename = Path::new(&file_url)
//                 .file_name()
//                 .unwrap()
//                 .to_string_lossy()
//                 .to_string();
//               let dest = platform_dir.join(filename);
//               let _ = download_file(&file_url, &dest);
//             }
//           }
//         }
//       }
//       Ok(())
//     }
//   }
// }

// pub async fn fetch_dependency(dep: &str, lib_folder: &PathBuf) -> Result<()> {
//   if dep.starts_with("github:") {
//     if let Some(github_dep) = parse_github_dep(dep) {
//       check_github_repo(github_dep, lib_folder).await?;
//     } else {
//       eprintln!("URLError: Couldn't resolve \"{}\".", dep);
//     }
//   } else if dep.starts_with("http") {
//     if dep.ends_with(".lulib") {
//       download_lulib(dep, lib_folder).await?;
//     } else if dep.ends_with(".zip") || dep.ends_with(".tar.gz") {
//       // extract_archive(dep, lib_folder).await?;
//     }
//   } else {
//   }

//   Ok(())
// }
