# mvn-rs

A Rust implementation of Apache Maven, a software project management and comprehension tool.

## Overview

This project aims to provide a complete Rust implementation of Apache Maven, maintaining compatibility with Maven's Project Object Model (POM) and build lifecycle while leveraging Rust's performance and safety guarantees.

## Project Structure

The project is organized as a **single crate** with all modules under `src/`:

- **model/**: POM file parsing and data structures
- **artifact/**: Artifact handling and coordinates
- **core/**: Core execution engine and lifecycle
- **resolver/**: Dependency resolution
- **settings/**: Settings and configuration management
- **plugin_api/**: Plugin API definitions
- **compiler/**: Java compiler integration (javac invocation, classpath management)
- **packaging/**: JAR/WAR file creation and packaging
- **testing/**: Test discovery, execution, and reporting
- **main.rs**: Command-line interface

## Status

This is an ongoing migration project. Core functionality is being implemented incrementally.

**Key Features Implemented:**
- ✅ POM parsing and model building
- ✅ Dependency resolution (local and remote repositories)
- ✅ Plugin loading and descriptor parsing
- ✅ Plugin dependency resolution
- ✅ Java compiler integration
- ✅ Lifecycle execution framework
- ✅ Plugin execution framework (with classpath setup)
- ✅ JNI integration for Java plugin execution (optional feature)
- ✅ External Maven process fallback for plugin execution
- ✅ JAR/WAR file packaging with manifest generation
- ✅ Test discovery and execution (JUnit, TestNG)
- ✅ Test reporting
- ✅ Profile activation logic
- ✅ Property interpolation
- ✅ Model validation
- ✅ Advanced dependency resolution (version ranges, conflicts, exclusions)
- ✅ Build optimization (incremental compilation, parallel execution)
- ✅ Plugin compatibility and configuration inheritance

See [TODO.md](TODO.md) for the current list of remaining work items and [MIGRATION.md](MIGRATION.md) for migration details.

## Building

```bash
# Build without JNI (uses external Maven process for plugins)
cargo build --release

# Build with JNI support (enables direct Java class loading)
cargo build --release --features jni
```

## Running

```bash
cargo run --bin mvn -- [maven-args]
```

## Testing

The project includes comprehensive unit tests and integration tests:

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run integration tests
cargo test --test example_project

# Run with output
cargo test -- --nocapture
```

### Test Structure

- **Unit tests**: Located in `src/` modules with `#[test]` attributes
- **Integration tests**: Located in `tests/` directory
- **Mocks and fixtures**: `src/testing_utils.rs` provides mock implementations for testing

### Testing Utilities

The codebase provides several testing utilities in `testing_utils`:

- `MockArtifactRepository`: In-memory artifact repository for testing
- `MockDependencyResolver`: Mock dependency resolver for controlled testing
- `TestProjectBuilder`: Fluent builder for creating test projects

Example usage:

```rust
use mvn_rs::{MockArtifactRepository, ArtifactRepository};

#[test]
fn test_artifact_resolution() {
    let repo = MockArtifactRepository::new();
    repo.add_artifact("com.example", "lib", "1.0.0", "/path/to/lib.jar".to_string());
    
    assert!(repo.exists("com.example", "lib", "1.0.0"));
}
```

## Architecture & Design

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed architecture documentation.

### Key Design Patterns

- **Trait-based abstractions**: Core components use traits for testability
- **Builder patterns**: Complex objects use fluent builders
- **Custom error types**: Comprehensive error handling with `MavenError`
- **Dependency injection**: Supports mock implementations for testing

## License

Licensed under the Apache License, Version 2.0.

