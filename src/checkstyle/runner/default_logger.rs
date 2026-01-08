//! Default logger implementation for Checkstyle-rs

use crate::checkstyle::api::error::{CheckstyleError, CheckstyleResult};
use crate::checkstyle::api::event::AuditEvent;
use crate::checkstyle::api::listener::AuditListener;
use std::io::{self, Write};
use std::sync::Mutex;

/// Default logger that writes violations to stdout/stderr
pub struct DefaultLogger {
    /// Output stream (stdout) - wrapped in Mutex for Sync
    output: Mutex<Box<dyn Write + Send>>,
}

impl DefaultLogger {
    /// Create a new default logger writing to stdout
    pub fn new() -> Self {
        Self {
            output: Mutex::new(Box::new(io::stdout())),
        }
    }

    /// Create a logger writing to a custom writer
    pub fn with_writer(writer: Box<dyn Write + Send>) -> Self {
        Self {
            output: Mutex::new(writer),
        }
    }
}

impl Default for DefaultLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditListener for DefaultLogger {
    fn audit_started(&mut self, _event: &AuditEvent) -> CheckstyleResult<()> {
        // DefaultLogger in Checkstyle doesn't print audit started message
        Ok(())
    }

    fn audit_finished(&mut self, _event: &AuditEvent) -> CheckstyleResult<()> {
        // DefaultLogger in Checkstyle doesn't print audit finished message
        Ok(())
    }

    fn file_started(&mut self, _event: &AuditEvent) -> CheckstyleResult<()> {
        // DefaultLogger in Checkstyle doesn't print file started message
        Ok(())
    }

    fn file_finished(&mut self, _event: &AuditEvent) -> CheckstyleResult<()> {
        Ok(())
    }

    fn add_error(&mut self, event: &AuditEvent) -> CheckstyleResult<()> {
        if let Some(violation) = &event.violation {
            // Skip IGNORE severity violations
            if violation.severity_level == crate::checkstyle::api::event::SeverityLevel::Ignore {
                return Ok(());
            }

            let file_name = event
                .file
                .as_ref()
                .map(|f| f.display().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            // Format severity level - WARNING becomes WARN, others are uppercase
            let severity_str = match violation.severity_level {
                crate::checkstyle::api::event::SeverityLevel::Warning => "WARN",
                crate::checkstyle::api::event::SeverityLevel::Error => "ERROR",
                crate::checkstyle::api::event::SeverityLevel::Info => "INFO",
                crate::checkstyle::api::event::SeverityLevel::Ignore => "IGNORE", // Shouldn't reach here
            };

            let mut output = self.output.lock().unwrap();

            // Format: [SEVERITY] filePath:lineNo:columnNo: message [ModuleId]
            // Column is only shown if > 0
            if violation.column_no > 0 {
                writeln!(
                    output,
                    "[{}] {}:{}:{}: {} [{}]",
                    severity_str,
                    file_name,
                    violation.line_no,
                    violation.column_no,
                    violation.get_message(),
                    violation.module_id
                )
                .map_err(CheckstyleError::Io)?;
            } else {
                writeln!(
                    output,
                    "[{}] {}:{}: {} [{}]",
                    severity_str,
                    file_name,
                    violation.line_no,
                    violation.get_message(),
                    violation.module_id
                )
                .map_err(CheckstyleError::Io)?;
            }
        }
        Ok(())
    }

    fn add_exception(
        &mut self,
        event: &AuditEvent,
        error: &dyn std::error::Error,
    ) -> CheckstyleResult<()> {
        let file_name = event
            .file
            .as_ref()
            .map(|f| f.display().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let mut output = self.output.lock().unwrap();
        writeln!(
            output,
            "[ERROR] {file_name}: Exception occurred: {error}"
        )
        .map_err(CheckstyleError::Io)?;
        Ok(())
    }
}
