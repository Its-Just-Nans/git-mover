use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;

use crate::{config::Config, utils::main_sync};

#[derive(Parser, Deserialize, Default, Clone, Debug)]
pub struct GitMoverCli {
    #[arg(short, long, visible_alias = "from")]
    pub source: Option<String>,
    #[arg(short, long, visible_alias = "to")]
    pub destination: Option<String>,
    #[arg(short, long = "no-forks")]
    pub no_forks: bool,
    #[arg(short, long)]
    pub config: Option<String>,
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

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
