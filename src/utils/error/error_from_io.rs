use axum::http::StatusCode;

use super::{Error, ErrorInner};

impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Self {
        let message = source.to_string();
        let kind = source.kind().to_string().replace(' ', "_").to_uppercase();
        let name = format!("E_IO_{kind}");
        let source = ErrorInner::IoError(source);
        Self::new(source)
            .with_name(name)
            .with_status(StatusCode::INTERNAL_SERVER_ERROR)
            .with_message(message)
    }
}
