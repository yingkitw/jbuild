//! Gradle Task Execution
//!
//! Implements execution of standard Gradle tasks.

use std::path::PathBuf;
use anyhow::{Context, Result};
use crate::gradle::model::GradleProject;

/// Execute clean task - removes build directory
pub fn execute_clean(project: &GradleProject) -> Result<()> {
    let build_dir = project.base_dir.join("build");
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir)
            .with_context(|| format!("Failed to remove build directory: {build_dir:?}"))?;
        tracing::info!("Cleaned build directory");
    }
    Ok(())
}

/// Execute compile Java task
pub fn execute_compile_java(project: &GradleProject, resolve_dep: impl Fn(&str, &str, Option<&str>) -> Result<Option<PathBuf>>) -> Result<()> {
    use crate::compiler::java_compiler::{JavaCompiler, CompilerConfig};
    use crate::compiler::classpath::ClasspathBuilder;
    
    let source_dir = project.base_dir.join("src/main/java");
    let output_dir = project.base_dir.join("build/classes/java/main");
    
    std::fs::create_dir_all(&output_dir)?;
    
    let mut classpath = ClasspathBuilder::new();
    
    // Add dependencies to classpath
    for dep in &project.dependencies {
        if dep.configuration == "implementation" || dep.configuration == "compile" {
            if let (Some(group), Some(artifact)) = (&dep.group, &dep.artifact) {
                if let Ok(Some(jar_path)) = resolve_dep(group, artifact, dep.version.as_deref()) {
                    classpath = classpath.add_jar(jar_path);
                }
            }
        }
    }
    
    // Build compiler config
    let mut config = CompilerConfig::new(output_dir.clone())
        .with_source_roots(vec![source_dir])
        .with_classpath(classpath);
    
    if let Some(ref source_compat) = project.source_compatibility {
        config = config.with_source_version(source_compat.clone());
    }
    if let Some(ref target_compat) = project.target_compatibility {
        config = config.with_target_version(target_compat.clone());
    }
    
    let result = JavaCompiler::compile(&config)?;
    
    if !result.success {
        return Err(anyhow::anyhow!("Compilation failed: {}", result.error_output));
    }
    
    tracing::info!("Compiled {} Java source file(s)", result.compiled_files);
    Ok(())
}

/// Execute compile test Java task
pub fn execute_compile_test_java(project: &GradleProject) -> Result<()> {
    use crate::compiler::java_compiler::{JavaCompiler, CompilerConfig};
    use crate::compiler::classpath::ClasspathBuilder;
    
    let source_dir = project.base_dir.join("src/test/java");
    let output_dir = project.base_dir.join("build/classes/java/test");
    let main_classes_dir = project.base_dir.join("build/classes/java/main");
    
    std::fs::create_dir_all(&output_dir)?;
    
    let mut classpath = ClasspathBuilder::new();
    if main_classes_dir.exists() {
        classpath = classpath.add_directory(main_classes_dir);
    }
    
    let mut config = CompilerConfig::new(output_dir.clone())
        .with_source_roots(vec![source_dir])
        .with_classpath(classpath);
    
    if let Some(ref source_compat) = project.source_compatibility {
        config = config.with_source_version(source_compat.clone());
    }
    if let Some(ref target_compat) = project.target_compatibility {
        config = config.with_target_version(target_compat.clone());
    }
    
    let result = JavaCompiler::compile_tests(&config)?;
    
    if !result.success {
        return Err(anyhow::anyhow!("Test compilation failed: {}", result.error_output));
    }
    
    tracing::info!("Compiled {} test Java source file(s)", result.compiled_files);
    Ok(())
}

/// Execute test task
pub fn execute_test(project: &GradleProject) -> Result<()> {
    // First compile test sources
    execute_compile_test_java(project)?;
    
    use crate::testing::runner::TestRunner;
    use crate::testing::discovery::TestDiscovery;
    
    let test_classes_dir = project.base_dir.join("build/classes/java/test");
    let main_classes_dir = project.base_dir.join("build/classes/java/main");
    
    let test_classes = TestDiscovery::discover_tests(&test_classes_dir)?;
    
    if test_classes.is_empty() {
        tracing::info!("No test classes found");
        return Ok(());
    }
    
    let mut runner = TestRunner::new(test_classes_dir.clone());
    runner = runner.add_to_classpath(main_classes_dir);
    
    let mut all_passed = true;
    for test_class in &test_classes {
        let result = runner.run_test(test_class)?;
        if !result.success {
            all_passed = false;
            tracing::error!("Test {} failed: {}", test_class.class_name, result.error_output);
        } else {
            tracing::info!(
                "Test {} passed: {}/{} tests",
                test_class.class_name,
                result.tests_passed,
                result.tests_run
            );
        }
    }
    
    if !all_passed {
        return Err(anyhow::anyhow!("Some tests failed"));
    }
    
    tracing::info!("All tests passed");
    Ok(())
}

/// Execute jar task
pub fn execute_jar(project: &GradleProject) -> Result<()> {
    use crate::packaging::jar::JarBuilder;
    use crate::packaging::resources::ResourceFilter;
    
    let classes_dir = project.base_dir.join("build/classes/java/main");
    let resources_dir = project.base_dir.join("src/main/resources");
    let output_jar = project.base_dir.join(format!(
        "build/libs/{}-{}.jar",
        project.name,
        project.version.as_deref().unwrap_or("unspecified")
    ));
    
    std::fs::create_dir_all(output_jar.parent().unwrap())?;
    
    let mut jar_builder = JarBuilder::new().with_classes_dir(classes_dir);
    
    if resources_dir.exists() {
        jar_builder = jar_builder.add_resources_from_dir(
            resources_dir,
            None,
            ResourceFilter::default(),
        )?;
    }
    
    jar_builder.build(&output_jar)?;
    
    tracing::info!("Created JAR: {:?}", output_jar);
    Ok(())
}

/// Execute build task (compile + test + jar)
pub fn execute_build(project: &GradleProject, resolve_dep: impl Fn(&str, &str, Option<&str>) -> Result<Option<PathBuf>>) -> Result<()> {
    execute_compile_java(project, resolve_dep)?;
    execute_test(project)?;
    execute_jar(project)?;
    Ok(())
}
