use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub struct GitMoverError {
    inner: Box<Inner>,
}

impl GitMoverError {
    pub(crate) fn new(kind: GitMoverErrorKind) -> Self {
        Self {
            inner: Box::new(Inner { kind, source: None }),
        }
    }

    pub(crate) fn with_text(mut self, text: &str) -> Self {
        self.inner.source = Some(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            text,
        )));
        self
    }
}

pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;

#[derive(Debug)]
struct Inner {
    kind: GitMoverErrorKind,
    source: Option<BoxError>,
}

#[derive(Debug)]
pub(crate) enum GitMoverErrorKind {
    Platform,
    Reqwest,
    Serde,
    Unimplemented,
    RepoEdition,
    GetAllRepos,
    GetRepo,
    RepoNotFound,
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
            }),
        }
    }
}

impl From<git2::Error> for GitMoverError {
    fn from(e: git2::Error) -> Self {
        Self {
            inner: Box::new(Inner {
                kind: GitMoverErrorKind::Platform,
                source: Some(Box::new(e)),
            }),
        }
    }
}
