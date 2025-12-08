//! LineLength check - detects lines that exceed maximum length

use crate::checkstyle::api::check::FileSetCheck;
use crate::checkstyle::api::config::{Configurable, Configuration, Context, Contextualizable};
use crate::checkstyle::api::error::CheckstyleResult;
use crate::checkstyle::api::file::FileText;
use crate::checkstyle::api::violation::Violation;
use crate::checkstyle::utils::common_util::line_length_expanded;
use regex::Regex;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// Check for long lines
///
/// Detects lines that exceed the maximum allowed length.
/// By default, package and import statements are ignored.
pub struct LineLengthCheck {
    /// Maximum line length allowed
    max: usize,
    /// Pattern for lines to ignore
    ignore_pattern: Regex,
    /// Tab width for calculating expanded line length
    tab_width: usize,
    /// Context
    context: Context,
    /// Violations collected by this check
    violations: BTreeSet<Violation>,
}

impl LineLengthCheck {
    /// Create a new LineLengthCheck
    pub fn new() -> Self {
        Self {
            max: 80, // Default max columns
            ignore_pattern: Regex::new(r"^(package|import) .*").unwrap(),
            tab_width: 8, // Default tab width
            context: Context::new(),
            violations: BTreeSet::new(),
        }
    }

    /// Set the maximum line length
    pub fn set_max(&mut self, max: usize) {
        self.max = max;
    }

    /// Set the ignore pattern
    pub fn set_ignore_pattern(&mut self, pattern: String) -> Result<(), regex::Error> {
        self.ignore_pattern = Regex::new(&pattern)?;
        Ok(())
    }

    /// Set the tab width
    pub fn set_tab_width(&mut self, tab_width: usize) {
        self.tab_width = tab_width;
    }
}

impl Default for LineLengthCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl Configurable for LineLengthCheck {
    fn configure(&mut self, config: &Configuration) -> CheckstyleResult<()> {
        // Read max property
        if let Some(max_str) = config.get_property("max") {
            if let Ok(max_val) = max_str.parse::<usize>() {
                self.set_max(max_val);
            }
        }

        // Read ignorePattern property
        if let Some(pattern_str) = config.get_property("ignorePattern") {
            if let Err(e) = self.set_ignore_pattern(pattern_str.clone()) {
                return Err(crate::checkstyle::api::error::CheckstyleError::Configuration(format!(
                    "Invalid ignorePattern: {}",
                    e
                )));
            }
        }

        // Read tabWidth property
        if let Some(tab_width_str) = config.get_property("tabWidth") {
            if let Ok(tab_width_val) = tab_width_str.parse::<usize>() {
                self.set_tab_width(tab_width_val);
            }
        }

        Ok(())
    }
}

impl Contextualizable for LineLengthCheck {
    fn contextualize(&mut self, context: &Context) -> CheckstyleResult<()> {
        self.context = context.clone();
        // Use tab width from context if not explicitly configured
        if self.tab_width == 8 {
            // Only use context if still at default
            self.tab_width = context.tab_width;
        }
        Ok(())
    }
}

impl FileSetCheck for LineLengthCheck {
    fn set_message_dispatcher(
        &mut self,
        _dispatcher: Box<dyn crate::checkstyle::api::check::MessageDispatcher>,
    ) {
        // Not used for now
    }

    fn init(&mut self) -> CheckstyleResult<()> {
        self.violations.clear();
        Ok(())
    }

    fn destroy(&mut self) -> CheckstyleResult<()> {
        Ok(())
    }

    fn begin_processing(&mut self, _charset: &str) -> CheckstyleResult<()> {
        Ok(())
    }

    fn process(
        &mut self,
        _file: &PathBuf,
        file_text: &FileText,
    ) -> CheckstyleResult<BTreeSet<Violation>> {
        let mut violations = BTreeSet::new();

        // Process each line
        for (i, line) in file_text.lines.iter().enumerate() {
            let line_no = i + 1; // 1-based line number

            // Calculate expanded line length (with tabs expanded)
            let real_length = line_length_expanded(line, self.tab_width);

            // Check if line exceeds max and doesn't match ignore pattern
            if real_length > self.max && !self.ignore_pattern.is_match(line) {
                let violation = Violation::new(
                    line_no,
                    0,
                    0,
                    0,
                    self.context.severity,
                    "LineLength".to_string(),
                    "maxLineLen".to_string(),
                    vec![self.max.to_string(), real_length.to_string()],
                    "checkstyle".to_string(),
                    "LineLength".to_string(),
                    None,
                );
                violations.insert(violation);
            }
        }

        Ok(violations)
    }

    fn finish_processing(&mut self) -> CheckstyleResult<()> {
        Ok(())
    }
}
