use serde_json::json;

use super::{Backtrace, ErrorCode, ErrorInner, ErrorMessage, ErrorName};
use core::fmt::{Debug, Display};
use std::{str::FromStr, sync::Arc};

#[derive(Debug, Clone)]
pub struct Error {
    pub source: Arc<ErrorInner>,
    pub name: Option<ErrorName>,
    pub code: Option<ErrorCode>,
    pub message: Option<ErrorMessage>,
    pub backtrace: Option<Backtrace>,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name = self.name.clone().unwrap_or_default();
        let message = self.message.clone().unwrap_or_default();
        write!(f, "[{name}] {message}")
    }
}

impl FromStr for Error {
    type Err = Self;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(4, '|').collect();
        if parts.len() < 2 {
            return Err(Self::generic("Invalid error format"));
        }

        // Parse name and message (required)
        let name = ErrorName(parts.first().map_or("", |s| s.trim()).to_string());
        let message = ErrorMessage(parts.get(1).map_or("", |s| s.trim()).to_string());

        // Parse code (optional)
        let code = if parts.len() > 2 && !parts.get(2).map_or("", |s| s.trim()).is_empty() {
            match parts.get(2).map_or("", |s| s.trim()).parse::<u16>() {
                Ok(code_num) => Some(ErrorCode::from(code_num)),
                Err(_) => None,
            }
        } else {
            None
        };

        // Parse backtrace (optional)
        let backtrace = if parts.len() > 3 && !parts.get(3).map_or("", |s| s.trim()).is_empty() {
            Backtrace::from_str(parts.get(3).map_or("", |s| s.trim())).ok()
        } else {
            None
        };

        Ok(Self {
            source: Arc::new(ErrorInner::Generic(message.0.clone())),
            name: Some(name),
            code,
            message: Some(message),
            backtrace,
        })
    }
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl<T: Into<ErrorInner>> From<T> for Error {
    fn from(error: T) -> Self {
        Self::new(error)
    }
}

impl Error {
    pub fn new<E: Into<ErrorInner>>(error: E) -> Self {
        let error = error.into();
        let message = error.to_string();
        Self {
            source: Arc::new(error),
            name: Some(Default::default()),
            code: Some(Default::default()),
            message: Some(ErrorMessage(message)),
            backtrace: Some(std::backtrace::Backtrace::capture().into()),
        }
    }

    pub fn trace(self) -> Self {
        let error = self.clone();
        let error_name = error.name.unwrap_or_default().to_string();
        let error_code = self.code.unwrap_or_default().to_string();
        let error_message = error.message.unwrap_or_default().to_string();
        let error_backtrace = json!(error.backtrace.unwrap_or_default());
        let error_backtrace = error_backtrace.to_string();
        tracing::error!({
            error.name = error_name,
            error.code = error_code,
            error.message = error_message,
            error.backtrace = error_backtrace,
        });
        self
    }

    pub fn generic<U>(message: U) -> Self
    where
        U: Display + Debug + Send + Sync + 'static,
    {
        Self {
            source: Arc::new(ErrorInner::Generic(message.to_string())),
            name: Some(ErrorName("E_GENERIC".to_string())),
            code: Some(ErrorCode::default()),
            message: Some(ErrorMessage(message.to_string())),
            backtrace: Some(std::backtrace::Backtrace::capture().into()),
        }
    }

    pub fn source(&self) -> &ErrorInner {
        &self.source
    }

    pub fn with_name<U>(self, name: U) -> Self
    where
        U: Display + Debug + Send + Sync + 'static,
    {
        Self {
            name: Some(ErrorName(name.to_string())),
            code: self.code,
            source: self.source,
            message: self.message,
            backtrace: self.backtrace,
        }
    }

    pub fn with_message<U>(self, message: U) -> Self
    where
        U: Display + Debug + Send + Sync + 'static,
    {
        Self {
            name: self.name,
            code: self.code,
            source: self.source,
            message: Some(ErrorMessage(message.to_string())),
            backtrace: self.backtrace,
        }
    }

    pub fn with_status<U>(self, status: U) -> Self
    where
        U: Into<ErrorCode> + Send + Sync + 'static,
    {
        Self {
            name: self.name,
            code: Some(status.into()),
            source: self.source,
            message: self.message,
            backtrace: self.backtrace,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = Error::generic("An error occurred");
        assert_eq!(error.to_string(), "[E_GENERIC] An error occurred");
    }

    #[test]
    fn test_error_debug() {
        let error = Error::generic("An error occurred");
        let error = format!("{error:?}");
        // assert_eq!(
        //     format!("{error:?}"),
        //     "Error { name: ErrorName(\"E_GENERIC\"), message: ErrorMessage(\"An error occurred\"), code: Some(ErrorCode(500)), backtrace: Some(Backtrace { frames: [...] }) }"
        // );
        println!("{error}");
    }

    #[test]
    fn test_error_from_str() {
        let error_str = "E_NOT_FOUND|Resource not found|404|stack trace here";
        let error: Error = error_str.parse().unwrap();
        assert_eq!(error.name.unwrap().0, "E_NOT_FOUND");
        assert_eq!(error.message.unwrap().0, "Resource not found");
        assert_eq!(error.code.unwrap().into_status_code(), 404);
        assert!(error.backtrace.is_some());
    }
}
