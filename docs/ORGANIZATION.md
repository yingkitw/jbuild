# Codebase Organization

This document describes the organization of the jbuild codebase.

> **Note**: This organization is inspired by Gradle's platform-based architecture. See [GRADLE_LEARNINGS.md](GRADLE_LEARNINGS.md) for details on the architectural patterns we're adopting.

## Directory Structure

```
src/
├── lib.rs                 # Library root, exports all public APIs
├── main.rs                # CLI entry point
│
├── build/                 # Build system abstraction layer
│   ├── mod.rs            # Build system module exports
│   ├── detection.rs       # Build system detection (Maven vs Gradle)
│   └── executor.rs       # Generic build executor trait
│
├── common/                # Shared functionality (used by all build systems)
│   ├── artifact/         # Artifact handling and coordinates
│   ├── resolver/         # Dependency resolution
│   ├── compiler/         # Java compiler integration
│   ├── packaging/        # JAR/WAR packaging
│   └── testing/          # Test discovery and execution
│
├── maven/                 # Maven-specific implementation
│   ├── mod.rs            # Maven module exports
│   ├── model/            # Maven POM model (re-exports from model/)
│   ├── core/             # Maven execution engine (re-exports from core/)
│   ├── settings/         # Maven settings.xml (re-exports from settings/)
│   └── plugin/           # Maven plugins (re-exports from plugin_api/)
│
├── gradle/                # Gradle-specific implementation (future)
│   ├── mod.rs            # Gradle module exports
│   ├── model/            # Gradle build script model
│   └── core/             # Gradle task execution engine
│
├── model/                 # Maven POM model (kept for backward compatibility)
├── core/                  # Maven core execution (kept for backward compatibility)
├── settings/              # Maven settings (kept for backward compatibility)
├── plugin_api/            # Maven plugins (kept for backward compatibility)
│
├── artifact/              # Shared artifact handling
├── resolver/              # Shared dependency resolution
├── compiler/              # Shared Java compiler integration
├── packaging/             # Shared packaging functionality
├── testing/               # Shared test execution
├── error.rs               # Error types
└── testing_utils.rs       # Testing utilities
```

## Module Responsibilities

### `build/` - Build System Abstraction
- **Purpose**: Provides abstraction layer for different build systems
- **Key Components**:
  - `BuildSystem` enum: Identifies Maven vs Gradle
  - `BuildExecutor` trait: Generic interface for executing builds
  - Detection logic: Automatically detects build system from project files

### `common/` - Shared Functionality
- **Purpose**: Code shared across all build systems
- **Modules**:
  - `artifact/`: Artifact coordinates, handling, and repository operations
  - `resolver/`: Dependency resolution (works for both Maven and Gradle)
  - `compiler/`: Java compiler invocation and classpath management
  - `packaging/`: JAR/WAR file creation
  - `testing/`: Test discovery, execution, and reporting

### `maven/` - Maven Implementation
- **Purpose**: All Maven-specific functionality
- **Structure**: Re-exports from existing modules for now, will be migrated gradually
- **Modules**:
  - `model/`: Maven POM parsing and model structures
  - `core/`: Maven lifecycle execution, project building
  - `settings/`: Maven settings.xml parsing
  - `plugin/`: Maven plugin system

### `gradle/` - Gradle Implementation
- **Purpose**: Gradle-specific functionality (future)
- **Status**: Placeholder structure, ready for implementation
- **Planned Modules**:
  - `model/`: Gradle build script parsing (Groovy/Kotlin DSL)
  - `core/`: Gradle task execution engine

## Backward Compatibility

The existing module structure (`model/`, `core/`, `settings/`, `plugin_api/`) is maintained for backward compatibility. The new `maven/` module re-exports from these modules, allowing gradual migration.

## Migration Path

1. **Phase 1 (Current)**: 
   - Create new structure with re-exports
   - Add build system detection
   - Maintain backward compatibility

2. **Phase 2 (Future)**:
   - Gradually move Maven-specific code into `maven/` subdirectories
   - Update imports throughout codebase
   - Remove old module structure once migration complete

3. **Phase 3 (Future)**:
   - Implement Gradle support in `gradle/` module
   - Share common functionality via `common/` module

4. **Phase 4 (Future - Inspired by Gradle)**:
   - Reorganize into platform architecture:
     - `core-runtime/`: Process management, daemon, worker processes
     - `core-configuration/`: Project model building, build file parsing
     - `core-execution/`: Task/lifecycle execution, scheduling, caching
   - Build Maven and Gradle platforms on top of core platforms
   - Add service-based dependency injection
   - Implement explicit build state model

## Design Principles (Inspired by Gradle)

1. **Platform-Based Architecture**: Core platform provides base functionality, Maven/Gradle platforms build on top
2. **Separation of Concerns**: 
   - **Runtime**: Process management, daemon, worker processes
   - **Configuration**: Project model, build file parsing
   - **Execution**: Task/lifecycle execution, scheduling, caching
3. **Shared Infrastructure**: Common functionality (artifact handling, dependency resolution) is shared
4. **API Boundaries**: Clear separation between public API and internal implementation
5. **Backward Compatibility**: Existing code continues to work during migration
6. **Extensibility**: Easy to add new build systems in the future
7. **Type Safety**: Strong typing throughout, leveraging Rust's type system
8. **Service-Based Design**: Dependency injection for testability (future)

## Benefits of This Organization

1. **Clear Boundaries**: Easy to see what's Maven-specific vs shared
2. **Scalability**: Easy to add Gradle or other build systems
3. **Maintainability**: Related code is grouped together
4. **Testability**: Each module can be tested independently
5. **Documentation**: Structure makes it clear what each part does

