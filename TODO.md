# TODO - jbuild: High-Performance Java Build Tool

This file tracks the remaining work items for jbuild, a Rust implementation supporting both Maven and Gradle build systems.

## Completed ✅

### Maven Support
- [x] Complete POM XML parsing with namespace handling
- [x] Full dependency resolution algorithm with transitive dependencies (framework)
- [x] Build lifecycle execution (actual phase execution)
- [x] Settings file parsing
- [x] ProjectBuilder for building projects from POM files
- [x] LifecycleStarter for lifecycle execution
- [x] MojoExecutor for plugin execution
- [x] Reactor for multi-module project support
- [x] DefaultMaven execution engine
- [x] GraphBuilder for dependency graph construction
- [x] ModelBuilder for effective model construction
- [x] LifecycleExecutor with plugin bindings
- [x] **Remote repository artifact downloading** - HTTP client integration for artifact downloads
- [x] **Goal parsing and phase mapping** - Parse Maven goals and map to lifecycle phases
- [x] **Effective model building with parent resolution** - Build effective POM with parent inheritance
- [x] **Plugin loading and execution framework**
  - [x] Load plugins from local/remote repositories
  - [x] Plugin descriptor parsing from JAR files (META-INF/maven/plugin.xml)
  - [x] PluginRegistry for plugin caching and management
  - [x] Mojo execution with PluginRegistry integration
  - [x] Plugin dependency resolution (from plugin POM)
  - [x] Plugin classpath building (plugin JAR + dependencies)
  - [x] Basic Java plugin execution framework (JavaMojo with classpath setup)
  - [x] JNI integration for direct Java class loading (optional feature)
  - [x] External Maven process invocation for plugin execution (fallback)
  - [ ] Full JNI Mojo execution (requires Java helper classes for complete implementation)

### Common Infrastructure
- [x] **Enhanced remote repository features**
  - [x] Repository metadata fetching
  - [x] Artifact checksum verification
  - [x] Download progress reporting
  - [x] Retry logic for failed downloads
- [x] **Compiler integration (javac, etc.)**
  - [x] Java compiler invocation
  - [x] Compilation error handling
  - [x] Source file discovery
  - [x] Classpath management
  - [ ] Annotation processing support
- [x] **Test execution support**
  - [x] Test discovery (JUnit, TestNG, etc.)
  - [x] Test runner integration
  - [x] Test reporting
  - [x] Test classpath setup
- [x] **Packaging (jar, war, etc.)**
  - [x] JAR file creation
  - [x] WAR file packaging
  - [x] Manifest generation
  - [x] Resource inclusion/exclusion
- [x] **Advanced dependency resolution**
  - [x] Version range resolution
  - [x] Conflict resolution
  - [x] Optional dependencies
  - [x] Exclusions handling
  - [x] Dependency mediation
- [x] **Build optimization**
  - [x] Incremental compilation (build cache framework)
  - [x] Parallel execution (framework with tokio)
  - [x] Build caching
  - [ ] Reactor optimization (basic reactor exists, optimization pending)

## In Progress 🚧

### Gradle Support
- [x] Gradle build script parsing (Groovy DSL) - Basic parser implemented
- [x] Gradle build script parsing (Kotlin DSL) - Basic support (uses Groovy parser)
- [x] Gradle task execution framework - Core tasks (clean, compileJava, test, jar, build)
- [x] Gradle dependency resolution - Integrated with shared resolver
- [x] Gradle plugin system integration - Basic plugin detection and standard tasks

## Recent Improvements ✨

### Gradle Support Implementation (Nov 2025)
- [x] **Gradle Build Script Parser** - Implemented parser for Groovy/Kotlin DSL build scripts
- [x] **Gradle Model Structures** - Created GradleProject, Task, Dependency, Repository, Plugin models
- [x] **Gradle Task Execution** - Implemented task execution engine with dependency resolution
- [x] **Build System Integration** - Unified BuildExecutor trait for both Maven and Gradle
- [x] **Dependency Resolution** - Integrated Gradle dependencies with shared resolver
- [x] **Example Project** - Created example Gradle project for testing
- [x] **Tests** - Added basic tests for Gradle functionality

### Code Quality & Architecture (Nov 2025)
- [x] **Project Rename** - Renamed from mvn-rs to jbuild to reflect dual Maven/Gradle support
- [x] **Custom Error Types** - Created `MavenError` enum for better error handling
- [x] **Trait-Based Design** - Added traits for testability: `ProjectBuildStrategy`, `LifecycleExecutionStrategy`, `DependencyResolutionStrategy`, `ArtifactRepository`
- [x] **Builder Patterns** - Implemented `ExecutionRequestBuilder` for fluent API
- [x] **Testing Utilities** - Created `MockArtifactRepository`, `MockDependencyResolver`, `TestProjectBuilder`
- [x] **Unit Tests** - Added 12 comprehensive unit tests with mocks
- [x] **Snapshot Tests** - Added insta snapshot tests for complex outputs
- [x] **Documentation** - Updated README with testing guidelines and architecture patterns

## Pending 📋

### High Priority

- [x] **Gradle Build System Support**
  - [x] Gradle build script parser (Groovy DSL) - Basic parser implemented
  - [x] Gradle build script parser (Kotlin DSL) - Basic support
  - [x] Gradle task graph construction - Task dependency resolution
  - [x] Gradle task execution - Core tasks implemented
  - [ ] Gradle dependency resolution (using same artifact repository) - Integration in progress
  - [x] Gradle plugin system - Basic plugin detection and standard tasks
  - [ ] Gradle settings.gradle support - Multi-project builds (future)

### Medium Priority

- [x] **Build System Detection**
  - [x] Automatic detection of pom.xml vs build.gradle
  - [x] Unified CLI interface for both systems
  - [x] Cross-build system compatibility (Maven artifacts in Gradle, etc.) - Shared artifact repository

- [ ] **Enhanced Maven Features**
  - [ ] Enhanced POM features
    - [x] Profile activation logic
    - [x] Property interpolation
    - [x] Model validation
  - [ ] Maven plugin compatibility
    - [x] Plugin API compatibility layer
    - [x] Plugin configuration inheritance
    - [ ] Legacy plugin support (framework ready, specific legacy formats pending)

### Lower Priority

- [ ] **Performance Optimizations**
  - [ ] Reactor optimization (basic reactor exists, optimization pending)
  - [ ] Parallel dependency resolution
  - [ ] Build cache improvements
  - [ ] Incremental build improvements

- [ ] **Additional Features**
  - [ ] Annotation processing support
  - [ ] Multi-language support (Kotlin, Scala, etc.)
  - [ ] Build tool migration utilities

## Notes

- All code is consolidated under `src/` in a single crate
- The Maven foundation is solid and ready for Gradle support implementation
- Shared artifact repository and dependency resolution will benefit both build systems
- Rust's performance advantages will provide faster builds compared to Java-based tools
- The architecture is designed to support multiple build systems through trait-based abstractions

## Contributing

When working on TODO items:
1. Update this file to mark items as completed
2. Add implementation details to MIGRATION.md
3. Update code documentation
4. Add tests where applicable
5. Ensure both Maven and Gradle compatibility where applicable
