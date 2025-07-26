use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// A generic error message that can be used to report errors in a consistent way.
#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub struct ErrorMessage(pub String);

impl Display for ErrorMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ErrorMessage {
    fn default() -> Self {
        Self("An unexpected error occurred".to_string())
    }
}
