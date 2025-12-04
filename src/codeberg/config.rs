//! Codeberg configuration
use super::platform::CodebergPlatform;
use crate::{
    config::GitMoverConfig, config_password_wrap, config_value_wrap, errors::GitMoverError,
};
use serde::{Deserialize, Serialize};

/// Codeberg configuration
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct CodebergConfig {
    /// Codeberg username
    pub username: Option<String>,

    /// Codeberg token
    pub token: Option<String>,
}

impl CodebergConfig {
    /// Get the codeberg platform
    pub fn get_plateform(config: &mut GitMoverConfig) -> Result<CodebergPlatform, GitMoverError> {
        let username = config_value_wrap!(
            config,
            codeberg,
            CodebergConfig,
            username,
            "your codeberg username"
        );
        let token = config_password_wrap!(
            config,
            codeberg,
            CodebergConfig,
            token,
            "your codeberg token (https://codeberg.org/user/settings/applications)"
        );
        Ok(CodebergPlatform::new(username, token))
    }
}
