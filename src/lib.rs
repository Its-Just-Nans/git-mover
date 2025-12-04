//! # git-mover
//!
//! Move git repositories to a new location
//!
//! ## Usage
//!
//! ```txt
//! Usage: git-mover [OPTIONS]
//!
//! Options:
//!  -s, --source <SOURCE>            The source platform (github, gitlab, codeberg) [aliases: from]
//!  -d, --destination <DESTINATION>  The destination platform (github, gitlab, codeberg) [aliases: to]
//!  -n, --no-forks                   Don't sync forked repositories
//!  -r, --resync                     Resync all repositories
//!  -c, --config <CONFIG>            Custom configuration file
//!      --show-config-path           Show the current config path
//!  -v, --verbose...                 Verbose mode (-v, -vv, -vvv)
//!  -h, --help                       Print help
//! ```

#![warn(clippy::all, rust_2018_idioms)]
#![deny(
    missing_docs,
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::expect_used
)]
#![warn(clippy::multiple_crate_versions)]

pub(crate) mod cli;
pub(crate) mod config;
pub(crate) mod errors;
pub(crate) mod macros;
pub(crate) mod platform;
pub(crate) mod sync;
pub(crate) mod utils;
pub(crate) use macros::config_password_wrap;
pub(crate) use macros::config_value_wrap;

mod codeberg;
mod github;
mod gitlab;

pub use cli::{git_mover_main, GitMoverCli};
pub use config::GitMoverConfig;
pub use utils::main_sync;
