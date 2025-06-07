use super::{Error, ErrorMessage, ErrorName};
use axum::{http::StatusCode, response::IntoResponse};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ErrorBody {
    name: ErrorName,
    message: ErrorMessage,
    status_code: u16,
    status_message: String,
}

impl Default for ErrorBody {
    fn default() -> Self {
        Self {
            name: ErrorName::default(),
            message: ErrorMessage::default(),
            status_code: 500,
            status_message: "Internal Server Error".to_string(),
        }
    }
}

impl From<Error> for ErrorBody {
    fn from(error: Error) -> Self {
        let code = error.code.unwrap_or_default();
        Self {
            name: error.name.unwrap_or_default(),
            message: error.message.unwrap_or_default(),
            status_code: code.into_status_code(),
            status_message: code.into_status_message(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let body = ErrorBody::from(self);
        let status = StatusCode::from_u16(body.status_code).unwrap_or_default();
        (status, axum::Json(body)).into_response()
    }
}
