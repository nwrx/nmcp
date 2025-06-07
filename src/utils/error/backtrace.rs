use super::{Error, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct Backtrace {
    /// A collection of frames representing the backtrace.
    pub frames: Vec<Frame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Frame {
    /// Name of the function or method.
    pub name: Option<String>,

    /// Path to the source file.
    pub filename: Option<PathBuf>,

    /// Line number in the source file.
    pub lineno: Option<u32>,

    /// Column number in the source file.
    pub colno: Option<u32>,
}

impl FromStr for Backtrace {
    type Err = Error;
    fn from_str(backtrace: &str) -> Result<Self> {
        let mut frames = Vec::new();
        let lines: Vec<&str> = backtrace.lines().collect();
        let mut i = 0;

        while (i + 1) < lines.len() {
            let name_line = match lines.get(i) {
                Some(line) => line,
                None => {
                    i += 1;
                    continue;
                }
            };
            let location_line = match lines.get(i + 1) {
                Some(line) => line,
                None => {
                    i += 1;
                    continue;
                }
            };

            // Parse frame number and function name
            if let Some((frame_num, name)) = name_line.split_once(": ") {
                if frame_num.trim().parse::<usize>().is_err() {
                    i += 1;
                    continue;
                }

                // Parse location information
                let location = location_line.trim_start();
                if !location.starts_with("at ") {
                    i += 1;
                    continue;
                }

                let location = &location[3..].trim(); // Skip "at " prefix

                // Parse filename:line:col format
                let mut parts = location.rsplitn(3, ':');
                let colno = parts.next().and_then(|s| s.parse::<u32>().ok());
                let lineno = parts.next().and_then(|s| s.parse::<u32>().ok());
                let filename = parts.next().map(PathBuf::from);

                frames.push(Frame {
                    name: Some(name.trim().to_string()),
                    filename,
                    lineno,
                    colno,
                });

                i += 2; // Move to the next frame
            } else {
                i += 1;
            }
        }

        Ok(Self { frames })
    }
}

impl From<std::backtrace::Backtrace> for Backtrace {
    fn from(backtrace: std::backtrace::Backtrace) -> Self {
        let backtrace = format!("{backtrace}");
        Self::from_str(&backtrace).unwrap_or_else(|_| Self { frames: vec![] })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backtrace_from_str() {
        let backtrace_str = r#"
            0: nmcp::utils::error::error::Error::new
                at ./src/utils/error/error.rs:47:29
            1: some::module::Error::from
                at ./src/utils/error/error.rs:115:17
            2: core::ops::function::FnOnce::call_once
                at /some/path/to/rust/lib.rs:250:5
        "#;

        assert_eq!(
            Backtrace::from_str(backtrace_str).unwrap().frames,
            vec![
                Frame {
                    name: Some("nmcp::utils::error::error::Error::new".to_string()),
                    filename: Some(PathBuf::from("./src/utils/error/error.rs")),
                    lineno: Some(47),
                    colno: Some(29),
                },
                Frame {
                    name: Some("some::module::Error::from".to_string()),
                    filename: Some(PathBuf::from("./src/utils/error/error.rs")),
                    lineno: Some(115),
                    colno: Some(17),
                },
                Frame {
                    name: Some("core::ops::function::FnOnce::call_once".to_string()),
                    filename: Some(PathBuf::from("/some/path/to/rust/lib.rs")),
                    lineno: Some(250),
                    colno: Some(5),
                }
            ]
        );
    }
}
