//! # git-mover
//!
//! Move git repositories to a new location
//!
//! ## Usage
//!
//! ```txt
//! Usage: git-mover [OPTIONS]
//! Options:
//! -s, --source <SOURCE>            The source platform (github, gitlab, codeberg) [aliases: from]
//! -d, --destination <DESTINATION>  The destination platform (github, gitlab, codeberg) [aliases: to]
//! -n, --no-forks                   Don't sync forked repositories
//! -c, --config <CONFIG>            Custom configuration file
//! -v, --verbose...                 Verbose mode (-v, -vv, -vvv)
//! -h, --help                       Print help
//! ```

pub(crate) mod cli;
pub(crate) mod config;
pub(crate) mod errors;
pub(crate) mod macros;
pub(crate) mod platform;
pub(crate) mod sync;
pub(crate) mod utils;
pub(crate) use macros::config_value;
pub(crate) use macros::config_value_wrap;

mod codeberg;
mod github;
mod gitlab;

pub use cli::cli_main;
pub use config::Config;
pub use utils::main_sync;
