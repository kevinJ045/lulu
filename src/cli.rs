use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rew")]
#[command(version = env!("CARGO_PKG_VERSION"),)]
#[command(about = "A Rust-based Rew runtime using deno_core")]
pub struct Cli {
  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
  Run {
    #[arg(name = "FILE")]
    file: PathBuf,

    #[arg(trailing_var_arg = true)]
    args: Vec<String>,
  },
  Test {
    #[arg(name = "FILE")]
    file: PathBuf,

    #[arg(short = 't', long)]
    test: Option<String>,

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
  },
  New {
    #[arg(name = "NAME")]
    name: String,

    #[arg(short = 'g', long)]
    git: bool,

    #[arg(short = 't', long)]
    lib: bool,

    #[arg(short = 'i', long)]
    ignore: bool,
  },
  Build {
    #[arg(name = "PATH", default_value = ".")]
    path: PathBuf,
  },
  Resolve {
    #[arg(name = "URL", default_value = ".")]
    item: String,
  },
  Update {
    #[arg(name = "PACKAGES")]
    packages: Vec<String>,

    #[arg(short, long, default_value = ".")]
    project: PathBuf,
  },
  Cache {
    #[command(subcommand)]
    cache_command: CacheCommand,
  },
}

#[derive(Subcommand)]
pub enum CacheCommand {
  Clear,
  List,
  Remove {
    #[arg(name = "PACKAGE_URL")]
    package_url: String,
  },
}
