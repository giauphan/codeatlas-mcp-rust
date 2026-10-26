use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "codeatlas")]
#[command(
    version,
    about = "High-performance Rust MCP server for CodeAtlas"
)]
pub struct Cli {
    #[arg(long = "projectDir", global = true)]
    pub project_dir: Option<PathBuf>,

    #[arg(long = "sync-dreams", global = true)]
    pub sync_dreams: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Doctor,
    Init,
    Setup,
    SetupZed {
        #[arg(long)]
        force: bool,
    },
    SetupClaude {
        #[arg(long)]
        force: bool,
    },
    RouteTask {
        task: String,
    },
    BrainContext {
        task: String,
    },
    BrainSave {
        content: String,
        #[arg(short, long, default_value = "knowledge")]
        category: String,
    },
}
