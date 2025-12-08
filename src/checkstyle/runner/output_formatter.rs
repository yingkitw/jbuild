//! Output formatters for Checkstyle-rs

use crate::checkstyle::api::violation::Violation;
use std::io::Write;

/// Trait for formatting output
pub trait OutputFormatter: Send + Sync {
    /// Format a violation
    fn format_violation(&self, violation: &Violation, file_name: &str) -> String;

    /// Write violations to a writer
    fn write_violations<W: Write>(
        &self,
        writer: &mut W,
        violations: &[(String, Vec<Violation>)],
    ) -> std::io::Result<()>;
}

/// Plain text formatter (default Checkstyle format)
pub struct PlainFormatter;

impl PlainFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlainFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for PlainFormatter {
    fn format_violation(&self, violation: &Violation, file_name: &str) -> String {
        let severity_str = match violation.severity_level {
            crate::checkstyle::api::event::SeverityLevel::Warning => "WARN",
            crate::checkstyle::api::event::SeverityLevel::Error => "ERROR",
            crate::checkstyle::api::event::SeverityLevel::Info => "INFO",
            crate::checkstyle::api::event::SeverityLevel::Ignore => "IGNORE",
        };

        if violation.column_no > 0 {
            format!(
                "[{}] {}:{}:{}: {} [{}]",
                severity_str,
                file_name,
                violation.line_no,
                violation.column_no,
                violation.get_message(),
                violation.module_id
            )
        } else {
            format!(
                "[{}] {}:{}: {} [{}]",
                severity_str,
                file_name,
                violation.line_no,
                violation.get_message(),
                violation.module_id
            )
        }
    }

    fn write_violations<W: Write>(
        &self,
        writer: &mut W,
        violations: &[(String, Vec<Violation>)],
    ) -> std::io::Result<()> {
        for (file_name, file_violations) in violations {
            for violation in file_violations {
                if violation.severity_level != crate::checkstyle::api::event::SeverityLevel::Ignore {
                    writeln!(writer, "{}", self.format_violation(violation, file_name))?;
                }
            }
        }
        Ok(())
    }
}

/// XML formatter
pub struct XmlFormatter;

impl XmlFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for XmlFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for XmlFormatter {
    fn format_violation(&self, violation: &Violation, _file_name: &str) -> String {
        format!(
            r#"<error line="{}" column="{}" severity="{}" message="{}" source="{}"/>"#,
            violation.line_no,
            violation.column_no,
            match violation.severity_level {
                crate::checkstyle::api::event::SeverityLevel::Warning => "warning",
                crate::checkstyle::api::event::SeverityLevel::Error => "error",
                crate::checkstyle::api::event::SeverityLevel::Info => "info",
                crate::checkstyle::api::event::SeverityLevel::Ignore => "ignore",
            },
            xml_escape(&violation.get_message()),
            xml_escape(&violation.module_id)
        )
    }

    fn write_violations<W: Write>(
        &self,
        writer: &mut W,
        violations: &[(String, Vec<Violation>)],
    ) -> std::io::Result<()> {
        writeln!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
        writeln!(writer, r#"<checkstyle version="checkstyle-rs">"#)?;

        for (file_name, file_violations) in violations {
            if !file_violations.is_empty() {
                writeln!(writer, r#"  <file name="{}">"#, xml_escape(file_name))?;
                for violation in file_violations {
                    if violation.severity_level != crate::checkstyle::api::event::SeverityLevel::Ignore {
                        writeln!(
                            writer,
                            "    {}",
                            self.format_violation(violation, file_name)
                        )?;
                    }
                }
                writeln!(writer, r#"  </file>"#)?;
            }
        }

        writeln!(writer, r#"</checkstyle>"#)?;
        Ok(())
    }
}

/// Escape XML special characters
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
