# Architecture - jbuild

This document describes the architecture of jbuild, a high-performance Rust implementation supporting both Maven and Gradle build systems.

## Overview

jbuild is a single-crate Rust implementation that supports both Maven and Gradle build systems, organized into logical modules under `src/`. The architecture follows the core principles of both build systems while leveraging Rust's type safety and performance.

## Module Structure

See [ORGANIZATION.md](ORGANIZATION.md) for detailed organization documentation.

```
src/
├── lib.rs              # Library root, exports all public APIs
├── main.rs             # CLI entry point
│
├── build/              # Build system abstraction layer
│   ├── detection.rs    # Build system detection (Maven vs Gradle)
│   └── executor.rs     # Generic build executor trait
│
├── maven/              # Maven-specific implementation
│   ├── model/          # Maven POM model
│   ├── core/           # Maven execution engine
│   ├── settings/       # Maven settings.xml
│   └── plugin/         # Maven plugins
│
├── gradle/             # Gradle-specific implementation (future)
│   ├── model/          # Gradle build script model
│   └── core/           # Gradle task execution
│
├── model/              # Maven POM model (backward compatibility)
│   ├── mod.rs
│   ├── model.rs        # Core Model structure (Maven POM)
│   ├── parent.rs        # Parent POM reference
│   ├── dependency.rs    # Dependency definitions
│   ├── build.rs         # Build configuration
│   ├── repository.rs   # Repository definitions
│   ├── profile.rs       # Build profiles
│   ├── distribution.rs  # Distribution management
│   ├── parser.rs        # POM XML parser with namespace handling
│   ├── model_builder.rs # Effective model construction
│   ├── effective_model.rs # Effective model with parent resolution
│   ├── profile_activator.rs # Profile activation logic
│   ├── interpolation.rs # Property interpolation
│   ├── validator.rs     # Model validation
│   └── gradle.rs        # Gradle build script parsing (in progress)
├── artifact/           # Artifact handling
│   ├── mod.rs
│   ├── artifact.rs     # Artifact representation
│   ├── coordinates.rs   # GAV coordinates
│   ├── handler.rs       # Artifact type handlers
│   └── repository.rs    # Local repository interface
├── core/               # Core execution engine
│   ├── mod.rs
│   ├── execution.rs     # Execution request/result
│   ├── lifecycle.rs     # Lifecycle phases (Maven)
│   ├── project.rs       # MavenProject representation
│   ├── session.rs       # MavenSession management
│   ├── project_builder.rs # Builds projects from POMs
│   ├── lifecycle_starter.rs # Starts lifecycle execution
│   ├── lifecycle_executor.rs # Executes lifecycle phases
│   ├── mojo_executor.rs # Executes plugin mojos
│   ├── reactor.rs       # Multi-module reactor
│   ├── default_maven.rs # Main execution engine
│   ├── graph_builder.rs # Dependency graph construction
│   ├── goal_parser.rs  # Goal parsing and phase mapping
│   └── optimization.rs # Build optimization (caching, parallel execution)
├── resolver/           # Dependency resolution
│   ├── mod.rs
│   ├── repository.rs    # Remote repository
│   ├── resolver.rs       # Dependency resolver
│   ├── metadata.rs      # Repository metadata
│   ├── transitive.rs    # Transitive dependency resolution
│   ├── downloader.rs   # Artifact downloader (HTTP)
│   └── advanced.rs      # Advanced dependency resolution (ranges, conflicts, exclusions)
├── settings/           # Settings management
│   ├── mod.rs
│   ├── settings.rs      # Settings structure
│   ├── profile.rs       # Settings profiles
│   ├── server.rs        # Server configuration
│   ├── mirror.rs        # Repository mirrors
│   └── parser.rs        # Settings.xml parser
├── plugin_api/         # Plugin API
│   ├── mod.rs
│   ├── mojo.rs         # Mojo interface
│   ├── plugin.rs       # Plugin trait
│   ├── descriptor.rs   # Plugin descriptor
│   ├── registry.rs     # Plugin registry and loading
│   └── compatibility.rs # Plugin compatibility and configuration inheritance
├── compiler/           # Java compiler integration
│   ├── mod.rs
│   ├── java_compiler.rs # Java compiler invocation
│   ├── classpath.rs     # Classpath management
│   └── source_discovery.rs # Source file discovery
├── packaging/          # JAR/WAR packaging
│   ├── mod.rs
│   ├── jar.rs          # JAR file creation
│   ├── war.rs          # WAR file packaging
│   ├── manifest.rs     # Manifest generation
│   └── resources.rs    # Resource handling
└── testing/            # Test execution
    ├── mod.rs
    ├── discovery.rs    # Test class discovery
    ├── runner.rs       # Test runner
    └── reporting.rs    # Test reporting
```

## Core Components

### Execution Flow

```
CLI (main.rs)
  ↓
BuildSystemDetection (Maven/Gradle)
  ↓
MavenExecutionRequest / GradleExecutionRequest
  ↓
DefaultMaven.execute() / GradleExecutor.execute()
  ↓
ProjectBuilder.build_reactor()
  ↓
Reactor (dependency graph)
  ↓
LifecycleStarter.execute()
  ↓
LifecycleExecutor.execute_to_phase()
  ↓
MojoExecutor.execute()
  ↓
MavenExecutionResult / GradleExecutionResult
```

### Key Design Decisions

1. **Single Crate**: All code in one crate for simplicity and faster compilation
2. **Module Organization**: Logical grouping matching Maven's Java structure, with Gradle support integrated
3. **Error Handling**: Uses `anyhow` for application errors, `thiserror` for library errors
4. **Async Support**: `tokio` for I/O operations (future use for parallel builds)
5. **Type Safety**: Strong typing for coordinates, phases, and configurations
6. **Compiler Integration**: Native Rust implementation for Java compiler invocation with classpath management
7. **Dual Build System Support**: Unified artifact and dependency resolution for both Maven and Gradle

### Data Flow

#### Project Building
1. Detect build system (pom.xml or build.gradle)
2. Parse build file → `Model` (Maven) or `GradleModel` (Gradle)
3. Build effective model (inherit from parent) → `Model`
4. Create `MavenProject` or `GradleProject` with resolved paths
5. Build dependency graph for reactor

#### Dependency Resolution
1. Check local repository (shared between Maven and Gradle)
2. If not found, download from remote repositories via HTTP
3. Resolve transitive dependencies recursively
4. Cache resolved artifacts
5. Store downloaded artifacts in local repository

#### Lifecycle Execution
1. Parse goals → map to lifecycle phases (Maven) or tasks (Gradle)
2. For each phase/task up to target:
   - Get plugin bindings for phase
   - Execute mojos/tasks in order
   - Handle failures appropriately

### Extension Points

- **Plugin API**: Trait-based for future plugin loading
- **Repository**: Trait-based for custom repository implementations
- **Artifact Handler**: Trait-based for custom packaging types
- **Build System**: Trait-based for adding new build system support

## Dependencies

### Core Dependencies
- `serde` / `serde_json` - Serialization
- `quick-xml` - XML parsing (Maven POM)
- `anyhow` / `thiserror` - Error handling
- `clap` - CLI parsing
- `tracing` - Structured logging
- `tokio` - Async runtime
- `reqwest` - HTTP client (for remote repositories)
- `url` - URL handling
- `zip` - JAR file handling for plugins and packaging
- `walkdir` - File system traversal for source discovery
- `which` - Finding executables (javac, mvn, java, gradle) in PATH
- `glob` - Pattern matching for resource filtering
- `jni` - JNI for Java integration (optional feature)

## Testing & Testability

### Trait-Based Design
The codebase uses trait-based abstractions to enable dependency injection and testing:

- `ProjectBuildStrategy`: Trait for building Maven/Gradle projects
- `LifecycleExecutionStrategy`: Trait for executing lifecycle phases
- `DependencyResolutionStrategy`: Trait for resolving dependencies
- `ArtifactRepository`: Trait for artifact repository operations

### Testing Utilities
Located in `src/testing_utils.rs`:

- `MockArtifactRepository`: In-memory artifact repository for isolated testing
- `MockDependencyResolver`: Mock dependency resolver with configurable behavior
- `TestProjectBuilder`: Fluent builder for creating test projects

### Builder Patterns
Complex objects use fluent builders for easier construction:

- `ExecutionRequestBuilder`: Builds `MavenExecutionRequest` with fluent API

### Error Handling
Custom error types in `src/error.rs` provide:

- Specific error variants for different failure modes
- Better error messages and context
- Conversion from standard library errors

## Future Architecture Considerations

1. **Gradle Support**: Full Gradle build script parsing and execution
   - Gradle build script parsing (Groovy/Kotlin DSL)
   - Gradle task execution
   - Gradle dependency resolution
2. **Plugin System**: Full Java plugin execution
   - JNI integration available (optional `jni` feature) for direct Java class loading
   - External Maven/Gradle process fallback for plugin execution
   - Framework ready for complete Mojo execution implementation
3. **Parallel Execution**: Use tokio for parallel project builds
4. **Caching**: Build result caching for incremental builds
5. **Incremental Compilation**: Track file changes for faster rebuilds
6. **Annotation Processing**: Support for Java annotation processors
7. **Test Execution**: Integration with JUnit/TestNG test runners
8. **Deterministic Snapshots**: Use `IndexMap` for deterministic HashMap ordering in snapshot tests

## Performance Considerations

- Single crate reduces compilation time
- Zero-cost abstractions where possible
- Efficient dependency graph algorithms
- Lazy loading of POMs/Gradle files and artifacts
- Caching of resolved dependencies
- Rust's performance advantages over Java-based build tools

## Compatibility

The implementation aims for compatibility with:
- Maven POM 4.0.0 format
- Standard Maven lifecycle
- Maven repository layout
- Maven plugin API (where possible in Rust)
- Gradle build scripts (Groovy/Kotlin DSL) - in progress
- Gradle dependency management - in progress
