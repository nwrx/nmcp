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

/// Add `Suggestions` to the error report so that the client can understand how to resolve the error.
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Suggestion(pub String);
impl Display for Suggestion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
