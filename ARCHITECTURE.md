# Architecture - mvn-rs

This document describes the architecture of the Rust implementation of Apache Maven.

## Overview

mvn-rs is a single-crate Rust implementation of Apache Maven, organized into logical modules under `src/`. The architecture follows Maven's core principles while leveraging Rust's type safety and performance.

## Module Structure

```
src/
├── lib.rs              # Library root, exports all public APIs
├── main.rs             # CLI entry point
├── model/              # POM (Project Object Model) structures
│   ├── mod.rs
│   ├── model.rs        # Core Model structure
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
│   └── validator.rs     # Model validation
├── artifact/           # Artifact handling
│   ├── mod.rs
│   ├── artifact.rs     # Artifact representation
│   ├── coordinates.rs   # GAV coordinates
│   ├── handler.rs       # Artifact type handlers
│   └── repository.rs    # Local repository interface
├── core/               # Core execution engine
│   ├── mod.rs
│   ├── execution.rs     # Execution request/result
│   ├── lifecycle.rs     # Lifecycle phases
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
MavenExecutionRequest
  ↓
DefaultMaven.execute()
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
MavenExecutionResult
```

### Key Design Decisions

1. **Single Crate**: All code in one crate for simplicity and faster compilation
2. **Module Organization**: Logical grouping matching Maven's Java structure
3. **Error Handling**: Uses `anyhow` for application errors, `thiserror` for library errors
4. **Async Support**: `tokio` for I/O operations (future use for parallel builds)
5. **Type Safety**: Strong typing for coordinates, phases, and configurations
6. **Compiler Integration**: Native Rust implementation for Java compiler invocation with classpath management

### Data Flow

#### Project Building
1. Parse POM XML → `Model`
2. Build effective model (inherit from parent) → `Model`
3. Create `MavenProject` with resolved paths
4. Build dependency graph for reactor

#### Dependency Resolution
1. Check local repository
2. If not found, download from remote repositories via HTTP
3. Resolve transitive dependencies recursively
4. Cache resolved artifacts
5. Store downloaded artifacts in local repository

#### Lifecycle Execution
1. Parse goals → map to lifecycle phases
2. For each phase up to target:
   - Get plugin bindings for phase
   - Execute mojos in order
   - Handle failures appropriately

### Extension Points

- **Plugin API**: Trait-based for future plugin loading
- **Repository**: Trait-based for custom repository implementations
- **Artifact Handler**: Trait-based for custom packaging types

## Dependencies

### Core Dependencies
- `serde` / `serde_json` - Serialization
- `quick-xml` - XML parsing
- `anyhow` / `thiserror` - Error handling
- `clap` - CLI parsing
- `tracing` - Structured logging
- `tokio` - Async runtime
- `reqwest` - HTTP client (for remote repositories)
- `url` - URL handling
- `zip` - JAR file handling for plugins and packaging
- `walkdir` - File system traversal for source discovery
- `which` - Finding executables (javac, mvn, java) in PATH
- `glob` - Pattern matching for resource filtering
- `jni` - JNI for Java integration (optional feature)

## Future Architecture Considerations

1. **Plugin System**: Full Java plugin execution
   - JNI integration available (optional `jni` feature) for direct Java class loading
   - External Maven process fallback for plugin execution
   - Framework ready for complete Mojo execution implementation
2. **Parallel Execution**: Use tokio for parallel project builds
3. **Caching**: Build result caching for incremental builds
4. **Incremental Compilation**: Track file changes for faster rebuilds
5. **Annotation Processing**: Support for Java annotation processors
6. **Test Execution**: Integration with JUnit/TestNG test runners

## Performance Considerations

- Single crate reduces compilation time
- Zero-cost abstractions where possible
- Efficient dependency graph algorithms
- Lazy loading of POMs and artifacts
- Caching of resolved dependencies

## Compatibility

The implementation aims for compatibility with:
- Maven POM 4.0.0 format
- Standard Maven lifecycle
- Maven repository layout
- Maven plugin API (where possible in Rust)

