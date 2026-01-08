# Architecture - jbuild

This document describes the architecture of jbuild, a high-performance Rust implementation supporting both Maven and Gradle build systems.

## Domain-Driven Design (DDD)

jbuild adopts Domain-Driven Design principles to create a maintainable, scalable, and test-friendly architecture. See [docs/DDD_ARCHITECTURE.md](docs/DDD_ARCHITECTURE.md) for detailed DDD documentation.

### Key DDD Principles

- **Bounded Contexts**: Clear boundaries between domain areas (Maven, Gradle, Artifact, etc.)
- **Ubiquitous Language**: Consistent terminology across code and documentation
- **Entities & Value Objects**: Rich domain models with clear identity semantics
- **Aggregates**: Consistency boundaries for domain operations
- **Domain Services**: Business logic that doesn't belong to a single entity
- **Repositories**: Abstraction for data access and persistence
- **Domain Events**: Decoupled communication between contexts

## Overview

jbuild is a single-crate Rust implementation that supports both Maven and Gradle build systems, organized into logical modules under `src/`. The architecture follows the core principles of both build systems while leveraging Rust's type safety and performance.

## Module Structure

See [ORGANIZATION.md](ORGANIZATION.md) for detailed organization documentation.

```
src/
├── lib.rs              # Library root, exports all public APIs
├── main.rs             # CLI dispatcher (minimal entry point)
├── cli.rs              # CLI definition (Clap structs)
│
├── domain/             # Domain layer (DDD architecture)
│   ├── shared/         # Shared domain concepts (value objects, events)
│   ├── build_system/   # Build system detection and abstraction
│   ├── maven/          # Maven bounded context
│   ├── gradle/         # Gradle bounded context
│   ├── artifact/       # Artifact management bounded context
│   ├── compilation/    # Compilation bounded context
│   ├── testing/        # Testing bounded context
│   ├── packaging/      # Packaging bounded context
│   ├── plugin/         # Plugin bounded context
│   ├── config/         # Configuration bounded context
│   └── quality/        # Code quality bounded context
│
├── runner/             # Command implementation logic (Application layer)
│   ├── mod.rs          # Runner utilities
│   ├── cli.rs          # Modularized CLI command implementations
│   └── ...             # Other runner submodules (main_class, maven_central, etc.)
│
├── build/              # Build system abstraction layer (Infrastructure)
│   ├── detection.rs    # Build system detection (Maven vs Gradle)
│   └── executor.rs     # Generic build executor trait
│
├── maven/              # Maven-specific implementation
│   ├── model/          # Maven POM model
│   ├── core/           # Maven execution engine
│   ├── settings/       # Maven settings.xml
│   └── plugin/         # Maven plugins
│
├── gradle/             # Gradle-specific implementation
│   ├── model/          # Gradle build script model
│   ├── core/           # Gradle task execution
│   └── settings.rs     # settings.gradle parsing for multi-project builds
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
│   ├── optimization.rs # Build optimization (caching, parallel execution)
│   └── unit_of_work.rs # Gradle-inspired UnitOfWork abstraction
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

## Layered Architecture

jbuild follows a layered architecture aligned with DDD principles:

```
┌─────────────────────────────────────────────────┐
│         Presentation Layer (CLI)                │
│  - cli.rs: Command definitions                  │
│  - runner/: Command implementations             │
│  - ui/: User interface utilities                │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│         Application Layer                       │
│  - Orchestrates domain services                 │
│  - Use cases (build, test, run, etc.)          │
│  - Transaction boundaries                       │
│  - Located in: runner/, main.rs                 │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│         Domain Layer                            │
│  - Entities, Value Objects, Aggregates          │
│  - Domain Services                              │
│  - Domain Events                                │
│  - Business logic and invariants                │
│  - Located in: domain/                          │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│         Infrastructure Layer                    │
│  - Repository implementations                   │
│  - External service adapters                    │
│  - File system, HTTP, process execution         │
│  - Located in: artifact/, resolver/, etc.       │
└─────────────────────────────────────────────────┘
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

1. **Domain-Driven Design**: Explicit bounded contexts, rich domain models, and clear separation of concerns
2. **Single Crate**: All code in one crate for simplicity and faster compilation
3. **Layered Architecture**: Presentation → Application → Domain → Infrastructure
4. **Modular CLI**: Decoupled CLI definition (`src/cli.rs`) from command implementation (`src/runner/cli.rs`), improving maintainability and testability
5. **Consolidated Model Building**: Centralized all POM inheritance, property merging, and interpolation logic into `ModelBuilder`, reducing redundancy
6. **Error Handling**: Uses `anyhow` for application errors, `thiserror` for library errors
7. **Async Support**: `tokio` for I/O operations (future use for parallel builds)
8. **Type Safety**: Strong typing for coordinates, phases, and configurations
9. **Compiler Integration**: Native Rust implementation for Java compiler invocation with classpath management
10. **Dual Build System Support**: Unified artifact and dependency resolution for both Maven and Gradle
11. **Repository Pattern**: All data access through trait-based repository interfaces
12. **Domain Events**: Decoupled communication between bounded contexts

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

## Bounded Contexts

jbuild is organized into the following bounded contexts:

1. **Build System Context** - Build system detection and abstraction
2. **Maven Context** - Maven-specific implementation
3. **Gradle Context** - Gradle-specific implementation
4. **Artifact Context** - Artifact management and resolution
5. **Compilation Context** - Java source compilation
6. **Testing Context** - Test discovery and execution
7. **Packaging Context** - Creating distributable artifacts
8. **Plugin Context** - Plugin loading and execution
9. **Configuration Context** - Project configuration (jbuild.toml)
10. **Code Quality Context** - Code quality checks

Each bounded context has its own:
- **Entities**: Objects with identity
- **Value Objects**: Immutable objects without identity (✅ Phase 2 complete)
- **Aggregates**: Consistency boundaries
- **Domain Services**: Business logic
- **Repositories**: Data access abstractions

### Implementation Status

**Phase 2: Value Objects (Completed)**
- ✅ `ArtifactCoordinates` with validation, GAV parsing, and repository path calculation
- ✅ `Version` with semantic comparison and ordering (handles snapshots and qualifiers)
- ✅ `Scope` enum for dependency scopes
- ✅ `LifecyclePhase` enum with ordering and phase execution logic
- ✅ `VersionRange` for dependency version constraints
- ✅ 29 comprehensive tests passing

**Phase 3: Aggregate Roots (Completed)**
- ✅ `MavenProject` aggregate root with:
  - Project coordinates, metadata, and configuration
  - Dependencies and plugins as entities within the aggregate
  - Multi-module support with validation
  - Business invariants: no duplicate dependencies/plugins, multi-module must be POM packaging
- ✅ `GradleProject` aggregate root with:
  - Project identity (name, group, version)
  - Configurations for dependency management
  - Tasks with dependency graph validation
  - Circular dependency detection
  - Multi-project build support
- ✅ Consistency boundaries enforced at aggregate level
- ✅ 13 aggregate tests passing (243 total tests)

**Phase 4: Domain Services (Completed)**
- ✅ `BuildSystemDetector` service for detecting Maven/Gradle/JBuild projects
- ✅ `DependencyResolver` service with:
  - Transitive dependency resolution
  - Circular dependency detection
  - Conflict resolution using nearest-wins strategy
  - Scope-based filtering
- ✅ `VersionResolver` service for version range resolution
- ✅ `LifecycleExecutor` service for Maven with:
  - Phase execution planning
  - Plugin goal binding
  - Custom plugin execution support
- ✅ `TaskExecutor` service for Gradle with:
  - Task dependency resolution using topological sort
  - Circular dependency detection
  - Parallel execution planning with execution levels
- ✅ **257 tests passing** (14 new service tests added)

**Phase 5: Repository Implementations (Completed)**
- ✅ `LocalRepository` for local artifact storage (~/.m2/repository)
  - Install and retrieve artifacts
  - List available versions
  - POM file parsing (stub)
- ✅ `RemoteRepository` for Maven Central with local caching
  - Cache directory management
  - Artifact URL generation
  - Download stub (ready for HTTP implementation)
- ✅ `RepositoryChain` for fallback logic
  - Multiple repository support
  - Automatic fallback on failure
  - Install to first repository
- ✅ **270 tests passing** (13 new repository tests added)

**Phase 6: Application Services (Completed)**
- ✅ `BuildOrchestrationService` for build execution
  - Detect build system type
  - Execute Maven phases and goals
  - Execute Gradle tasks
  - Clean build artifacts
- ✅ `ProjectInitializationService` for project creation
  - Create Maven projects with pom.xml
  - Create Gradle projects with build.gradle
  - Create JBuild projects with jbuild.toml
  - Generate standard directory structure
  - Create sample Java files
- ✅ `DependencyManagementService` for dependency operations
  - Resolve transitive dependencies
  - Get latest versions
  - Add dependencies to projects
- ✅ **285 tests passing** (15 new application service tests added)

See [docs/DDD_ARCHITECTURE.md](docs/DDD_ARCHITECTURE.md) for detailed information on each bounded context.

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

## Gradle-Inspired Patterns

The following patterns were migrated from Gradle's execution engine:

### UnitOfWork Trait
Inspired by Gradle's `UnitOfWork.java`, this abstraction provides:
- **WorkIdentity**: Unique identification for work units
- **InputFingerprint**: Caching and up-to-date checks via input hashing
- **WorkOutput**: Standardized execution results
- **ExecutionContext**: Work execution environment
- **InputVisitor/OutputVisitor**: Patterns for discovering inputs/outputs

### Multi-Project Builds
Settings.gradle support enables:
- **GradleSettings**: Model for settings.gradle parsing
- **SubprojectConfig**: Per-subproject configuration
- **include/includeFlat**: Standard Gradle include statements
- **Multi-project task execution**: Execute tasks across all projects

## Future Architecture Considerations

1. **Gradle Support**: ✅ Implemented
   - ✅ Gradle build script parsing (Groovy/Kotlin DSL)
   - ✅ Gradle task execution
   - ✅ Gradle dependency resolution
   - ✅ Multi-project builds (settings.gradle)
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
- Gradle build scripts (Groovy/Kotlin DSL) - implemented
- Gradle dependency management - implemented
- Gradle multi-project builds (settings.gradle) - implemented
