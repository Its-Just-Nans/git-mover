//! Github configuration
use super::platform::GithubPlatform;
use serde::{Deserialize, Serialize};

use crate::{config::Config, config_value_wrap};

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
    pub fn get_plateform(config: &mut Config) -> GithubPlatform {
        let username = config_value_wrap!(
            config,
            github,
            GithubConfig,
            username,
            "your github username"
        );
        let token = config_value_wrap!(
            config,
            github,
            GithubConfig,
            token,
            "your github token (https://github.com/settings/personal-access-tokens)"
        );
        GithubPlatform::new(username, token)
    }
}
