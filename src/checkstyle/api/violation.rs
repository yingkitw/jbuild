//! Violation types for Checkstyle-rs

use crate::checkstyle::api::event::SeverityLevel;
use std::cmp::Ordering;

/// Represents a violation found by a check
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Line number where the violation occurs (1-based)
    pub line_no: usize,
    /// Column number where the violation occurs (1-based)
    pub column_no: usize,
    /// Column character index
    pub column_char_index: usize,
    /// Token type associated with the violation
    pub token_type: i32,
    /// Severity level of the violation
    pub severity_level: SeverityLevel,
    /// Module ID that generated the violation
    pub module_id: String,
    /// Message key for localization
    pub key: String,
    /// Arguments for message formatting
    pub args: Vec<String>,
    /// Resource bundle name
    pub bundle: String,
    /// Source class/module name
    pub source_name: String,
    /// Custom message (overrides default from bundle)
    pub custom_message: Option<String>,
}

impl Violation {
    /// Create a new violation
    pub fn new(
        line_no: usize,
        column_no: usize,
        column_char_index: usize,
        token_type: i32,
        severity_level: SeverityLevel,
        module_id: String,
        key: String,
        args: Vec<String>,
        bundle: String,
        source_name: String,
        custom_message: Option<String>,
    ) -> Self {
        Self {
            line_no,
            column_no,
            column_char_index,
            token_type,
            severity_level,
            module_id,
            key,
            args,
            bundle,
            source_name,
            custom_message,
        }
    }

    /// Get the formatted violation message
    pub fn get_message(&self) -> String {
        if let Some(ref custom) = self.custom_message {
            // Simple formatting - replace {0}, {1}, etc. with args
            let mut msg = custom.clone();
            for (i, arg) in self.args.iter().enumerate() {
                msg = msg.replace(&format!("{{{}}}", i), arg);
            }
            msg
        } else {
            // TODO: Implement proper localization
            format!("{}: {}", self.key, self.args.join(", "))
        }
    }
}

impl PartialOrd for Violation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Violation {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.line_no.cmp(&other.line_no) {
            Ordering::Equal => match self.column_no.cmp(&other.column_no) {
                Ordering::Equal => match self.module_id.cmp(&other.module_id) {
                    Ordering::Equal => match self.source_name.cmp(&other.source_name) {
                        Ordering::Equal => self.get_message().cmp(&other.get_message()),
                        other => other,
                    },
                    other => other,
                },
                other => other,
            },
            other => other,
        }
    }
}
