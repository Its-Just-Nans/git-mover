use crate::{config::Config, config_value_wrap};
use platform::CodebergPlatform;
use serde::{Deserialize, Serialize};

pub mod platform;

const CODEBERG_URL: &str = "codeberg.org";

/// Codeberg configuration
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct CodebergConfig {
    pub username: Option<String>,

    pub token: Option<String>,
}

impl CodebergConfig {
    pub fn get_plateform(config: &mut Config) -> CodebergPlatform {
        let username = config_value_wrap!(
            config,
            codeberg,
            CodebergConfig,
            username,
            "your codeberg username"
        );
        let token = config_value_wrap!(
            config,
            codeberg,
            CodebergConfig,
            token,
            "your codeberg token (https://codeberg.org/user/settings/applications)"
        );
        CodebergPlatform::new(username, token)
    }
}
