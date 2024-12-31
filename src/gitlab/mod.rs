use platform::GitlabPlatform;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::config_value_wrap;

pub mod platform;

const GITLAB_URL: &str = "gitlab.com";

/// Gitlab configuration
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct GitlabConfig {
    pub username: Option<String>,

    pub token: Option<String>,
}

impl GitlabConfig {
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
