//! Listener traits for Checkstyle-rs

use crate::checkstyle::api::error::CheckstyleResult;
use crate::checkstyle::api::event::AuditEvent;

/// Listener in charge of receiving events from the Checker
///
/// Typical event sequence:
/// - audit_started
///   - (file_started
///     - (add_error)*
///   - file_finished)*
/// - audit_finished
pub trait AuditListener: Send + Sync {
    /// Notify that the audit is about to start
    fn audit_started(&mut self, event: &AuditEvent) -> CheckstyleResult<()>;

    /// Notify that the audit is finished
    fn audit_finished(&mut self, event: &AuditEvent) -> CheckstyleResult<()>;

    /// Notify that audit is about to start on a specific file
    fn file_started(&mut self, event: &AuditEvent) -> CheckstyleResult<()>;

    /// Notify that audit is finished on a specific file
    fn file_finished(&mut self, event: &AuditEvent) -> CheckstyleResult<()>;

    /// Notify that an audit error was discovered on a specific file
    fn add_error(&mut self, event: &AuditEvent) -> CheckstyleResult<()>;

    /// Notify that an exception happened while performing audit
    fn add_exception(
        &mut self,
        event: &AuditEvent,
        error: &dyn std::error::Error,
    ) -> CheckstyleResult<()>;
}
