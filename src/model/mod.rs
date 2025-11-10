pub mod model;
pub mod parent;
pub mod dependency;
pub mod build;
pub mod repository;
pub mod profile;
pub mod distribution;
pub mod parser;
pub mod model_builder;
pub mod effective_model;
pub mod profile_activator;
pub mod interpolation;
pub mod validator;

pub use model::{Model, Organization, License, Developer, Contributor, Reporting, ReportingPlugin};
pub use parent::Parent;
pub use dependency::{Dependency, Exclusion};
pub use build::{Build, Extension, Resource, Plugin, Execution, PluginManagement};
pub use repository::{Repository, RepositoryPolicy};
pub use profile::{Profile, Activation, ActivationOS, ActivationProperty, ActivationFile};
pub use distribution::{DistributionManagement, DeploymentRepository, Site, Relocation};
pub use parser::{parse_pom, parse_pom_file, normalize_xml_namespaces, ParseError};
pub use model_builder::*;
pub use effective_model::*;
pub use profile_activator::{ProfileActivator, ProfileActivationContext};
pub use interpolation::PropertyInterpolator;
pub use validator::{ModelValidator, ValidationError};

