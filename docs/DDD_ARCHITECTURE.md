# Domain-Driven Design Architecture for jbuild

## Overview

jbuild adopts Domain-Driven Design (DDD) principles to create a maintainable, scalable, and test-friendly architecture. This document outlines the bounded contexts, domain models, and architectural patterns used throughout the codebase.

## Core DDD Principles Applied

1. **Ubiquitous Language** - Consistent terminology across code, documentation, and team communication
2. **Bounded Contexts** - Clear boundaries between different domain areas
3. **Entities & Value Objects** - Rich domain models with clear identity semantics
4. **Aggregates** - Consistency boundaries for domain operations
5. **Domain Services** - Business logic that doesn't belong to a single entity
6. **Repositories** - Abstraction for data access and persistence
7. **Domain Events** - Decoupled communication between bounded contexts

## Bounded Contexts

### 1. Build System Context
**Responsibility**: Detect and abstract different build systems (Maven, Gradle)

**Domain Model**:
- **Entities**: None (stateless detection)
- **Value Objects**: 
  - `BuildSystemType` (Maven, Gradle, JBuild)
  - `BuildFile` (path, type)
- **Services**:
  - `BuildSystemDetector` - Detects build system from project structure
  - `BuildExecutor` (trait) - Unified interface for executing builds
- **Anti-Corruption Layer**: 
  - `GoalMapper` - Maps Maven goals ↔ Gradle tasks
  - `PropertyConverter` - Converts Maven ↔ Gradle properties
  - `DependencyNotationConverter` - Converts dependency formats

**Bounded Context Map**:
```
Build System Context
    ↓ (uses)
Maven Context | Gradle Context
```

### 2. Maven Context
**Responsibility**: Maven-specific build system implementation

**Domain Model**:
- **Aggregates**:
  - `MavenProject` (root) - Represents a Maven project with its lifecycle
    - Contains: `Model`, `Build`, `Dependencies`, `Plugins`
    - Invariants: Valid GAV coordinates, valid lifecycle phases
  
- **Entities**:
  - `MavenSession` - Execution session with state
  - `ReactorProject` - Multi-module project with build status
  
- **Value Objects**:
  - `ArtifactCoordinates` (GAV: groupId, artifactId, version)
  - `LifecyclePhase` (Clean, Validate, Compile, Test, Package, etc.)
  - `Scope` (Compile, Test, Runtime, Provided, System, Import)
  - `Model` (POM representation)
  - `Dependency` (dependency declaration)
  
- **Services**:
  - `ModelBuilder` - Builds effective POM from raw POM + parent
  - `ProjectBuilder` - Constructs MavenProject from Model
  - `LifecycleExecutor` - Executes lifecycle phases
  - `MojoExecutor` - Executes plugin goals
  - `DependencyResolver` - Resolves transitive dependencies
  
- **Repositories**:
  - `LocalRepository` (trait) - Local artifact storage
  - `RemoteRepository` - Remote Maven repository access

**Domain Events**:
- `ProjectBuilt`
- `DependencyResolved`
- `LifecyclePhaseCompleted`
- `MojoExecuted`

### 3. Gradle Context
**Responsibility**: Gradle-specific build system implementation

**Domain Model**:
- **Aggregates**:
  - `GradleProject` (root) - Represents a Gradle project
    - Contains: `Tasks`, `Configurations`, `SourceSets`, `Plugins`
    - Invariants: Valid task graph (no cycles), valid configurations
  
- **Entities**:
  - `Task` - Executable unit of work with dependencies
  - `Configuration` - Dependency configuration (api, implementation, etc.)
  
- **Value Objects**:
  - `SourceSet` (main, test, custom)
  - `TaskNode` (task with dependencies)
  - `ConfigurationDependency` (GAV or project dependency)
  - `JavaToolchain` (JDK specification)
  
- **Services**:
  - `GradleExecutor` - Executes Gradle tasks
  - `TaskGraphBuilder` - Builds task dependency graph
  - `ToolchainResolver` - Finds matching JDK installations
  
- **Repositories**:
  - Shares `LocalRepository` with Maven Context
  - `VersionCatalog` - Centralized dependency versions

**Domain Events**:
- `TaskExecuted`
- `ConfigurationResolved`
- `SourceSetCompiled`

### 4. Artifact Context
**Responsibility**: Artifact management and resolution

**Domain Model**:
- **Aggregates**:
  - `Artifact` (root) - Represents a resolved artifact
    - Contains: `ArtifactCoordinates`, file path, metadata
    - Invariants: Coordinates are valid, file exists
  
- **Value Objects**:
  - `ArtifactCoordinates` (groupId, artifactId, version, classifier, extension)
  - `Version` (with comparison logic)
  - `VersionRange` (version constraints)
  
- **Services**:
  - `ArtifactResolver` - Resolves artifacts from repositories
  - `DependencyResolver` - Resolves transitive dependencies
  - `ConflictResolver` - Resolves version conflicts
  - `VersionComparator` - Compares versions semantically
  
- **Repositories**:
  - `LocalRepository` (trait) - Local artifact cache
  - `RemoteRepository` - Remote artifact source

**Domain Events**:
- `ArtifactDownloaded`
- `DependencyConflictResolved`
- `TransitiveDependenciesResolved`

### 5. Compilation Context
**Responsibility**: Java source compilation

**Domain Model**:
- **Aggregates**:
  - `CompilationUnit` (root) - Represents a compilation task
    - Contains: source files, classpath, output directory
    - Invariants: Valid Java sources, valid classpath
  
- **Value Objects**:
  - `Classpath` (ordered list of paths)
  - `SourceFile` (path, package, class name)
  - `CompilerOptions` (source version, target version, encoding)
  
- **Services**:
  - `JavaCompiler` - Invokes javac
  - `SourceDiscovery` - Discovers Java source files
  - `ClasspathBuilder` - Constructs compilation classpath
  
- **Domain Events**:
- `CompilationStarted`
- `CompilationCompleted`
- `CompilationFailed`

### 6. Testing Context
**Responsibility**: Test discovery and execution

**Domain Model**:
- **Aggregates**:
  - `TestSuite` (root) - Collection of test classes
    - Contains: test classes, test results
    - Invariants: Valid test framework (JUnit, TestNG)
  
- **Value Objects**:
  - `TestClass` (class name, test methods)
  - `TestResult` (passed, failed, skipped, duration)
  
- **Services**:
  - `TestDiscovery` - Discovers test classes
  - `TestRunner` - Executes tests
  - `TestReporter` - Generates test reports
  
- **Domain Events**:
- `TestsDiscovered`
- `TestExecuted`
- `TestSuiteCompleted`

### 7. Packaging Context
**Responsibility**: Creating distributable artifacts (JAR, WAR)

**Domain Model**:
- **Aggregates**:
  - `Package` (root) - Represents a packaged artifact
    - Contains: manifest, resources, compiled classes
    - Invariants: Valid manifest, valid structure
  
- **Value Objects**:
  - `Manifest` (main class, classpath, attributes)
  - `PackageType` (JAR, WAR, EAR)
  
- **Services**:
  - `JarPackager` - Creates JAR files
  - `WarPackager` - Creates WAR files
  - `ManifestGenerator` - Generates MANIFEST.MF
  
- **Domain Events**:
- `PackageCreated`
- `ManifestGenerated`

### 8. Plugin Context
**Responsibility**: Plugin loading and execution

**Domain Model**:
- **Aggregates**:
  - `Plugin` (root) - Represents a loaded plugin
    - Contains: descriptor, mojos, dependencies
    - Invariants: Valid descriptor, compatible version
  
- **Entities**:
  - `Mojo` - Executable plugin goal
  
- **Value Objects**:
  - `PluginDescriptor` (GAV, goals, configuration)
  - `PluginConfiguration` (parameters)
  
- **Services**:
  - `PluginRegistry` - Manages loaded plugins
  - `PluginLoader` - Loads plugins from repositories
  - `MojoExecutor` - Executes plugin goals
  
- **Repositories**:
  - Uses `LocalRepository` and `RemoteRepository`

### 9. Configuration Context
**Responsibility**: Project configuration (jbuild.toml, workspace)

**Domain Model**:
- **Aggregates**:
  - `JBuildConfig` (root) - jbuild.toml configuration
    - Contains: package info, dependencies, dev-dependencies
    - Invariants: Valid TOML, valid dependencies
  
- **Value Objects**:
  - `PackageInfo` (name, version, java version)
  - `WorkspaceConfig` (members, resolver settings)
  - `LockFile` (locked dependencies)
  
- **Services**:
  - `ConfigParser` - Parses jbuild.toml
  - `WorkspaceResolver` - Resolves workspace members
  - `LockFileGenerator` - Generates jbuild.lock

### 10. Code Quality Context
**Responsibility**: Code quality checks (linting, formatting)

**Domain Model**:
- **Aggregates**:
  - `LintReport` (root) - Results of linting
    - Contains: violations, file locations
    - Invariants: Valid source files
  
- **Value Objects**:
  - `Violation` (rule, severity, location, message)
  - `CheckstyleRule` (name, configuration)
  
- **Services**:
  - `CheckstyleRunner` - Runs Checkstyle checks
  - `CodeFormatter` - Formats Java code

## Layered Architecture

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
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│         Domain Layer                            │
│  - Entities, Value Objects, Aggregates          │
│  - Domain Services                              │
│  - Domain Events                                │
│  - Business logic and invariants                │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│         Infrastructure Layer                    │
│  - Repository implementations                   │
│  - External service adapters                    │
│  - File system, HTTP, process execution         │
└─────────────────────────────────────────────────┘
```

## Domain Model Patterns

### Entities vs Value Objects

**Entities** (have identity, mutable):
- `MavenProject` - identified by GAV coordinates
- `MavenSession` - identified by session ID
- `Task` - identified by name
- `Plugin` - identified by GAV coordinates

**Value Objects** (no identity, immutable):
- `ArtifactCoordinates` - GAV tuple
- `LifecyclePhase` - enum
- `Version` - version string with comparison
- `Classpath` - list of paths
- `Manifest` - manifest content

### Aggregates and Consistency Boundaries

**Aggregate Rules**:
1. Each aggregate has a root entity
2. External objects can only reference the root
3. Invariants are enforced within aggregate boundaries
4. Aggregates are transaction boundaries

**Key Aggregates**:
- `MavenProject` aggregate - ensures valid project state
- `GradleProject` aggregate - ensures valid task graph
- `Artifact` aggregate - ensures valid coordinates and file
- `Plugin` aggregate - ensures valid descriptor and mojos

### Repository Pattern

All repositories implement trait-based interfaces:

```rust
pub trait LocalRepository: Send + Sync {
    fn find_artifact(&self, coords: &ArtifactCoordinates) -> Result<Option<PathBuf>>;
    fn install_artifact(&self, artifact: &Artifact) -> Result<()>;
    fn exists(&self, coords: &ArtifactCoordinates) -> bool;
}
```

**Benefits**:
- Testability (mock implementations)
- Flexibility (swap implementations)
- Separation of concerns (domain vs infrastructure)

### Domain Services

Domain services encapsulate business logic that:
- Doesn't naturally fit in an entity or value object
- Operates on multiple aggregates
- Requires external dependencies

**Examples**:
- `DependencyResolver` - resolves transitive dependencies
- `ConflictResolver` - resolves version conflicts
- `ModelBuilder` - builds effective POM from inheritance
- `TaskGraphBuilder` - builds task dependency graph

## Anti-Corruption Layers

### Maven ↔ Gradle Translation

The `build` module provides anti-corruption layers:

- `GoalMapper` - translates Maven goals to Gradle tasks and vice versa
- `PropertyConverter` - converts property names between systems
- `DependencyNotationConverter` - converts dependency formats
- `ScopeMapper` - maps Maven scopes to Gradle configurations

This prevents Maven concepts from leaking into Gradle context and vice versa.

## Testing Strategy

### Unit Tests
- Test domain logic in isolation
- Use mock repositories and services
- Focus on invariants and business rules

### Integration Tests
- Test aggregate boundaries
- Test repository implementations
- Test service orchestration

### Domain Model Tests
- Test entity behavior
- Test value object equality and comparison
- Test aggregate invariants

## Migration Path

### Current State
The codebase has implicit DDD patterns but lacks explicit structure.

### Target State
1. **Explicit bounded contexts** - Clear module boundaries
2. **Rich domain models** - Entities with behavior, not anemic models
3. **Domain events** - Decoupled communication
4. **Repository abstractions** - All data access through repositories
5. **Application services** - Orchestrate use cases

### Migration Steps
1. ✅ Identify bounded contexts (completed)
2. ⏳ Extract value objects from primitives
3. ⏳ Define aggregate roots and boundaries
4. ⏳ Implement domain services
5. ⏳ Add domain events
6. ⏳ Refactor to layered architecture

## Benefits of DDD for jbuild

1. **Maintainability** - Clear separation of concerns
2. **Testability** - Mock-friendly interfaces
3. **Extensibility** - Easy to add new build systems
4. **Domain clarity** - Code reflects business concepts
5. **Team alignment** - Ubiquitous language
6. **Reduced coupling** - Bounded contexts prevent tight coupling

## References

- Eric Evans - "Domain-Driven Design: Tackling Complexity in the Heart of Software"
- Vaughn Vernon - "Implementing Domain-Driven Design"
- Martin Fowler - "Patterns of Enterprise Application Architecture"
