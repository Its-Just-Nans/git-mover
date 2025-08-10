//! Command line options for the git-mover tool
use crate::{config::Config, platform::PlatformType, utils::main_sync};
use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

/// git-mover - Move git repositories to a new location
#[derive(Parser, Deserialize, Default, Clone, Debug)]
pub struct GitMoverCli {
    /// The source platform (github, gitlab, codeberg)
    #[arg(short, long, visible_alias = "from")]
    pub source: Option<PlatformType>,

    /// The destination platform (github, gitlab, codeberg)
    #[arg(short, long, visible_alias = "to")]
    pub destination: Option<PlatformType>,

    /// Don't sync forked repositories
    #[arg(short, long = "no-forks")]
    pub no_forks: bool,

    /// Don't delete repositories
    #[arg(long = "no-delete")]
    pub no_delete: bool,

    /// Resync all repositories
    #[arg(short, long)]
    pub resync: bool,

    /// Custom configuration file path
    #[arg(short, long)]
    pub config: Option<String>,

    /// Show the current config path
    #[arg(long)]
    pub show_config_path: bool,

    /// Sync manually
    #[arg(short, long)]
    pub manual: bool,

    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

/// Run the git-mover tool with the provided command line options
pub async fn cli_main() {
    let args = GitMoverCli::parse();
    let config_path = args.config.clone();
    let mut config = match config_path {
        Some(path_str) => {
            let path = PathBuf::from(path_str);
            Config::new_from_path(args, &path)
        }
        None => Config::new(args),
    };
    if config.cli_args.show_config_path {
        println!("{}", config.config_path.display());
        return;
    }
    main_sync(&mut config).await;
}
