//! Error handling for the git-mover crate.
use std::{error::Error, fmt};
use tokio::time::error::Elapsed;

/// Error type for the git-mover crate.
#[derive(Debug)]
pub struct GitMoverError {
    /// Inner error.
    message: String,

    /// Error source
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl GitMoverError {
    /// Create a new error.
    pub(crate) fn new(message: String) -> Self {
        Self {
            message,
            source: None,
        }
    }

    /// Create a new GeneralError instance with a source
    pub fn new_with_source<S: Into<String>, B: std::error::Error + Send + Sync + 'static>(
        message: S,
        from: B,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(from)),
        }
    }
}

impl std::error::Error for GitMoverError {}

impl fmt::Display for GitMoverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.source {
            Some(e) => write!(f, "{} - {}", self.message, e),
            None => write!(f, "{}", self.message),
        }
    }
}

impl From<&str> for GitMoverError {
    fn from(e: &str) -> Self {
        Self::new(e.to_string())
    }
}

impl From<String> for GitMoverError {
    fn from(e: String) -> Self {
        Self::new(e.to_string())
    }
}

impl From<Elapsed> for GitMoverError {
    fn from(e: Elapsed) -> Self {
        Self::new_with_source(e.to_string(), e)
    }
}

impl From<std::str::Utf8Error> for GitMoverError {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::new_with_source(e.to_string(), e)
    }
}

impl From<reqwest::Error> for GitMoverError {
    fn from(e: reqwest::Error) -> Self {
        Self::new_with_source(e.to_string(), e)
    }
}

impl From<serde_json::Error> for GitMoverError {
    fn from(e: serde_json::Error) -> Self {
        Self::new_with_source(e.to_string(), e)
    }
}

impl From<std::io::Error> for GitMoverError {
    fn from(e: std::io::Error) -> Self {
        Self::new_with_source(e.to_string(), e)
    }
}

impl From<git2::Error> for GitMoverError {
    fn from(e: git2::Error) -> Self {
        Self::new_with_source(e.to_string(), e)
    }
}

impl From<toml::de::Error> for GitMoverError {
    fn from(e: toml::de::Error) -> Self {
        Self::new_with_source(e.to_string(), e)
    }
}

impl<S, B> From<(S, B)> for GitMoverError
where
    S: Into<String>,
    B: std::error::Error + Send + Sync + 'static,
{
    fn from(value: (S, B)) -> Self {
        // value.0 is the string, value.1 is the error
        Self::new_with_source(value.0.into(), Box::new(value.1))
    }
}
