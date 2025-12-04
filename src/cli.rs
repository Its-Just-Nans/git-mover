//! Command line options for the git-mover tool
use crate::{
    config::GitMoverConfig, errors::GitMoverError, platform::PlatformType, utils::main_sync,
};
use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

/// git-mover - Move git repositories to a new location
#[derive(Parser, Deserialize, Default, Clone, Debug)]
pub struct GitMoverCli {
    /// The source platform (github, gitlab, codeberg)
    #[arg(long, visible_alias = "from")]
    pub source: Option<PlatformType>,

    /// The destination platform (github, gitlab, codeberg)
    #[arg(long, visible_alias = "to")]
    pub destination: Option<PlatformType>,

    /// Don't sync forked repositories
    #[arg(long = "no-forks")]
    pub no_forks: bool,

    /// Don't delete repositories
    #[arg(long = "no-delete")]
    pub no_delete: bool,

    /// Resync all repositories
    #[arg(long)]
    pub resync: bool,

    /// Custom configuration file path
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Show the current config path and exit
    #[arg(long)]
    pub show_config_path: bool,

    /// Sync manually
    #[arg(long)]
    pub manual: bool,

    /// Verbose mode
    #[arg(short, long, visible_short_alias = 'd', action = clap::ArgAction::Count)]
    pub verbose: u8,
}

impl GitMoverCli {
    /// Run the git-mover tool with the provided command line options
    /// # Errors
    /// Errors if something happens
    pub async fn main(self) -> Result<(), GitMoverError> {
        let config = GitMoverConfig::try_new(self)?;
        if config.cli_args.show_config_path {
            println!("{}", config.config_path.display());
            return Ok(());
        }
        main_sync(config).await
    }
}

/// Main git-mover cli
/// # Errors
/// Error if cli error
pub async fn git_mover_main() -> Result<(), GitMoverError> {
    let git_mover_inst = GitMoverCli::parse();
    git_mover_inst.main().await
}
