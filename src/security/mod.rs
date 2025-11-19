//! Security utilities for Kandil Code.
//!
//! This module centralizes access to sensitive credentials so other parts of
//! the codebase can rely on a hardened API rather than touching the OS
//! keyring directly.

pub mod credentials;
pub mod mobile;
pub mod model;
pub mod platform;

#[allow(unused_imports)]
pub use credentials::{CredentialManager, CredentialState};
