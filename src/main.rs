use jbuild::cli::{Cli, Commands};
use std::path::PathBuf;

use jbuild::build::{BuildSystem, BuildExecutor, ExecutionRequest, GoalMapper};
use jbuild::maven::core::MavenBuildExecutor;
use jbuild::gradle::core::GradleExecutor;
use jbuild::checkstyle::{Checker, ConfigurationLoader, DefaultLogger};
use jbuild::ui::{success as ui_success, error as ui_error, warn as ui_warn, build_success, build_failure};
use jbuild::config::JbuildConfig;
use jbuild::runner::*;
use clap::Parser;

/// jbuild - A high-performance build tool for Java projects (Maven & Gradle)
// Cli and Commands moved to jbuild::cli

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::New { name, template, build_system }) => return run_new(name, template, build_system),
        Some(Commands::Init { build_system }) => return run_init(build_system),
        Some(Commands::Add { dependency, dev }) => return run_add(dependency, *dev),
        Some(Commands::Remove { dependency }) => return run_remove(dependency),
        Some(Commands::Tree) => return run_tree(),
        Some(Commands::Search { query, limit }) => return run_search(query, *limit),
        Some(Commands::Info { package }) => return run_info(package),
        Some(Commands::Update { dependency }) => return run_update(dependency.as_deref()),
        Some(Commands::Run { args, main_class, example }) => return run_run(args.clone(), main_class.clone(), example.clone()),
        Some(Commands::Fmt { files, check }) => return run_fmt(files.clone(), *check),
        Some(Commands::Doc { open, output }) => return run_doc(*open, output.clone()),
        Some(Commands::Audit) => return run_audit(),
        Some(Commands::Watch { test, watch_paths }) => return run_watch(*test, watch_paths.clone()),
        Some(Commands::Completions { shell }) => return run_completions(*shell),
        Some(Commands::WorkspaceNew { name }) => return run_workspace_new(name),
        Some(Commands::WorkspaceAdd { path }) => return run_workspace_add(path),
        Some(Commands::WorkspaceRemove { path }) => return run_workspace_remove(path),
        Some(Commands::WorkspaceList) => return run_workspace_list(),
        Some(Commands::WorkspaceBuild { goals }) => return run_workspace_build(goals.clone()),
        Some(Commands::Lint { config, files }) => return run_lint_internal(config.as_ref(), files),
        _ => {}
    }

    let base_dir = if let Some(file) = &cli.file {
        file.parent().unwrap_or_else(|| std::path::Path::new(".")).to_path_buf()
    } else {
        std::env::current_dir()?
    };

    ensure_pom_from_jbuild(&base_dir)?;

    let build_system = BuildSystem::detect(&base_dir)
        .ok_or_else(|| anyhow::anyhow!("No build system detected"))?;
    
    let goals = if !cli.goals.is_empty() {
        cli.goals.clone()
    } else {
        match &cli.command {
            Some(Commands::Validate) => vec!["validate".to_string()],
            Some(Commands::Compile) => vec!["compile".to_string()],
            Some(Commands::Test) => vec!["test".to_string()],
            Some(Commands::Package) => vec!["package".to_string()],
            Some(Commands::Install) => vec!["install".to_string()],
            Some(Commands::Deploy) => vec!["deploy".to_string()],
            Some(Commands::Clean) => vec!["clean".to_string()],
            Some(Commands::Build) => vec!["build".to_string()],
            Some(Commands::Check) => vec!["check".to_string()],
            None => vec!["compile".to_string()],
            _ => unreachable!(),
        }
    };

    let goal_mapper = GoalMapper::new();
    let mapped_goals = match build_system {
        BuildSystem::Maven => {
            if goals.contains(&"check".to_string()) {
                goals.iter().flat_map(|g| if g == "check" { vec!["compile".to_string(), "test-compile".to_string()] } else { vec![g.clone()] }).collect()
            } else { goals.clone() }
        }
        BuildSystem::Gradle => {
            goals.iter().flat_map(|g| if GoalMapper::is_lifecycle_phase(g) { goal_mapper.maven_to_gradle(g) } else { vec![g.clone()] }).collect()
        }
    };

    let mut system_properties = std::collections::HashMap::new();
    for prop in &cli.define {
        if let Some((key, value)) = prop.split_once('=') {
            system_properties.insert(key.to_string(), value.to_string());
        }
    }

    let request = ExecutionRequest {
        base_directory: base_dir.clone(),
        goals: mapped_goals,
        system_properties,
        show_errors: cli.show_errors,
        offline: cli.offline,
    };

    let executor: Box<dyn BuildExecutor> = match build_system {
        BuildSystem::Maven => Box::new(MavenBuildExecutor::new()),
        BuildSystem::Gradle => Box::new(GradleExecutor::new()),
    };

    match executor.execute(request) {
        Ok(result) => {
            if result.success {
                build_success();
                Ok(())
            } else {
                build_failure();
                for err in &result.errors { ui_error(err); }
                std::process::exit(1);
            }
        }
        Err(e) => {
            ui_error(&format!("Build failed: {e}"));
            std::process::exit(1);
        }
    }
}

fn run_lint_internal(config_file: Option<&PathBuf>, files: &[PathBuf]) -> anyhow::Result<()> {
    let base_dir = std::env::current_dir()?;
    let files_to_check = if files.is_empty() {
        let mut collected = Vec::new();
        for dir in &["src/main/java", "src/test/java"] {
            let path = base_dir.join(dir);
            if path.exists() { collected.extend(collect_java_files(&path)?); }
        }
        collected
    } else {
        let mut collected = Vec::new();
        for path in files {
            let full_path = if path.is_absolute() { path.clone() } else { base_dir.join(path) };
            if full_path.is_dir() { collected.extend(collect_java_files(&full_path)?); }
            else if full_path.extension().is_some_and(|e| e == "java") { collected.push(full_path); }
        }
        collected
    };

    if files_to_check.is_empty() {
        ui_warn("No Java files found to check");
        return Ok(());
    }
    
    let config = if let Some(path) = config_file { ConfigurationLoader::load_configuration(path)? }
    else { ConfigurationLoader::create_default_configuration() };

    let mut checker = Checker::new();
    checker.configure(&config)?;
    checker.add_listener(Box::new(DefaultLogger::new()));

    let error_count = checker.process(&files_to_check)?;
    if error_count > 0 {
        ui_error(&format!("Checkstyle found {error_count} error(s)"));
        std::process::exit(1);
    } else {
        ui_success("Checkstyle completed with no errors");
        Ok(())
    }
}

fn collect_java_files(dir: &std::path::Path) -> anyhow::Result<Vec<PathBuf>> {
    use walkdir::WalkDir;
    let mut files = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == "java") {
            files.push(path.to_path_buf());
        }
    }
    Ok(files)
}

fn ensure_pom_from_jbuild(base_dir: &std::path::Path) -> anyhow::Result<()> {
    let jbuild_path = base_dir.join("jbuild.toml");
    if base_dir.join("pom.xml").exists() || base_dir.join("build.gradle").exists() || !jbuild_path.exists() {
        return Ok(());
    }
    let cfg = JbuildConfig::from_file(&jbuild_path)?;
    std::fs::write(base_dir.join("pom.xml"), cfg.to_pom_xml())?;
    Ok(())
}
