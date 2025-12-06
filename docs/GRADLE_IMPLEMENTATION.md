# Gradle Implementation in jbuild

This document describes the Gradle support implementation in jbuild.

## Overview

jbuild now supports both Maven and Gradle build systems through a unified `BuildExecutor` trait. The implementation follows Gradle's architectural patterns while leveraging Rust's performance.

## Architecture

### Build System Abstraction

Both Maven and Gradle executors implement the `BuildExecutor` trait:

```rust
pub trait BuildExecutor: Send + Sync {
    fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult>;
    fn build_system(&self) -> BuildSystem;
}
```

### Gradle Components

1. **Model** (`src/gradle/model/`):
   - `GradleProject`: Represents a Gradle project
   - `Task`: Represents a Gradle task
   - `Dependency`: Represents a Gradle dependency
   - `Repository`: Represents a Gradle repository
   - `Plugin`: Represents a Gradle plugin

2. **Parser** (`src/gradle/model/parser.rs`):
   - Parses `build.gradle` (Groovy DSL) files
   - Parses `build.gradle.kts` (Kotlin DSL) files
   - Extracts plugins, dependencies, repositories, tasks

3. **Executor** (`src/gradle/core/mod.rs`):
   - `GradleExecutor`: Implements `BuildExecutor` trait
   - Task execution with dependency resolution
   - Integration with shared infrastructure (compiler, packaging, testing)

## Supported Features

### Build Script Parsing

- **Groovy DSL**: Parses `build.gradle` files
- **Kotlin DSL**: Basic support for `build.gradle.kts` files
- **Extracted Information**:
  - Plugins (e.g., `id 'java'`)
  - Group, version, name
  - Source/target compatibility
  - Repositories (mavenCentral, jcenter, google, custom)
  - Dependencies (implementation, testImplementation, etc.)
  - Tasks (custom and standard)

### Task Execution

Supported standard tasks (when Java plugin is applied):
- `clean`: Removes build directory
- `compileJava`: Compiles Java sources
- `test`: Compiles and runs tests
- `jar`: Creates JAR file
- `build`: Full build (compile, test, jar)

Task dependencies are automatically resolved and executed in order.

### Integration with Shared Infrastructure

Gradle executor leverages shared jbuild components:
- **Java Compiler**: Uses `crate::compiler::java_compiler::JavaCompiler`
- **Packaging**: Uses `crate::packaging::jar::JarBuilder`
- **Testing**: Uses `crate::testing::runner::TestRunner`
- **Dependency Resolution**: Will use shared `crate::resolver` (in progress)

## Usage

### Command Line

```bash
# Build a Gradle project
jbuild build

# Run specific task
jbuild test

# Clean project
jbuild clean
```

The build system is automatically detected from the presence of `build.gradle` or `build.gradle.kts`.

### Programmatic Usage

```rust
use jbuild::build::{BuildExecutor, ExecutionRequest};
use jbuild::gradle::core::GradleExecutor;

let executor = GradleExecutor::new();
let request = ExecutionRequest {
    base_directory: PathBuf::from("."),
    goals: vec!["build".to_string()],
    system_properties: HashMap::new(),
    show_errors: true,
    offline: false,
};

let result = executor.execute(request)?;
```

## Implementation Details

### Build Script Parser

The parser uses simple pattern matching to extract information from Gradle build scripts. This is a simplified approach suitable for common use cases. A full implementation would require:
- Full Groovy parser for complex scripts
- Full Kotlin parser for Kotlin DSL
- Support for all Gradle DSL features

### Task Execution

Tasks are executed with dependency resolution:
1. Find task by name
2. Execute all task dependencies first
3. Execute the task itself

Standard tasks are implemented using shared jbuild infrastructure.

### Limitations

Current implementation has some limitations:
1. **Parser**: Simplified parser, doesn't handle all Gradle DSL features
2. **Dependency Resolution**: Basic support, full integration with shared resolver in progress
3. **Plugins**: Only detects plugins, doesn't execute plugin-specific tasks
4. **Multi-project**: Settings.gradle support not yet implemented
5. **Custom Tasks**: Custom task actions are not executed (only logged)

## Future Enhancements

1. **Full Parser**: Implement complete Groovy/Kotlin DSL parser
2. **Dependency Resolution**: Full integration with shared dependency resolver
3. **Plugin System**: Execute plugin-specific tasks and configurations
4. **Multi-project**: Support for settings.gradle and multi-project builds
5. **Custom Tasks**: Execute custom task actions
6. **Gradle Wrapper**: Support for Gradle wrapper
7. **Build Cache**: Leverage Gradle's build cache

## Testing

Basic tests are included in the parser module. More comprehensive tests should be added for:
- Build script parsing (various formats)
- Task execution
- Dependency resolution
- Integration with shared components

## Performance

The Rust implementation provides significant performance benefits:
- Faster build script parsing
- Faster task execution
- Better memory efficiency
- Parallel execution support (future)

