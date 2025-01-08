//! Codeberg configuration
use super::platform::CodebergPlatform;
use crate::{config::Config, config_password_wrap, config_value_wrap};
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
    pub fn get_plateform(config: &mut Config) -> CodebergPlatform {
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
        CodebergPlatform::new(username, token)
    }
}
