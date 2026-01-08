//! Checker implementation for Checkstyle-rs

use crate::checkstyle::api::check::FileSetCheck;
use crate::checkstyle::api::config::{Configuration, Context};
use crate::checkstyle::api::error::{CheckstyleError, CheckstyleResult};
use crate::checkstyle::api::event::{AuditEvent, AuditEventType, SeverityLevel};
use crate::checkstyle::api::file::{FileContents, FileText};
use crate::checkstyle::api::listener::AuditListener;
use crate::checkstyle::runner::module_factory::{
    DefaultModuleFactory, ModuleFactory, ModuleInstance, configure_module,
};
use std::path::{Path, PathBuf};

/// Main checker that orchestrates file checking
pub struct Checker {
    /// File set checks
    file_set_checks: Vec<Box<dyn FileSetCheck>>,
    /// Audit listeners
    listeners: Vec<Box<dyn AuditListener>>,
    /// File extensions to process
    file_extensions: Vec<String>,
    /// Context for child modules
    context: Context,
    /// Charset for file reading
    charset: String,
}

impl Checker {
    /// Create a new checker
    pub fn new() -> Self {
        Self {
            file_set_checks: Vec::new(),
            listeners: Vec::new(),
            file_extensions: vec!["java".to_string()],
            context: Context::new(),
            charset: "UTF-8".to_string(),
        }
    }

    /// Add a file set check
    pub fn add_file_set_check(&mut self, check: Box<dyn FileSetCheck>) {
        self.file_set_checks.push(check);
    }

    /// Add an audit listener
    pub fn add_listener(&mut self, listener: Box<dyn AuditListener>) {
        self.listeners.push(listener);
    }

    /// Configure the checker
    pub fn configure(&mut self, config: &Configuration) -> CheckstyleResult<()> {
        // Configure file extensions if specified
        if let Some(extensions) = config.get_property("fileExtensions") {
            self.file_extensions = extensions
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        // Configure charset if specified
        if let Some(charset) = config.get_property("charset") {
            self.charset = charset.clone();
        }

        // Configure severity
        if let Some(severity_str) = config.get_property("severity") {
            self.context.severity = match severity_str.as_str() {
                "ignore" => SeverityLevel::Ignore,
                "info" => SeverityLevel::Info,
                "warning" => SeverityLevel::Warning,
                "error" => SeverityLevel::Error,
                _ => SeverityLevel::Error,
            };
        }

        // Create and configure child modules
        let factory = DefaultModuleFactory::new();
        for child_config in config.get_children() {
            let mut instance = factory.create_module(&child_config.name)?;
            configure_module(&mut instance, child_config, &self.context)?;

            match instance {
                ModuleInstance::FileSetCheck(check) => {
                    self.add_file_set_check(check);
                }
                ModuleInstance::Check(_) => {
                    // Checks should be added to TreeWalker, not Checker directly
                    // This will be handled when TreeWalker is configured
                }
            }
        }

        Ok(())
    }

    /// Process a list of files
    pub fn process(&mut self, files: &[PathBuf]) -> CheckstyleResult<usize> {
        let mut error_count = 0;

        // Fire audit started event
        let audit_started = AuditEvent::new(None, None, AuditEventType::AuditStarted);
        for listener in &mut self.listeners {
            listener.audit_started(&audit_started)?;
        }

        // Initialize file set checks
        for check in &mut self.file_set_checks {
            check.init()?;
            check.begin_processing(&self.charset)?;
        }

        // Process each file
        for file in files {
            if !self.should_process_file(file) {
                continue;
            }

            // Fire file started event
            let file_started =
                AuditEvent::new(Some(file.clone()), None, AuditEventType::FileStarted);
            for listener in &mut self.listeners {
                listener.file_started(&file_started)?;
            }

            // Read file
            let file_text = self.read_file(file)?;
            let file_contents = FileContents::new(file_text);

            // Process with file set checks
            for check in &mut self.file_set_checks {
                match check.process(file, file_contents.get_text()) {
                    Ok(violations) => {
                        for violation in violations {
                            // Only count ERROR level violations
                            if violation.severity_level == SeverityLevel::Error {
                                error_count += 1;
                            }
                            let event = AuditEvent::new(
                                Some(file.clone()),
                                Some(violation),
                                AuditEventType::AddError,
                            );
                            for listener in &mut self.listeners {
                                listener.add_error(&event)?;
                            }
                        }
                    }
                    Err(e) => {
                        let event =
                            AuditEvent::new(Some(file.clone()), None, AuditEventType::AddException);
                        for listener in &mut self.listeners {
                            listener.add_exception(&event, &e)?;
                        }
                    }
                }
            }

            // Fire file finished event
            let file_finished =
                AuditEvent::new(Some(file.clone()), None, AuditEventType::FileFinished);
            for listener in &mut self.listeners {
                listener.file_finished(&file_finished)?;
            }
        }

        // Finish processing
        for check in &mut self.file_set_checks {
            check.finish_processing()?;
            check.destroy()?;
        }

        // Fire audit finished event
        let audit_finished = AuditEvent::new(None, None, AuditEventType::AuditFinished);
        for listener in &mut self.listeners {
            listener.audit_finished(&audit_finished)?;
        }

        Ok(error_count)
    }

    /// Check if a file should be processed
    fn should_process_file(&self, file: &Path) -> bool {
        if let Some(ext) = file.extension() {
            if let Some(ext_str) = ext.to_str() {
                return self.file_extensions.iter().any(|e| e == ext_str);
            }
        }
        false
    }

    /// Read a file
    fn read_file(&self, path: &Path) -> CheckstyleResult<FileText> {
        let content = std::fs::read_to_string(path).map_err(CheckstyleError::Io)?;
        Ok(FileText::new(path.to_path_buf(), content))
    }
}

impl Default for Checker {
    fn default() -> Self {
        Self::new()
    }
}
