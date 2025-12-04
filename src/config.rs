//! Configuration handling
use std::{
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    path::PathBuf,
};

use home::home_dir;
use serde::{Deserialize, Serialize};

use crate::{
    cli::GitMoverCli, codeberg::config::CodebergConfig, errors::GitMoverError,
    github::config::GithubConfig, gitlab::config::GitlabConfig,
};

/// Configuration data
#[derive(Deserialize, Default, Clone, Debug)]
pub struct GitMoverConfig {
    /// path to the configuration file
    pub config_path: PathBuf,

    /// actual configuration data
    pub config_data: ConfigData,

    /// CLI arguments
    pub cli_args: GitMoverCli,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct ConfigData {
    /// Gitlab configuration
    pub gitlab: Option<GitlabConfig>,

    /// Github configuration
    pub github: Option<GithubConfig>,

    /// Codeberg configuration
    pub codeberg: Option<CodebergConfig>,
}

impl GitMoverConfig {
    /// Create a new Config object from the default path
    /// # Errors
    /// Error if the config file can't be opened
    pub fn try_new(cli_args: GitMoverCli) -> Result<Self, GitMoverError> {
        let config_path = match cli_args.config.clone() {
            Some(p) => p,
            None => Self::get_config_path()?,
        };
        let contents = read_to_string(config_path.clone())
            .map_err(|e| GitMoverError::new_with_source("Unable to open", e))?;
        let config_data = toml::from_str(&contents)?;
        Ok(GitMoverConfig {
            config_path,
            cli_args,
            config_data,
        })
    }

    /// Save the config data to the config file
    /// # Errors
    /// Error if the config file can't be created or written to
    pub fn save(&self) -> Result<(), GitMoverError> {
        let config_str = toml::to_string(&self.config_data)
            .map_err(|e| GitMoverError::new_with_source("Unable to serialize config", e))?;
        let mut file = File::create(&self.config_path)
            .map_err(|e| GitMoverError::new_with_source("Unable to create config file", e))?;
        file.write_all(config_str.as_bytes())
            .map_err(|e| GitMoverError::new_with_source("Unable to write to config file", e))
    }

    /// Get the path to the config file
    /// # Errors
    /// Error if the home directory can't be found
    pub fn get_config_path() -> Result<PathBuf, GitMoverError> {
        let home_dir = match home_dir() {
            Some(path) if !path.as_os_str().is_empty() => path,
            _ => return Err("Unable to get your home dir! home::home_dir() isn't working".into()),
        };
        let config_directory = home_dir.join(".config").join(".git-mover");
        let config_path = config_directory.join("config.toml");
        create_dir_all(config_directory)
            .map_err(|e| GitMoverError::new_with_source("Unable to create config dir", e))?;
        if !config_path.exists() {
            let mut file = File::create(&config_path)
                .map_err(|e| GitMoverError::new_with_source("Unable to create config file", e))?;
            file.write_all(b"")
                .map_err(|e| GitMoverError::new_with_source("Unable to write to config file", e))?;
        }
        Ok(config_path)
    }

    /// Update the config data and save it to the config file
    /// # Errors
    /// Error if fail to save config
    pub fn update(
        &mut self,
        updater_fn: impl FnOnce(&mut ConfigData),
    ) -> Result<(), GitMoverError> {
        updater_fn(&mut self.config_data);
        self.save()?;
        Ok(())
    }
}
