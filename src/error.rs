//! Error types used throughout the library.

/// A list specifying general categories of Vonage API errors.
#[derive(Clone, Copy, Debug, thiserror::Error)]
pub enum ErrorKind {
    /// An authentication error occurred.
    #[error("authentication error")]
    Auth,
    /// An HTTP error occurred.
    #[error("HTTP error")]
    Http,
    /// Received an unexpected HTTP status code.
    #[error("received unexpected status code: {0}")]
    Status(hyper::StatusCode),
    /// An error occurred in the [Verify (2FA)](https://developer.nexmo.com/api/verify) API.
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

/// The error type for Vonage API operations.
///
/// It is used with the [`ErrorKind`](./enum.ErrorKind.html) enum.
#[derive(Debug, thiserror::Error)]
#[error("{kind}")]
pub struct Error {
    kind: ErrorKind,
    source: Option<anyhow::Error>,
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

    /// The underlying cause of the error.
    #[inline]
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        Error::with_cause(ErrorKind::Http, e)
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
