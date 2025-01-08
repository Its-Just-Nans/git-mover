//! GitHub API module.
pub(crate) mod config;
pub(crate) mod platform;
pub(crate) mod repo;

/// GitHub URL
const GITHUB_URL: &str = "github.com";

/// GitHub API URL
const GITHUB_API_URL: &str = "api.github.com";

/// GitHub API Header
const GITHUB_API_HEADER: &str = "X-GitHub-Api-Version";

/// GitHub API Version
const GITHUB_API_VERSION: &str = "2022-11-28";
