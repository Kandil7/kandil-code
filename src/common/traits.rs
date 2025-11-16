//! Common traits for Kandil Code
//!
//! Contains shared traits that are used across different modules.

use crate::errors::LocalModelError;
use async_trait::async_trait;

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String, LocalModelError>;
    async fn is_available(&self) -> bool;
    async fn name(&self) -> String;
}
