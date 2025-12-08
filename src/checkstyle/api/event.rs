//! Event types for Checkstyle-rs

use crate::checkstyle::api::violation::Violation;
use std::path::PathBuf;

/// Severity level for violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SeverityLevel {
    /// Ignore - violations are not reported
    Ignore,
    /// Info - informational violations
    Info,
    /// Warning - warning level violations
    Warning,
    /// Error - error level violations (default)
    Error,
}

impl Default for SeverityLevel {
    fn default() -> Self {
        Self::Error
    }
}

/// Audit event that can be sent to listeners
#[derive(Debug, Clone)]
pub struct AuditEvent {
    /// File being processed
    pub file: Option<PathBuf>,
    /// Violation associated with the event (if any)
    pub violation: Option<Violation>,
    /// Event type
    pub event_type: AuditEventType,
}

/// Type of audit event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditEventType {
    /// Audit started
    AuditStarted,
    /// Audit finished
    AuditFinished,
    /// File processing started
    FileStarted,
    /// File processing finished
    FileFinished,
    /// Error/violation found
    AddError,
    /// Exception occurred
    AddException,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(
        file: Option<PathBuf>,
        violation: Option<Violation>,
        event_type: AuditEventType,
    ) -> Self {
        Self {
            file,
            violation,
            event_type,
        }
    }
}
