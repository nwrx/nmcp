// Serialization utilities

use crate::Error;
use serde::Serialize;
use serde_json::Value;
use std::str::FromStr;

/// Supported serialization formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializeFormat {
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

impl FromStr for SerializeFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(SerializeFormat::Json),
            "yaml" => Ok(SerializeFormat::Yaml),
            _ => Err(Error::Generic(format!("Unsupported format: {}. Supported formats are 'json' and 'yaml'", s))),
        }
    }
}

impl SerializeFormat {
    /// Returns the string representation of the format
    pub fn to_string(self) -> &'static str {
        match self {
            SerializeFormat::Json => "json",
            SerializeFormat::Yaml => "yaml",
        }
    }
}

/// Formats and outputs data in the specified format (JSON or YAML)
pub fn serialize<T: Serialize>(data: &T, format: SerializeFormat) -> Result<String, Error> {
    match format {
        SerializeFormat::Json => {
            serde_json::to_string_pretty(data)
                .map_err(Error::SerializationError)
        },
        SerializeFormat::Yaml => {
            let json_value: Value = serde_json::to_value(data)
                .map_err(Error::SerializationError)?;

            // Convert JSON value to YAML string
            serde_yaml::to_string(&json_value)
                .map_err(|e| Error::Generic(format!("Failed to serialize to YAML: {}", e)))
        },
    }
}
