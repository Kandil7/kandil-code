//! Error handling for Kandil Code local model integration
//!
//! Contains all error types for the local model system.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LocalModelError {
    #[error("Model not found: {name}")]
    ModelNotFound { name: String },

    #[error("Model loading failed: {source}")]
    ModelLoadError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Model download failed: {source}")]
    ModelDownloadError {
        #[from]
        source: reqwest::Error,
    },

    #[error("Hardware insufficient: {requirement} required, {available} available")]
    InsufficientHardware {
        requirement: String,
        available: String,
    },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Cache error: {message}")]
    CacheError { message: String },

    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },

    #[error("Validation error: {message}")]
    ValidationError { message: String },
}

// Add to the main module
