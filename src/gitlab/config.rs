//! Gitlab configuration
use super::platform::GitlabPlatform;
use crate::config::Config;
use crate::config_value_wrap;
use serde::{Deserialize, Serialize};

/// Gitlab configuration
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct GitlabConfig {
    /// Gitlab username
    pub username: Option<String>,

    /// Gitlab token
    pub token: Option<String>,
}

impl GitlabConfig {
    /// Get Gitlab platform
    pub fn get_plateform(config: &mut Config) -> GitlabPlatform {
        let username = config_value_wrap!(
            config,
            gitlab,
            GitlabConfig,
            username,
            "your gitlab username"
        );
        let token = config_value_wrap!(
            config,
            gitlab,
            GitlabConfig,
            token,
            "your gitlab token (https://gitlab.com/-/user_settings/personal_access_tokens)"
        );
        GitlabPlatform::new(username, token)
    }
}
