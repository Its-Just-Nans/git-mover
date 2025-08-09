//! Error handling for the git-mover crate.
use std::{error::Error as StdError, fmt};

use crate::platform::PlatformType;

/// Error type for the git-mover crate.
#[derive(Debug)]
pub struct GitMoverError {
    /// Inner error.
    inner: Box<Inner>,
}

impl GitMoverError {
    /// Create a new error.
    pub(crate) fn new(kind: GitMoverErrorKind) -> Self {
        Self {
            inner: Box::new(Inner {
                kind,
                source: None,
                platform: None,
            }),
        }
    }

    /// Create a new error with a source.
    pub(crate) fn with_text(mut self, text: &str) -> Self {
        self.inner.source = Some(Box::new(std::io::Error::other(
            text,
        )));
        self
    }

    /// Create a new error with a platform.
    pub(crate) fn with_platform(mut self, platform: PlatformType) -> Self {
        self.inner.platform = Some(platform);
        self
    }
}

/// Type alias for a boxed error.
pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;

/// Inner error type for the git-mover crate.
#[derive(Debug)]
struct Inner {
    /// Error kind.
    kind: GitMoverErrorKind,

    /// Platform error
    platform: Option<PlatformType>,

    /// Source error.
    source: Option<BoxError>,
}

#[derive(Debug)]
pub(crate) enum GitMoverErrorKind {
    /// Error related to the platform.
    Platform,

    /// Error related to the reqwest crate.
    Reqwest,

    /// Error related to serde.
    Serde,

    /// Error related to Git2.
    Git2,

    /// Error related to the RepoEdition func.
    RepoEdition,

    /// Error related to the RepoCreation func.
    RepoCreation,

    /// Error related to the GetAllRepo func.
    GetAllRepos,

    /// Error related to the GetRepo func.
    GetRepo,

    /// Error related to the RepoCreation func.
    RepoNotFound,

    /// Error related to the RepoDeletion func.
    RepoDeletion,
}

impl fmt::Display for GitMoverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner.kind)
    }
}

impl StdError for GitMoverError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.inner.source.as_ref().map(|e| &**e as _)
    }
}

impl From<reqwest::Error> for GitMoverError {
    fn from(e: reqwest::Error) -> Self {
        Self {
            inner: Box::new(Inner {
                kind: GitMoverErrorKind::Reqwest,
                source: Some(Box::new(e)),
                platform: None,
            }),
        }
    }
}

impl From<serde_json::Error> for GitMoverError {
    fn from(e: serde_json::Error) -> Self {
        Self {
            inner: Box::new(Inner {
                kind: GitMoverErrorKind::Serde,
                source: Some(Box::new(e)),
                platform: None,
            }),
        }
    }
}

impl From<std::io::Error> for GitMoverError {
    fn from(e: std::io::Error) -> Self {
        Self {
            inner: Box::new(Inner {
                kind: GitMoverErrorKind::Platform,
                source: Some(Box::new(e)),
                platform: None,
            }),
        }
    }
}

impl From<git2::Error> for GitMoverError {
    fn from(e: git2::Error) -> Self {
        Self {
            inner: Box::new(Inner {
                kind: GitMoverErrorKind::Git2,
                source: Some(Box::new(e)),
                platform: None,
            }),
        }
    }
}
