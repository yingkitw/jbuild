# DDD Migration Guide for jbuild

## Overview

This guide outlines the migration path from the current architecture to a full Domain-Driven Design (DDD) implementation. The migration is designed to be incremental, allowing the codebase to remain functional throughout the process.

## Current Status

✅ **Completed:**
- DDD architecture documentation created (`docs/DDD_ARCHITECTURE.md`)
- 10 bounded contexts identified and documented
- Domain layer structure created (`src/domain/`)
- Shared domain concepts implemented (value objects, domain events)
- Build System context with detector service
- Artifact context with value objects and repository traits
- ARCHITECTURE.md updated with DDD principles
- TODO.md updated with DDD adoption status

⏳ **In Progress:**
- Refactoring existing code to domain layer
- Repository implementations
- Domain events publishing and handling

## Migration Strategy

### Phase 1: Foundation (Completed ✅)

1. **Create Domain Layer Structure**
   - ✅ Created `src/domain/` module
   - ✅ Defined 10 bounded contexts
   - ✅ Created module structure for each context

2. **Define Shared Concepts**
   - ✅ Value objects: `Version`, `FilePath`, `JavaVersion`
   - ✅ Domain events: Base trait and common events
   - ✅ Shared utilities

3. **Documentation**
   - ✅ Comprehensive DDD architecture guide
   - ✅ Updated main ARCHITECTURE.md
   - ✅ Migration guide (this document)

### Phase 2: Value Objects Extraction (Next)

**Goal:** Replace primitive types with rich value objects

**Tasks:**
1. **Artifact Coordinates**
   - Refactor `artifact::ArtifactCoordinates` to use `domain::artifact::value_objects::ArtifactCoordinates`
   - Update all usages across the codebase
   - Add validation logic to value object

2. **Version Handling**
   - Replace string versions with `domain::shared::Version`
   - Implement semantic version comparison
   - Add snapshot detection logic

3. **Scope and Lifecycle**
   - Extract `Scope` enum to `domain::artifact::value_objects::Scope`
   - Extract `LifecyclePhase` to `domain::maven::value_objects::LifecyclePhase`
   - Add domain logic to these types

**Example Refactoring:**

Before:
```rust
pub struct Dependency {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub scope: Option<String>,
}
```

After:
```rust
use crate::domain::artifact::value_objects::{ArtifactCoordinates, Scope};
use crate::domain::shared::Version;

pub struct Dependency {
    coordinates: ArtifactCoordinates,
    scope: Scope,
}

impl Dependency {
    pub fn new(coordinates: ArtifactCoordinates, scope: Scope) -> Self {
        Self { coordinates, scope }
    }
    
    pub fn coordinates(&self) -> &ArtifactCoordinates {
        &self.coordinates
    }
    
    pub fn scope(&self) -> Scope {
        self.scope
    }
}
```

### Phase 3: Aggregate Roots (After Phase 2)

**Goal:** Define aggregate boundaries and roots

**Tasks:**
1. **MavenProject Aggregate**
   - Define `domain::maven::aggregates::MavenProject`
   - Encapsulate Model, Dependencies, Plugins
   - Enforce invariants (valid GAV, valid phases)
   - Migrate from `core::MavenProject`

2. **GradleProject Aggregate**
   - Define `domain::gradle::aggregates::GradleProject`
   - Encapsulate Tasks, Configurations, SourceSets
   - Enforce invariants (no circular task dependencies)
   - Migrate from `gradle::GradleProject`

3. **Artifact Aggregate**
   - Define `domain::artifact::aggregates::Artifact`
   - Encapsulate coordinates and file path
   - Enforce invariants (valid coordinates, file exists)

**Aggregate Design Pattern:**
```rust
pub struct MavenProject {
    // Aggregate root
    coordinates: ArtifactCoordinates,
    model: Model,
    dependencies: Vec<Dependency>,
    plugins: Vec<Plugin>,
}

impl MavenProject {
    // Factory method with validation
    pub fn new(coordinates: ArtifactCoordinates, model: Model) -> Result<Self> {
        // Validate invariants
        if !coordinates.is_valid() {
            return Err(anyhow!("Invalid coordinates"));
        }
        
        Ok(Self {
            coordinates,
            model,
            dependencies: Vec::new(),
            plugins: Vec::new(),
        })
    }
    
    // Domain methods
    pub fn add_dependency(&mut self, dep: Dependency) -> Result<()> {
        // Business logic
        self.dependencies.push(dep);
        Ok(())
    }
    
    // No direct access to internals
    pub fn coordinates(&self) -> &ArtifactCoordinates {
        &self.coordinates
    }
}
```

### Phase 4: Domain Services (After Phase 3)

**Goal:** Extract business logic into domain services

**Tasks:**
1. **Dependency Resolution Service**
   - Move logic from `resolver/` to `domain::artifact::services`
   - Implement `DependencyResolver` domain service
   - Use repository traits for data access

2. **Model Building Service**
   - Move logic from `model/model_builder.rs` to `domain::maven::services`
   - Implement `ModelBuilder` domain service
   - Handle POM inheritance and interpolation

3. **Task Graph Service**
   - Move logic from `gradle/task_graph.rs` to `domain::gradle::services`
   - Implement `TaskGraphBuilder` domain service
   - Handle task dependencies and cycles

**Domain Service Pattern:**
```rust
pub struct DependencyResolver {
    local_repo: Arc<dyn ArtifactRepository>,
    remote_repos: Vec<RemoteRepository>,
}

impl DependencyResolver {
    pub fn new(
        local_repo: Arc<dyn ArtifactRepository>,
        remote_repos: Vec<RemoteRepository>,
    ) -> Self {
        Self { local_repo, remote_repos }
    }
    
    pub fn resolve(&self, coords: &ArtifactCoordinates) -> Result<Artifact> {
        // Domain logic for resolution
        if let Some(path) = self.local_repo.find(coords)? {
            return Ok(Artifact::new(coords.clone(), path));
        }
        
        // Try remote repositories
        for repo in &self.remote_repos {
            if let Some(artifact) = repo.download(coords)? {
                self.local_repo.install(coords, artifact.path())?;
                return Ok(artifact);
            }
        }
        
        Err(anyhow!("Artifact not found: {}", coords))
    }
}
```

### Phase 5: Repository Pattern (After Phase 4)

**Goal:** Implement repository traits for all data access

**Tasks:**
1. **Artifact Repository**
   - Implement `domain::artifact::repositories::ArtifactRepository` trait
   - Create `LocalArtifactRepository` implementation
   - Migrate from `artifact::LocalRepository`

2. **Model Repository**
   - Create `ModelRepository` trait for POM access
   - Implement file-based and remote implementations

3. **Configuration Repository**
   - Create repository for jbuild.toml access
   - Implement workspace configuration loading

**Repository Implementation:**
```rust
pub struct LocalArtifactRepository {
    base_path: PathBuf,
}

impl ArtifactRepository for LocalArtifactRepository {
    fn find(&self, coords: &ArtifactCoordinates) -> Result<Option<PathBuf>> {
        let path = self.base_path
            .join(coords.group_id().replace('.', "/"))
            .join(coords.artifact_id())
            .join(coords.version())
            .join(format!("{}-{}.{}", 
                coords.artifact_id(), 
                coords.version(), 
                coords.extension()));
        
        if path.exists() {
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }
    
    fn install(&self, coords: &ArtifactCoordinates, file: PathBuf) -> Result<()> {
        // Installation logic
        Ok(())
    }
    
    fn exists(&self, coords: &ArtifactCoordinates) -> bool {
        self.find(coords).ok().flatten().is_some()
    }
    
    fn path(&self) -> &PathBuf {
        &self.base_path
    }
}
```

### Phase 6: Domain Events (After Phase 5)

**Goal:** Implement event-driven communication between contexts

**Tasks:**
1. **Event Publisher**
   - Create event bus/publisher
   - Implement async event handling
   - Add event persistence (optional)

2. **Event Handlers**
   - Create handlers for cross-context communication
   - Implement event-driven workflows
   - Add event logging

3. **Key Events**
   - `ProjectBuiltEvent` - triggers packaging
   - `DependencyResolvedEvent` - triggers compilation
   - `CompilationCompletedEvent` - triggers testing
   - `TestSuiteCompletedEvent` - triggers reporting

**Event Pattern:**
```rust
pub struct EventBus {
    handlers: HashMap<String, Vec<Box<dyn EventHandler>>>,
}

impl EventBus {
    pub fn publish<E: DomainEvent>(&self, event: E) {
        if let Some(handlers) = self.handlers.get(event.event_type()) {
            for handler in handlers {
                handler.handle(&event);
            }
        }
    }
    
    pub fn subscribe<E: DomainEvent, H: EventHandler>(&mut self, handler: H) {
        self.handlers
            .entry(E::event_type())
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
    }
}
```

### Phase 7: Layered Architecture (Final)

**Goal:** Complete separation into layers

**Tasks:**
1. **Presentation Layer**
   - Keep in `cli.rs` and `ui/`
   - Only handle user interaction
   - Delegate to application layer

2. **Application Layer**
   - Refactor `runner/` as application services
   - Orchestrate domain services
   - Handle transactions

3. **Domain Layer**
   - Complete migration to `domain/`
   - Pure business logic
   - No infrastructure dependencies

4. **Infrastructure Layer**
   - Keep in `artifact/`, `resolver/`, etc.
   - Implement repository traits
   - Handle external systems

## Testing Strategy

### Unit Tests
- Test value objects in isolation
- Test domain services with mock repositories
- Test aggregate invariants

### Integration Tests
- Test repository implementations
- Test service orchestration
- Test event handling

### Migration Tests
- Keep existing tests passing
- Add new tests for domain layer
- Gradually migrate tests to use domain types

## Benefits After Migration

1. **Maintainability**
   - Clear separation of concerns
   - Easy to understand domain logic
   - Reduced coupling between modules

2. **Testability**
   - Mock-friendly interfaces
   - Isolated domain logic
   - Comprehensive test coverage

3. **Extensibility**
   - Easy to add new build systems
   - Plugin architecture
   - Event-driven workflows

4. **Team Alignment**
   - Ubiquitous language
   - Clear domain boundaries
   - Consistent terminology

## Migration Checklist

- [x] Phase 1: Foundation
  - [x] Create domain layer structure
  - [x] Define shared concepts
  - [x] Write documentation

- [ ] Phase 2: Value Objects Extraction
  - [ ] Extract ArtifactCoordinates
  - [ ] Extract Version
  - [ ] Extract Scope and LifecyclePhase
  - [ ] Update all usages

- [ ] Phase 3: Aggregate Roots
  - [ ] Define MavenProject aggregate
  - [ ] Define GradleProject aggregate
  - [ ] Define Artifact aggregate
  - [ ] Migrate existing code

- [ ] Phase 4: Domain Services
  - [ ] Implement DependencyResolver
  - [ ] Implement ModelBuilder
  - [ ] Implement TaskGraphBuilder
  - [ ] Migrate business logic

- [ ] Phase 5: Repository Pattern
  - [ ] Implement ArtifactRepository
  - [ ] Implement ModelRepository
  - [ ] Implement ConfigRepository
  - [ ] Migrate data access

- [ ] Phase 6: Domain Events
  - [ ] Create event bus
  - [ ] Implement event handlers
  - [ ] Add key events
  - [ ] Test event workflows

- [ ] Phase 7: Layered Architecture
  - [ ] Finalize presentation layer
  - [ ] Finalize application layer
  - [ ] Finalize domain layer
  - [ ] Finalize infrastructure layer

## Next Steps

1. Start Phase 2: Extract value objects
2. Update tests to use new value objects
3. Ensure all builds pass
4. Continue with Phase 3

## References

- [DDD_ARCHITECTURE.md](DDD_ARCHITECTURE.md) - Detailed DDD documentation
- [ARCHITECTURE.md](../ARCHITECTURE.md) - Overall architecture
- [TODO.md](../TODO.md) - Current work items
