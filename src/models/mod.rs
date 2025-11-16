//! Model management for Kandil Code
//!
//! Contains modules for handling local model specifications and catalogs.

pub mod catalog;
pub mod registry;

#[allow(unused_imports)]
pub use registry::{ModelProfile, ModelResources, ProviderKind, UniversalModelRegistry};
