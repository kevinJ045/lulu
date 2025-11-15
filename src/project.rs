use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const GITIGNORE_TEMPLATE: &str = r#"
**/target/
**/**.lulib
.lib/
**/.lib/
"#;

const MAIN_CODE: &str = r#"print("Hii!")"#;

fn yesify(value: bool) -> String {
  if value {
    "yes".to_string()
  } else {
    "no".to_string()
  }
}

fn is_yes(input: &str) -> bool {
  input.trim().to_lowercase().starts_with('y')
}

fn prompt_user(message: &str) -> bool {
  print!("{} ", message);
  io::stdout().flush().unwrap();

  let mut input = String::new();
  match io::stdin().read_line(&mut input) {
    Ok(_) => is_yes(&input),
    Err(_) => false,
  }
}

fn optionify(logs: &[&str], ignore: bool, current_value: bool) -> bool {
  if ignore {
    println!("{} {}", logs.join(" "), yesify(current_value));
    current_value
  } else {
    let prompt = format!("{} ", logs.join(" "));
    prompt_user(&prompt)
  }
}

fn create_lulu_conf(
  path: &Path,
  app_name: &str,
  is_lib: bool,
) -> Result<(), Box<dyn std::error::Error>> {
  let main_file = if is_lib {
    "init.lua"
  } else {
    "main.lua"
  };

  let mut content = format!(
    "manifest = {{\n  name = \"{}\",\n  version = \"0.1.0\"\n}}\n\nmods = {{\n  {} = \"{}\"\n}}\n",
    app_name, if is_lib { "init" } else { "main" }, main_file
  );

  if is_lib {
    content.push_str("\nfetch = \"code\"\n");
  }

  content.push_str(
    format!("\nbuild = function()\n  resolve_dependencies()\n  bundle_main(\"{}\", {})\n  print('Built binary to \".lib\" folder.')\nend\n", main_file, is_lib).as_str()
  );

  let app_yaml_path = path.join("lulu.conf.lua");
  fs::write(&app_yaml_path, content)?;

  println!("Created file {}", "lulu.conf.lua");
  Ok(())
}

fn create_main_file(path: &Path, is_lib: bool) -> Result<(), Box<dyn std::error::Error>> {
  let (filename, content) = if is_lib {
    ("init.lua", MAIN_CODE)
  } else {
    ("main.lua", MAIN_CODE)
  };

  let main_path = path.join(filename);
  fs::write(&main_path, content)?;

  println!("Created file {}", filename);
  Ok(())
}

fn create_git_files(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
  let output = Command::new("git")
    .args(&["init", "."])
    .current_dir(path)
    .output();

  match output {
    Ok(result) => {
      if !result.status.success() {
        println!(
          "Git init failed: {}",
          String::from_utf8_lossy(&result.stderr)
        );
      }
    }
    Err(e) => {
      println!("Failed to run git init: {}", e);
    }
  }

  let gitignore_path = path.join(".gitignore");
  fs::write(&gitignore_path, GITIGNORE_TEMPLATE)?;
  println!("Created file {}", ".gitignore");

  Ok(())
}

pub fn new(path: String, git: bool, ignore: bool, lib: bool) {
  let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
  let new_path = if path == "." || path.is_empty() {
    current_dir.clone()
  } else {
    current_dir.join(path.clone())
  };

  if new_path.exists() {
    match fs::read_dir(&new_path) {
      Ok(entries) => {
        if entries.count() > 0 {
          println!("Cannot overwrite a populated directory");
          return;
        }
      }
      Err(e) => {
        println!("Failed to read directory: {}", e);
        return;
      }
    }
  }

  let display_path = if path == "true" || path.is_empty() {
    ".".to_string()
  } else {
    path.clone()
  };

  println!("Creating at {}", display_path);

  let app_name = new_path
    .file_name()
    .and_then(|name| name.to_str())
    .unwrap_or("app")
    .to_string();

  println!("package: {}", app_name);

  let use_git = optionify(&["use git?"], ignore, git);

  let is_lib = optionify(&["is this a lulu library?"], ignore, lib);

  println!("Creating files");

  if let Err(e) = fs::create_dir_all(&new_path) {
    println!("Failed to create directory: {}", e);
    return;
  }

  if let Err(e) = fs::create_dir_all(&new_path.join(".lib")) {
    println!("Failed to create directory: {}", e);
    return;
  }

  if let Err(e) = create_lulu_conf(&new_path, &app_name, is_lib) {
    println!("Failed to create app.yaml: {}", e);
    return;
  }

  if let Err(e) = create_main_file(&new_path, is_lib) {
    println!("Failed to create main file: {}", e);
    return;
  }

  if use_git {
    if let Err(e) = create_git_files(&new_path) {
      println!("Failed to create git files: {}", e);
      return;
    }
  }

  println!("Project '{}' created successfully!", app_name);
}
