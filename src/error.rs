#[derive(Debug, thiserror::Error)]
#[error("{kind}")]
pub struct Error {
    kind: ErrorKind,
    source: Option<anyhow::Error>,
}

#[derive(Clone, Copy, Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("authentication error")]
    Auth,
    #[error("received unexpected status code: {0}")]
    Status(hyper::StatusCode),
    #[error("verify error")]
    Verify { code_mismatch: bool },
}

impl ErrorKind {
    pub(crate) fn is_code_mismatch(self) -> bool {
        match self {
            ErrorKind::Verify { code_mismatch } => code_mismatch,
            _ => false,
        }
    }
}

impl Error {
    pub(crate) fn new_auth(src: impl Into<anyhow::Error>) -> Self {
        Error::with_cause(ErrorKind::Auth, src)
    }

    pub(crate) fn new_verify(src: impl Into<anyhow::Error>) -> Self {
        Error::with_cause(
            ErrorKind::Verify {
                code_mismatch: false,
            },
            src,
        )
    }

    pub(crate) fn new_code_mismatch(src: impl Into<anyhow::Error>) -> Self {
        Error::with_cause(
            ErrorKind::Verify {
                code_mismatch: true,
            },
            src,
        )
    }

    pub(crate) fn with_cause(kind: ErrorKind, src: impl Into<anyhow::Error>) -> Self {
        Error {
            kind,
            source: Some(src.into()),
        }
    }

    #[inline]
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl From<hyper::StatusCode> for Error {
    fn from(code: hyper::StatusCode) -> Self {
        Error {
            kind: ErrorKind::Status(code),
            source: None,
        }
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        Error::with_cause(ErrorKind::Auth, e)
    }
}
