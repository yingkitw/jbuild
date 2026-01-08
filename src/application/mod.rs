//! Application layer - orchestrates domain services and repositories
//! 
//! Application services coordinate domain services, repositories, and external systems
//! to fulfill use cases. They are stateless and transaction-scoped.

pub mod build_orchestration;
pub mod project_initialization;
pub mod dependency_management;

pub use build_orchestration::BuildOrchestrationService;
pub use project_initialization::ProjectInitializationService;
pub use dependency_management::DependencyManagementService;
