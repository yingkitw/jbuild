//! FileSetCheck wrapper for TreeWalker

use crate::checkstyle::api::check::{FileSetCheck, MessageDispatcher};
use crate::checkstyle::api::config::{Configurable, Configuration, Context, Contextualizable};
use crate::checkstyle::api::error::CheckstyleResult;
use crate::checkstyle::api::file::FileText;
use crate::checkstyle::api::violation::Violation;
use crate::checkstyle::runner::{ModuleFactory, TreeWalker};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// FileSetCheck implementation that wraps TreeWalker
pub struct TreeWalkerFileSetCheck {
    /// The tree walker
    tree_walker: Arc<Mutex<TreeWalker>>,
    /// Message dispatcher
    message_dispatcher: Option<Box<dyn MessageDispatcher>>,
    /// Context
    context: Context,
}

impl TreeWalkerFileSetCheck {
    /// Create a new TreeWalkerFileSetCheck
    pub fn new() -> Self {
        Self {
            tree_walker: Arc::new(Mutex::new(TreeWalker::new())),
            message_dispatcher: None,
            context: Context::new(),
        }
    }

    /// Get a reference to the tree walker
    pub fn get_tree_walker(&self) -> Arc<Mutex<TreeWalker>> {
        self.tree_walker.clone()
    }
}

impl Default for TreeWalkerFileSetCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl Configurable for TreeWalkerFileSetCheck {
    fn configure(&mut self, config: &Configuration) -> CheckstyleResult<()> {
        // Create and configure child checks
        let factory = crate::checkstyle::runner::module_factory::DefaultModuleFactory::new();
        let mut walker = self.tree_walker.lock().unwrap();

        for child_config in config.get_children() {
            match factory.create_module(&child_config.name) {
                Ok(instance) => {
                    match instance {
                        crate::checkstyle::runner::module_factory::ModuleInstance::Check(check) => {
                            // Configure the check
                            let mut check_guard = check.lock().unwrap();
                            check_guard.contextualize(&self.context)?;
                            check_guard.configure(child_config)?;
                            drop(check_guard);

                            // Add to tree walker
                            walker.add_check(check)?;
                        }
                        crate::checkstyle::runner::module_factory::ModuleInstance::FileSetCheck(_) => {
                            // FileSetChecks are not allowed as children of TreeWalker
                            return Err(crate::checkstyle::api::error::CheckstyleError::Configuration(
                                format!(
                                    "TreeWalker cannot have FileSetCheck as child: {}",
                                    child_config.name
                                ),
                            ));
                        }
                    }
                }
                Err(e) => {
                    // Ignore unknown modules for now
                    eprintln!(
                        "Warning: Could not create module {}: {}",
                        child_config.name, e
                    );
                }
            }
        }

        Ok(())
    }
}

impl Contextualizable for TreeWalkerFileSetCheck {
    fn contextualize(&mut self, context: &Context) -> CheckstyleResult<()> {
        self.context = context.clone();
        let mut walker = self.tree_walker.lock().unwrap();
        walker.set_child_context(context.clone());
        Ok(())
    }
}

impl FileSetCheck for TreeWalkerFileSetCheck {
    fn set_message_dispatcher(&mut self, dispatcher: Box<dyn MessageDispatcher>) {
        self.message_dispatcher = Some(dispatcher);
    }

    fn init(&mut self) -> CheckstyleResult<()> {
        // TreeWalker is initialized when checks are added
        Ok(())
    }

    fn destroy(&mut self) -> CheckstyleResult<()> {
        // Cleanup if needed
        Ok(())
    }

    fn begin_processing(&mut self, _charset: &str) -> CheckstyleResult<()> {
        Ok(())
    }

    fn process(
        &mut self,
        file: &PathBuf,
        file_text: &FileText,
    ) -> CheckstyleResult<BTreeSet<Violation>> {
        let walker = self.tree_walker.lock().unwrap();
        walker.process_file(file, file_text)
    }

    fn finish_processing(&mut self) -> CheckstyleResult<()> {
        Ok(())
    }
}
