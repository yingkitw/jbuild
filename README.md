# jbuild

**Cargo for Java** - A modern, fast, and user-friendly build system for Java projects.

## The Problem

Java developers face significant friction with existing build tools:

- **Slow startup**: Maven takes 500ms+ and Gradle 1000ms+ just to start, even for simple commands
- **High memory usage**: JVM-based tools consume 200-300MB+ of RAM
- **Complex configuration**: Verbose XML (Maven) or DSL learning curve (Gradle)
- **Poor developer experience**: No simple `add dependency` command, manual XML editing required
- **Tool fragmentation**: Different commands and concepts between Maven and Gradle

**Every `mvn compile` or `gradle build` wastes seconds waiting for the JVM to start.**

## The Solution

jbuild is a **native Rust implementation** that brings the Cargo experience to Java:

| Metric | jbuild | Maven | Gradle |
|--------|--------|-------|--------|
| **Startup** | ~10ms | ~500ms | ~1000ms |
| **Memory** | ~50MB | ~200MB | ~300MB |
| **Add Dependency** | `jbuild add pkg` | Edit XML manually | Edit DSL manually |
| **Create Project** | `jbuild new app` | `mvn archetype:generate` (interactive) | `gradle init` |

## Key Advantages

1. **50x Faster Startup** - Native binary, no JVM warmup
2. **5x Less Memory** - Efficient Rust implementation
3. **Unified CLI** - Same commands for Maven and Gradle projects
4. **Modern UX** - Simple commands like `add`, `remove`, `search`, `tree`
5. **Zero Config** - Works with existing `pom.xml` or `build.gradle`
6. **Code Quality Built-in** - Integrated Checkstyle linting

## Quick Start

```bash
# Install (coming soon)
cargo install jbuild

# Create a new project
jbuild new my-app
cd my-app

# Build and run
jbuild build
jbuild run                    # Auto-detect and run main class
jbuild run --main-class com.example.App  # Specify main class
jbuild run arg1 arg2          # Pass arguments to application

# Manage dependencies
jbuild search slf4j              # Search Maven Central
jbuild add org.slf4j:slf4j-api:2.0.9
jbuild remove org.slf4j:slf4j-api
jbuild tree                      # Show dependency tree
jbuild info org.slf4j:slf4j-api  # Show package details
jbuild outdated                  # Check for outdated dependencies
jbuild update                    # Update all dependencies
```

# Code quality
jbuild lint                      # Run Checkstyle checks
jbuild test                      # Run tests
```

## Usage

### Project Management
```bash
jbuild new my-app                # Create new Maven project
jbuild new my-app -b gradle      # Create new Gradle project
jbuild new my-lib -t lib         # Create library project
jbuild init                      # Initialize in existing directory
```

### Building & Running
```bash
jbuild build                     # Compile + test + package
jbuild compile                   # Compile only
jbuild test                      # Run tests
jbuild run                       # Build and run main class
jbuild clean                     # Clean build outputs
```

### Dependency Management
```bash
jbuild add group:artifact              # Add dependency (auto-detects latest version)
jbuild add group:artifact:version      # Add dependency with specific version
jbuild add group:artifact --dev        # Add test dependency
jbuild remove group:artifact           # Remove dependency
jbuild tree                            # Show dependency tree
jbuild search <query>                  # Search Maven Central
jbuild info group:artifact             # Show package details and versions
jbuild outdated                        # Show outdated dependencies
jbuild update                          # Update all dependencies to latest
jbuild update group:artifact           # Update specific dependency
```

### Code Quality
```bash
jbuild lint                      # Run Checkstyle (9 checks)
jbuild lint -c checkstyle.xml    # Use custom config
jbuild lint src/main/java        # Check specific directory
```

## Comparison: jbuild vs Maven vs Gradle

| Feature | jbuild | Maven | Gradle |
|---------|--------|-------|--------|
| **Language** | Rust | Java | Groovy/Kotlin (JVM) |
| **Startup Time** | ~10ms | ~500ms | ~1000ms |
| **Memory Usage** | ~50MB | ~200MB+ | ~300MB+ |
| **Build File** | pom.xml / build.gradle | pom.xml | build.gradle(.kts) |
| **Add Dependency** | `jbuild add` | Manual XML edit | Manual DSL edit |
| **Search Packages** | `jbuild search` | ❌ | ❌ |
| **Remove Dependency** | `jbuild remove` | Manual XML edit | Manual DSL edit |
| **Dep Tree** | `jbuild tree` | `mvn dependency:tree` | `gradle dependencies` |
| **Package Info** | `jbuild info` | ❌ | ❌ |
| **Check Outdated** | `jbuild outdated` | ❌ | ❌ |
| **Update Dependencies** | `jbuild update` | Manual edit | Manual edit |
| **Linting** | `jbuild lint` | Plugin required | Plugin required |
| **Project Creation** | `jbuild new` | `mvn archetype:generate` | `gradle init` |
| **Multi-module** | ✅ Both systems | ✅ Reactor | ✅ Composite builds |
| **Incremental Builds** | ✅ Built-in | ✅ Plugin-based | ✅ Native |

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
- ✅ **Checkstyle integration** (`jbuild lint` command with 9 checks)
- ✅ **469 tests passing** (unit, checkstyle, CLI commands, integration)

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
# Modern Cargo-like commands (auto-detects build system)
jbuild build          # Compile the project
jbuild run            # Build and run main class
jbuild test           # Run tests
jbuild clean          # Clean build outputs
jbuild lint           # Check code style (Checkstyle)

# Dependency management
jbuild add org.slf4j:slf4j-api          # Add dependency (latest version)
jbuild add org.slf4j:slf4j-api:2.0.9    # Add specific version
jbuild remove org.slf4j:slf4j-api       # Remove dependency
jbuild tree                              # Show dependency tree
jbuild outdated                          # Show outdated dependencies

# Project creation
jbuild new my-app                        # Create new app project
jbuild new my-lib --template lib         # Create new library project
jbuild init                              # Initialize in existing directory

# Legacy mode (explicit build file)
jbuild --file pom.xml compile
jbuild --file build.gradle build

# Generate shell completions
jbuild completions bash > /etc/bash_completion.d/jbuild
jbuild completions zsh > ~/.zsh/completions/_jbuild
jbuild completions fish > ~/.config/fish/completions/jbuild.fish
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

### Code Quality: `jbuild lint`

The `lint` command runs Checkstyle checks on Java source files:

```bash
# Check all Java files in src/main/java and src/test/java
jbuild lint

# Check specific files or directories
jbuild lint src/main/java
jbuild lint src/main/java/com/example/App.java

# Use custom Checkstyle configuration
jbuild lint -c checkstyle.xml
```

**Available Checks:**
- **EmptyCatchBlock** - Detects empty catch blocks
- **EmptyStatement** - Detects empty statements (standalone semicolons)
- **MissingSwitchDefault** - Detects switch statements without default case
- **MultipleVariableDeclarations** - Detects multiple variable declarations per statement
- **SimplifyBooleanReturn** - Detects boolean returns that can be simplified
- **PackageName** - Validates package naming conventions
- **TypeName** - Validates type naming conventions
- **RedundantImport** - Detects redundant imports
- **LineLength** - Detects lines exceeding maximum length (default: 120)

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
