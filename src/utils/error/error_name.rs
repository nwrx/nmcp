use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Add a `ErrorName` to the error report so that the client can understand what kind of error it is.
#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub struct ErrorName(pub String);

impl Display for ErrorName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ErrorName {
    fn default() -> Self {
        Self("E_UNKNOWN".to_string())
    }
}
