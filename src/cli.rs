//! Command line options for the git-mover tool
use crate::{config::Config, platform::PlatformType, utils::main_sync};
use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

/// Command line options for the git-mover tool
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

    /// Resync all repositories
    #[arg(short, long)]
    pub resync: bool,

    /// Custom configuration file
    #[arg(short, long)]
    pub config: Option<String>,

    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

/// Run the git-mover tool with the provided command line options
pub async fn cli_main() {
    let args = GitMoverCli::parse();
    let mut config = match &args.config {
        Some(path_str) => {
            let path = PathBuf::from(path_str);
            Config::new_from_path(&path)
        }
        None => Config::new(),
    }
    .set_debug(args.verbose)
    .with_cli_args(args);
    main_sync(&mut config).await;
}
