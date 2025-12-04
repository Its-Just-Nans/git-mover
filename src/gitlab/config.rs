//! Gitlab configuration
use super::platform::GitlabPlatform;
use crate::config_value_wrap;
use crate::errors::GitMoverError;
use crate::{config::GitMoverConfig, config_password_wrap};
use serde::{Deserialize, Serialize};

/// Gitlab configuration
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct GitlabConfig {
    /// Gitlab username
    pub username: Option<String>,

    /// Gitlab token
    pub token: Option<String>,

    /// Custom Gitlab url
    pub custom_url: Option<String>,
}

impl GitlabConfig {
    /// Get Gitlab platform
    pub fn get_plateform(config: &mut GitMoverConfig) -> Result<GitlabPlatform, GitMoverError> {
        let username = config_value_wrap!(
            config,
            gitlab,
            GitlabConfig,
            username,
            "your gitlab username"
        );
        let token = config_password_wrap!(
            config,
            gitlab,
            GitlabConfig,
            token,
            "your gitlab token (https://gitlab.com/-/user_settings/personal_access_tokens)"
        );
        let custom_url = config_password_wrap!(
            config,
            gitlab,
            GitlabConfig,
            custom_url,
            "custom gitlab url - empty to get the default Gitlab url"
        );
        let cust_url = if custom_url.is_empty() {
            None
        } else {
            Some(custom_url)
        };
        Ok(GitlabPlatform::new(username, token, cust_url))
    }
}
