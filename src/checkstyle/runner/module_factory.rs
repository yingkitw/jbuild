//! Module factory for creating module instances from configuration

use crate::checkstyle::api::check::{Check, FileSetCheck};
use crate::checkstyle::api::config::{Configuration, Context};
use crate::checkstyle::api::error::{CheckstyleError, CheckstyleResult};
use crate::checkstyle::checks::empty_catch_block::EmptyCatchBlockCheck;
use crate::checkstyle::checks::empty_statement::EmptyStatementCheck;
use crate::checkstyle::checks::line_length::LineLengthCheck;
use crate::checkstyle::checks::missing_switch_default::MissingSwitchDefaultCheck;
use crate::checkstyle::checks::multiple_variable_declarations::MultipleVariableDeclarationsCheck;
use crate::checkstyle::checks::package_name::PackageNameCheck;
use crate::checkstyle::checks::redundant_import::RedundantImportCheck;
use crate::checkstyle::checks::simplify_boolean_return::SimplifyBooleanReturnCheck;
use crate::checkstyle::checks::type_name::TypeNameCheck;
use crate::checkstyle::runner::tree_walker_file_set_check::TreeWalkerFileSetCheck;
use std::sync::{Arc, Mutex};

/// Factory for creating module instances
pub trait ModuleFactory: Send + Sync {
    /// Create a module instance from a name
    fn create_module(&self, name: &str) -> CheckstyleResult<ModuleInstance>;
}

/// A module instance that can be configured
pub enum ModuleInstance {
    /// A FileSetCheck instance
    FileSetCheck(Box<dyn FileSetCheck>),
    /// A Check instance (for TreeWalker)
    Check(Arc<Mutex<dyn Check>>),
}

/// Default module factory implementation
pub struct DefaultModuleFactory;

impl DefaultModuleFactory {
    /// Create a new default module factory
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultModuleFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleFactory for DefaultModuleFactory {
    fn create_module(&self, name: &str) -> CheckstyleResult<ModuleInstance> {
        match name {
            "Checker" => {
                // Checker is handled separately, not created here
                Err(CheckstyleError::Configuration("Checker should not be created via factory".to_string()))
            }
            "TreeWalker" => {
                let file_set_check = TreeWalkerFileSetCheck::new();
                Ok(ModuleInstance::FileSetCheck(Box::new(file_set_check)))
            }
            "LineLength" => {
                let check = LineLengthCheck::new();
                Ok(ModuleInstance::FileSetCheck(Box::new(check)))
            }
            "EmptyCatchBlock" => {
                let check = EmptyCatchBlockCheck::new();
                Ok(ModuleInstance::Check(Arc::new(Mutex::new(check))))
            }
            "EmptyStatement" => {
                let check = EmptyStatementCheck::new();
                Ok(ModuleInstance::Check(Arc::new(Mutex::new(check))))
            }
            "MissingSwitchDefault" => {
                let check = MissingSwitchDefaultCheck::new();
                Ok(ModuleInstance::Check(Arc::new(Mutex::new(check))))
            }
            "SimplifyBooleanReturn" => {
                let check = SimplifyBooleanReturnCheck::new();
                Ok(ModuleInstance::Check(Arc::new(Mutex::new(check))))
            }
            "MultipleVariableDeclarations" => {
                let check = MultipleVariableDeclarationsCheck::new();
                Ok(ModuleInstance::Check(Arc::new(Mutex::new(check))))
            }
            "PackageName" => {
                let check = PackageNameCheck::new();
                Ok(ModuleInstance::Check(Arc::new(Mutex::new(check))))
            }
            "RedundantImport" => {
                let check = RedundantImportCheck::new();
                Ok(ModuleInstance::Check(Arc::new(Mutex::new(check))))
            }
            "TypeName" => {
                let check = TypeNameCheck::new();
                Ok(ModuleInstance::Check(Arc::new(Mutex::new(check))))
            }
            _ => Err(CheckstyleError::Configuration(format!(
                "Unknown module: {name}"
            ))),
        }
    }
}

/// Helper to configure a module instance
pub fn configure_module(
    instance: &mut ModuleInstance,
    config: &Configuration,
    context: &Context,
) -> CheckstyleResult<()> {
    match instance {
        ModuleInstance::FileSetCheck(check) => {
            check.contextualize(context)?;
            check.configure(config)?;
        }
        ModuleInstance::Check(check) => {
            let mut check_guard = check.lock().unwrap();
            check_guard.contextualize(context)?;
            check_guard.configure(config)?;
        }
    }
    Ok(())
}
