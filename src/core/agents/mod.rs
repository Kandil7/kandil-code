//! Agent implementations
//!
//! Contains specialized agents for various development tasks
//! This will be expanded starting in Phase 6

pub mod a11y;
pub mod architect;
pub mod base;
pub mod code;
pub mod collaboration;
pub mod collaboration_realtime;
pub mod consensus;
pub mod deployment;
pub mod design;
pub mod developer;
pub mod devops;
pub mod documentation;
pub mod enhanced_a11y;
pub mod ethics_security;
pub mod green_dev;
pub mod i18n;
pub mod ide_extension;
pub mod maintenance;
pub mod marketplace;
pub mod meta;
pub mod qa;
pub mod quality_assurance;
pub mod release_manager;
pub mod requirements;
pub mod review;
pub mod scrum;
pub mod simulations;
pub mod test;

pub use a11y::A11yAssistant;
pub use architect::ArchitectSimulation;
pub use code::CodeAgent;
pub use collaboration::CollaborationManager;
pub use collaboration_realtime::RealTimeCollaboration;
pub use deployment::DeploymentAgent;
pub use design::DesignAgent;
pub use developer::DeveloperSimulation;
pub use devops::DevOpsSimulation;
pub use ethics_security::EthicsSecurityAgent;
pub use i18n::I18nAssistant;
pub use ide_extension::IdeExtension;
pub use meta::MetaAgent;
pub use qa::QaSimulation;
pub use requirements::RequirementsAgent;
pub use review::ReviewAgent;
pub use scrum::ScrumSimulation;
pub use simulations::{BusinessAnalystSimulation, ProjectManagerSimulation};
pub use test::TestAgent;
