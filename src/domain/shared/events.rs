//! Domain events for decoupled communication between bounded contexts

use std::time::SystemTime;
use serde::{Deserialize, Serialize};

/// Base trait for all domain events
pub trait DomainEvent: Send + Sync {
    fn event_type(&self) -> &str;
    fn occurred_at(&self) -> SystemTime;
}

/// Event emitted when a project is built
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBuiltEvent {
    pub project_name: String,
    pub version: String,
    pub occurred_at: SystemTime,
}

impl DomainEvent for ProjectBuiltEvent {
    fn event_type(&self) -> &str {
        "ProjectBuilt"
    }

    fn occurred_at(&self) -> SystemTime {
        self.occurred_at
    }
}

/// Event emitted when a dependency is resolved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyResolvedEvent {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub occurred_at: SystemTime,
}

impl DomainEvent for DependencyResolvedEvent {
    fn event_type(&self) -> &str {
        "DependencyResolved"
    }

    fn occurred_at(&self) -> SystemTime {
        self.occurred_at
    }
}

/// Event emitted when a lifecycle phase completes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecyclePhaseCompletedEvent {
    pub phase: String,
    pub project_name: String,
    pub occurred_at: SystemTime,
}

impl DomainEvent for LifecyclePhaseCompletedEvent {
    fn event_type(&self) -> &str {
        "LifecyclePhaseCompleted"
    }

    fn occurred_at(&self) -> SystemTime {
        self.occurred_at
    }
}

/// Event emitted when a task is executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutedEvent {
    pub task_name: String,
    pub project_name: String,
    pub success: bool,
    pub occurred_at: SystemTime,
}

impl DomainEvent for TaskExecutedEvent {
    fn event_type(&self) -> &str {
        "TaskExecuted"
    }

    fn occurred_at(&self) -> SystemTime {
        self.occurred_at
    }
}

/// Event emitted when compilation completes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationCompletedEvent {
    pub source_files: usize,
    pub success: bool,
    pub occurred_at: SystemTime,
}

impl DomainEvent for CompilationCompletedEvent {
    fn event_type(&self) -> &str {
        "CompilationCompleted"
    }

    fn occurred_at(&self) -> SystemTime {
        self.occurred_at
    }
}

/// Event emitted when tests complete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteCompletedEvent {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub occurred_at: SystemTime,
}

impl DomainEvent for TestSuiteCompletedEvent {
    fn event_type(&self) -> &str {
        "TestSuiteCompleted"
    }

    fn occurred_at(&self) -> SystemTime {
        self.occurred_at
    }
}

/// Event emitted when a package is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageCreatedEvent {
    pub artifact_name: String,
    pub package_type: String,
    pub output_path: String,
    pub occurred_at: SystemTime,
}

impl DomainEvent for PackageCreatedEvent {
    fn event_type(&self) -> &str {
        "PackageCreated"
    }

    fn occurred_at(&self) -> SystemTime {
        self.occurred_at
    }
}
