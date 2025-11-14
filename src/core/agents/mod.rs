//! Agent implementations
//! 
//! Contains specialized agents for various development tasks
//! This will be expanded starting in Phase 6

pub mod base;
pub mod requirements;
pub mod design;
pub mod code;
pub mod test;
pub mod simulations;
pub mod review;
pub mod ethics_security;
pub mod deployment;
pub mod meta;
pub mod architect;
pub mod developer;
pub mod qa;
pub mod collaboration;
pub mod devops;
pub mod scrum;
pub mod i18n;
pub mod a11y;
pub mod collaboration_realtime;
pub mod ide_extension;
pub mod green_dev;
pub mod enhanced_a11y;
pub mod marketplace;
pub mod documentation;
pub mod release_manager;
pub mod quality_assurance;
pub mod maintenance;

pub use requirements::RequirementsAgent;
pub use design::DesignAgent;
pub use code::CodeAgent;
pub use test::TestAgent;
pub use simulations::{ProjectManagerSimulation, BusinessAnalystSimulation};
pub use review::ReviewAgent;
pub use ethics_security::EthicsSecurityAgent;
pub use deployment::DeploymentAgent;
pub use meta::MetaAgent;
pub use architect::ArchitectSimulation;
pub use developer::DeveloperSimulation;
pub use qa::QaSimulation;
pub use collaboration::CollaborationManager;
pub use devops::DevOpsSimulation;
pub use scrum::ScrumSimulation;
pub use i18n::I18nAssistant;
pub use a11y::A11yAssistant;
pub use collaboration_realtime::RealTimeCollaboration;
pub use ide_extension::IdeExtension;
