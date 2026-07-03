use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

/// Application error type.
///
/// Carries a machine-readable error code, a user-facing message,
/// optional debug detail, and an optional inner error source.
#[derive(Debug, Serialize, Deserialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub detail: Option<String>,
    pub recoverable: bool,
    #[serde(skip)]
    pub source: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            detail: None,
            recoverable: false,
            source: None,
        }
    }

    pub fn with_detail(
        code: impl Into<String>,
        message: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            detail: Some(detail.into()),
            recoverable: false,
            source: None,
        }
    }

    #[cfg(test)]
    pub fn with_source(mut self, err: impl Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(err));
        self
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|b| b.as_ref() as &(dyn Error + 'static))
    }
}

// Manual Clone: skip the `source` field (not cloneable).
impl Clone for AppError {
    fn clone(&self) -> Self {
        Self {
            code: self.code.clone(),
            message: self.message.clone(),
            detail: self.detail.clone(),
            recoverable: self.recoverable,
            source: None,
        }
    }
}

// Manual PartialEq: skip the `source` field.
impl PartialEq for AppError {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
            && self.message == other.message
            && self.detail == other.detail
            && self.recoverable == other.recoverable
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn app_error_display() {
        let err = AppError::new("TEST_CODE", "test message");
        assert_eq!(format!("{}", err), "[TEST_CODE] test message");
    }

    #[test]
    fn app_error_with_detail() {
        let err = AppError::with_detail("CODE", "msg", "detail");
        assert_eq!(err.detail, Some("detail".to_string()));
    }

    #[test]
    fn source_error_is_attached() {
        let err = AppError::new("TEST", "message").with_source(io::Error::other("boom"));
        assert!(err.source().is_some());
    }
}
