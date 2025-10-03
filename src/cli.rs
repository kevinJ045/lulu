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
  Build {
    #[arg(name = "FILE")]
    path: PathBuf,
  },
  Resolve {
    #[arg(name = "FILE")]
    item: String,
  },
}