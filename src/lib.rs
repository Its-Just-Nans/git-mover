pub(crate) mod cli;
pub(crate) mod config;
pub(crate) mod errors;
pub(crate) mod macros;
pub(crate) mod sync;
pub(crate) mod utils;
pub(crate) use macros::config_value;
pub(crate) use macros::config_value_wrap;

mod codeberg;
mod github;
mod gitlab;

pub use cli::cli_main;
