# jbuild

A high-performance Rust implementation of Java build tools, supporting both **Maven** and **Gradle** while leveraging Rust's performance and safety guarantees.

## Overview

jbuild provides a complete Rust implementation of Java build systems, maintaining compatibility with:
- **Maven**: Project Object Model (POM) and build lifecycle
- **Gradle**: Build scripts and dependency management

The tool aims to provide faster builds through Rust's performance while maintaining full compatibility with existing Java build configurations.

## Project Structure

The project is organized as a **single crate** with all modules under `src/`:

- **model/**: POM/Gradle file parsing and data structures
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

This is an ongoing project. Both Maven and Gradle support are implemented with shared infrastructure.

**Key Features Implemented:**
- ✅ Maven POM parsing and model building
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
- ✅ **Gradle build script parsing (Groovy/Kotlin DSL)**
- ✅ **Gradle task execution (clean, compileJava, test, jar, build)**
- ✅ **Build system detection and unified CLI**
- 🚧 Gradle dependency resolution integration (in progress)

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
# For Maven projects
jbuild --file pom.xml compile

# For Gradle projects (coming soon)
jbuild --file build.gradle build
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
use jbuild::{MockArtifactRepository, ArtifactRepository};

#[test]
fn test_artifact_resolution() {
    let repo = MockArtifactRepository::new();
    repo.add_artifact("com.example", "lib", "1.0.0", "/path/to/lib.jar".to_string());
    
    assert!(repo.exists("com.example", "lib", "1.0.0"));
}
```

## Architecture & Design

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed architecture documentation.

The architecture is inspired by Gradle's platform-based design. See [GRADLE_LEARNINGS.md](GRADLE_LEARNINGS.md) for architectural patterns we're adopting.

### Key Design Patterns

- **Trait-based abstractions**: Core components use traits for testability
- **Builder patterns**: Complex objects use fluent builders
- **Custom error types**: Comprehensive error handling with `MavenError`
- **Dependency injection**: Supports mock implementations for testing

## License

Licensed under the Apache License, Version 2.0.
