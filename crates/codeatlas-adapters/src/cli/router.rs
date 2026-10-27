use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "codeatlas")]
#[command(
    version,
    about = "High-performance Rust MCP server for CodeAtlas",
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(long = "projectDir", global = true)]
    pub project_dir: Option<PathBuf>,

    #[arg(long = "sync-dreams", global = true)]
    pub sync_dreams: bool,

    #[arg(
        short = 'v',
        short_alias = 'V',
        long = "version",
        action = clap::ArgAction::Version,
        help = "Print version"
    )]
    pub version: Option<bool>,

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
    /// Check for updates and upgrade CodeAtlas to the latest release
    Update {
        /// Only check for updates without installing
        #[arg(long)]
        check: bool,
        /// Force reinstallation even if already at latest version
        #[arg(long, short)]
        force: bool,
    },
    /// Hook compatibility fallback for legacy IDE hooks (outputs `{}` and exits 0)
    Hook {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_command() {
        let res = Cli::try_parse_from(["codeatlas", "update", "--check"]).unwrap();
        match res.command {
            Some(Commands::Update { check, force }) => {
                assert!(check);
                assert!(!force);
            }
            _ => panic!("Expected Commands::Update"),
        }
    }

    #[test]
    fn test_hook_command() {
        let res = Cli::try_parse_from(["codeatlas", "hook", "task-router"]).unwrap();
        match res.command {
            Some(Commands::Hook { args }) => {
                assert_eq!(args, vec!["task-router"]);
            }
            _ => panic!("Expected Commands::Hook"),
        }
    }

    #[test]
    fn test_version_flags() {
        let res = Cli::try_parse_from(["codeatlas", "-v"]);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().kind(),
            clap::error::ErrorKind::DisplayVersion
        );

        let res = Cli::try_parse_from(["codeatlas", "-V"]);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().kind(),
            clap::error::ErrorKind::DisplayVersion
        );

        let res = Cli::try_parse_from(["codeatlas", "--version"]);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().kind(),
            clap::error::ErrorKind::DisplayVersion
        );
    }
}
