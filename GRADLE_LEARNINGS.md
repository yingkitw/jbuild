# Learnings from Gradle Repository

This document captures key architectural patterns and organizational principles learned from studying the Gradle codebase.

## Core Architectural Principles

### 1. Platform-Based Architecture

Gradle uses a **platform-based architecture** with clear separation of concerns:

- **core-runtime**: Provides runtimes/containers (client, daemon, worker processes) - the base layer
- **core-configuration**: Build structure and work specification (project model, DSL)
- **core-execution**: Running work efficiently (scheduling, execution, caching)

These platforms are layered:
```
core-runtime (base)
  ↓
core-configuration
  ↓
core-execution
  ↓
software platform (builds on core)
  ↓
jvm platform (builds on software + core)
  ↓
extensibility platform (builds on jvm + software + core)
```

### 2. Clear Package Organization

Gradle organizes code by **responsibility** rather than by build system:

```
org.gradle/
├── api/              # Public API
├── internal/         # Internal implementation
├── execution/        # Execution logic
├── configuration/    # Configuration logic
├── initialization/   # Build initialization
├── caching/          # Caching infrastructure
├── composite/        # Composite builds
└── ...
```

### 3. Separation of API and Implementation

- **Public API** (`api/`): Stable interfaces for external use
- **Internal** (`internal/`): Implementation details, can change
- Clear boundaries prevent coupling

### 4. Test Organization

Gradle uses multiple test directories:
- `test/`: Unit tests
- `integTest/`: Integration tests
- `testFixtures/`: Shared test utilities
- `crossVersionTest/`: Cross-version compatibility tests

### 5. Build Logic Separation

- **Runtime code**: In `subprojects/` and `platforms/`
- **Build-time logic**: In `build-logic/` (separate from runtime)
- Clear separation prevents build-time dependencies from leaking into runtime

## Key Patterns for jbuild

### 1. Runtime vs Configuration vs Execution

We should separate:
- **Runtime**: Process management, daemon, worker processes
- **Configuration**: Project model building, POM/Gradle parsing
- **Execution**: Task/lifecycle execution, scheduling, caching

### 2. Platform Abstraction

Instead of `maven/` and `gradle/` modules, consider:
- **Core platform**: Shared execution, caching, dependency resolution
- **Maven platform**: Maven-specific configuration and execution
- **Gradle platform**: Gradle-specific configuration and execution

Both platforms build on the core platform.

### 3. Package Structure

Organize by responsibility:
```
jbuild/
├── runtime/          # Process management, daemon
├── configuration/   # Project model, build file parsing
├── execution/       # Task/lifecycle execution
├── caching/         # Build caching
├── dependency/      # Dependency resolution
└── ...
```

Then have build-system-specific implementations:
```
jbuild/
├── maven/
│   ├── configuration/  # POM parsing
│   └── execution/      # Maven lifecycle
└── gradle/
    ├── configuration/ # Gradle script parsing
    └── execution/     # Gradle task execution
```

### 4. Service-Based Architecture

Gradle uses dependency injection with services:
- Services are registered per scope (build, project, etc.)
- Clear service boundaries
- Easy to test with mock services

### 5. Build State Model

Gradle tracks build state through lifecycle:
- Initialization → Configuration → Execution
- Each phase has clear responsibilities
- State transitions are explicit

## Recommendations for jbuild

1. **Adopt platform architecture**: Separate core from Maven/Gradle-specific code
2. **Organize by responsibility**: Group code by what it does, not which build system
3. **Clear API boundaries**: Separate public API from internal implementation
4. **Service-based design**: Use dependency injection for testability
5. **Explicit state model**: Track build lifecycle explicitly
6. **Separate test types**: Unit tests, integration tests, test fixtures

## Implementation Strategy

1. **Phase 1 (Current)**: 
   - Build system detection and abstraction layer ✅
   - Maven/Gradle module separation ✅
   - Backward compatibility maintained ✅

2. **Phase 2 (Future)**: 
   - Reorganize existing code into runtime/configuration/execution
   - Extract common platform (core-runtime, core-configuration, core-execution)
   - Build Maven and Gradle platforms on top of core

3. **Phase 3 (Future)**:
   - Add service-based dependency injection
   - Implement explicit build state model
   - Separate test types (unit, integration, fixtures)

## Key Takeaways

1. **Layered Architecture**: Core platform provides base, specialized platforms build on top
2. **Responsibility-Based Organization**: Group by what code does, not which build system
3. **Clear Boundaries**: API vs implementation, runtime vs build-time
4. **Testability**: Service-based design enables easy testing
5. **Explicit State**: Track lifecycle explicitly for clarity and correctness
