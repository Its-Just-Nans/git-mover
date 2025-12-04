//! Github configuration
use super::platform::GithubPlatform;
use serde::{Deserialize, Serialize};

use crate::{
    config::GitMoverConfig, config_password_wrap, config_value_wrap, errors::GitMoverError,
};

/// Github configuration
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct GithubConfig {
    /// Github username
    pub username: Option<String>,

    /// Github token
    pub token: Option<String>,
}

impl GithubConfig {
    /// Get the github platform
    pub fn get_plateform(config: &mut GitMoverConfig) -> Result<GithubPlatform, GitMoverError> {
        let username = config_value_wrap!(
            config,
            github,
            GithubConfig,
            username,
            "your github username"
        );
        let token = config_password_wrap!(
            config,
            github,
            GithubConfig,
            token,
            "your github token (https://github.com/settings/personal-access-tokens)"
        );
        Ok(GithubPlatform::new(username, token))
    }
}
