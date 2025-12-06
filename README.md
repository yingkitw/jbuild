# jbuild

A high-performance Rust implementation of Java build tools, supporting both **Maven** and **Gradle** while leveraging Rust's performance and safety guarantees.

## Overview

jbuild provides a complete Rust implementation of Java build systems, maintaining compatibility with:
- **Maven**: Project Object Model (POM) and build lifecycle
- **Gradle**: Build scripts and dependency management

The tool aims to provide faster builds through Rust's performance while maintaining full compatibility with existing Java build configurations.

## Comparison: jbuild vs Maven vs Gradle

| Feature | jbuild | Maven | Gradle |
|---------|--------|-------|--------|
| **Language** | Rust | Java | Groovy/Kotlin (JVM) |
| **Startup Time** | ~10ms | ~500ms | ~1000ms |
| **Memory Usage** | Low (~50MB) | High (~200MB+) | High (~300MB+) |
| **Build File** | pom.xml / build.gradle | pom.xml | build.gradle(.kts) |
| **Dependency Resolution** | Shared resolver | Maven Resolver | Gradle Resolver |
| **Parallel Builds** | Native async (tokio) | Limited | Task-level |
| **Incremental Builds** | ✅ Built-in | ✅ Plugin-based | ✅ Native |
| **Build Cache** | ✅ Local | ❌ (requires plugin) | ✅ Local + Remote |
| **Multi-module** | ✅ Both systems | ✅ Reactor | ✅ Composite builds |
| **Plugin Ecosystem** | Maven plugins (via fallback) | Extensive | Extensive |
| **Configuration** | XML / Groovy DSL | XML only | Groovy/Kotlin DSL |

### Key Advantages of jbuild

1. **Performance**: Native Rust binary with no JVM startup overhead
2. **Memory Efficiency**: Significantly lower memory footprint
3. **Unified Tool**: Single binary supports both Maven and Gradle projects
4. **Modern Architecture**: Async I/O, parallel execution, trait-based design
5. **Cross-Platform**: Native binaries for all major platforms

### When to Use Each Tool

| Use Case | Recommended Tool |
|----------|------------------|
| New projects prioritizing build speed | jbuild |
| Existing Maven projects | jbuild or Maven |
| Existing Gradle projects | jbuild or Gradle |
| Complex plugin requirements | Maven or Gradle |
| CI/CD with memory constraints | jbuild |
| Android development | Gradle |
| Enterprise with existing tooling | Maven or Gradle |

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
- ✅ **Gradle dependency resolution** (integrated with shared resolver)
- ✅ **Multi-project builds** (settings.gradle support)
- ✅ **148 tests passing** (unit, integration, multi-module)

See [TODO.md](TODO.md) for the current list of remaining work items and [MIGRATION.md](MIGRATION.md) for migration details.

## Example Projects

The `examples/` directory contains sample projects demonstrating jbuild capabilities:

### Maven Examples

```
examples/
├── simple-java-project/     # Single-module Maven project
│   ├── pom.xml
│   └── src/main/java/...
└── multi-module-maven/      # Multi-module Maven project
    ├── pom.xml              # Parent POM
    ├── core/                # Core utilities module
    ├── api/                 # API interfaces module
    └── app/                 # Application module
```

**Sample Maven POM (`examples/simple-java-project/pom.xml`):**
```xml
<project>
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>simple-java-project</artifactId>
    <version>1.0.0</version>
    <packaging>jar</packaging>
    
    <properties>
        <maven.compiler.source>11</maven.compiler.source>
        <maven.compiler.target>11</maven.compiler.target>
    </properties>
    
    <dependencies>
        <dependency>
            <groupId>junit</groupId>
            <artifactId>junit</artifactId>
            <version>4.13.2</version>
            <scope>test</scope>
        </dependency>
    </dependencies>
</project>
```

### Gradle Examples

```
examples/
├── simple-gradle-project/   # Single-module Gradle project
│   ├── build.gradle
│   └── src/main/java/...
└── multi-module-gradle/     # Multi-module Gradle project
    ├── settings.gradle      # Project settings
    ├── build.gradle         # Root build script
    ├── core/                # Core utilities module
    ├── api/                 # API interfaces module
    └── app/                 # Application module
```

**Sample Gradle build script (`examples/simple-gradle-project/build.gradle`):**
```groovy
plugins {
    id 'java'
}

group = 'com.example'
version = '1.0.0'

sourceCompatibility = '11'
targetCompatibility = '11'

repositories {
    mavenCentral()
}

dependencies {
    testImplementation 'junit:junit:4.13.2'
}
```

**Sample settings.gradle (`examples/multi-module-gradle/settings.gradle`):**
```groovy
rootProject.name = 'multi-module-gradle'

include ':core'
include ':api'
include ':app'
```

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
jbuild --file pom.xml test
jbuild --file pom.xml package

# For Gradle projects
jbuild --file build.gradle build
jbuild --file build.gradle clean
jbuild --file build.gradle test
```

### Supported Tasks/Goals

| Maven Goal | Gradle Task | Description |
|------------|-------------|-------------|
| `compile` | `compileJava` | Compile Java sources |
| `test-compile` | `compileTestJava` | Compile test sources |
| `test` | `test` | Run tests |
| `package` | `jar` | Create JAR file |
| `clean` | `clean` | Clean build outputs |
| `install` | - | Install to local repository |
| - | `build` | Full build (compile + test + jar) |

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
