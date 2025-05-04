use anyhow::{Ok, Result};

#[derive(Clone)]
pub struct TestContext {}

impl TestContext {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}
