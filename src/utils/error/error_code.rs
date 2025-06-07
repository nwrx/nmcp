use axum::http::StatusCode;
use std::fmt::{Display, Formatter};

/// Append a `StatusCode` to the error report so that it can be used in the response body.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ErrorCode(pub StatusCode);

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let status_code = self.0.as_u16();
        let status_message = self.0.canonical_reason().unwrap_or_default().to_string();
        write!(f, "{status_code} → {status_message}")
    }
}

impl Default for ErrorCode {
    fn default() -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<StatusCode> for ErrorCode {
    fn from(status: StatusCode) -> Self {
        Self(status)
    }
}

impl From<u16> for ErrorCode {
    fn from(status: u16) -> Self {
        Self(StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
    }
}

impl ErrorCode {
    pub fn into_status_code(&self) -> u16 {
        self.0.as_u16()
    }

    pub fn into_status_message(&self) -> String {
        self.0.canonical_reason().unwrap_or_default().to_string()
    }
}
