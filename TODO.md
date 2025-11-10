# TODO - Apache Maven to Rust Migration

This file tracks the remaining work items for the Maven to Rust migration.

## Completed ✅

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

## In Progress 🚧

None currently.

## Pending 📋

### High Priority

- [x] **Enhanced remote repository features**
  - [x] Repository metadata fetching
  - [x] Artifact checksum verification
  - [x] Download progress reporting
  - [x] Retry logic for failed downloads

### Medium Priority

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

### Lower Priority

- [x] **Enhanced POM features**
  - [x] Profile activation logic
  - [x] Property interpolation
  - [x] Model validation

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

- [x] **Maven plugin compatibility**
  - [x] Plugin API compatibility layer
  - [x] Plugin configuration inheritance
  - [ ] Legacy plugin support (framework ready, specific legacy formats pending)

## Notes

- All code is consolidated under `src/` in a single crate
- The foundation is solid and ready for plugin execution implementation
- Remote repository support requires HTTP client integration
- Compiler integration will need to invoke external tools (javac)

## Contributing

When working on TODO items:
1. Update this file to mark items as completed
2. Add implementation details to MIGRATION.md
3. Update code documentation
4. Add tests where applicable

