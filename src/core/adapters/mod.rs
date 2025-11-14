//! Adapters for external services
//! 
//! Contains implementations for interacting with external services
//! such as AI providers, Git, and file systems.

pub mod ai;
pub mod git;
pub mod file;

pub use ai::tracked::TrackedAI;