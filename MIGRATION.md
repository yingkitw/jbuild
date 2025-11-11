# Apache Maven to Rust Migration

This document describes the migration of Apache Maven from Java to Rust.

## Project Structure

The Rust implementation is organized as a **single crate** with all modules under `src/`:

### Core Modules

1. **model/** - POM (Project Object Model) parsing and data structures
   - Model, Parent, Dependency structures
   - Build configuration
   - Repository and profile definitions
   - POM parser (XML parsing)
   - Model builder for effective model construction

2. **artifact/** - Artifact handling
   - Artifact coordinates (groupId:artifactId:version)
   - Artifact representation
   - Local repository interface
   - Artifact handlers

3. **core/** - Core execution engine
   - Maven execution request/result
   - Lifecycle phases
   - Project representation
   - Maven session management
   - **ProjectBuilder** - builds MavenProject from POM files
   - **LifecycleStarter** - starts lifecycle execution
   - **MojoExecutor** - executes plugin mojos
   - **Reactor** - manages multi-module project execution
   - **DefaultMaven** - main execution engine
   - **GraphBuilder** - builds project dependency graph

4. **resolver/** - Dependency resolution
   - Dependency resolver
   - Remote repository support
   - Repository metadata

5. **settings/** - Settings and configuration
   - Settings structure
   - Profile management
   - Server and mirror configuration

6. **plugin_api/** - Plugin API
   - Mojo interface
   - Plugin descriptor
   - Plugin execution context

7. **main.rs** - Command-line interface
   - CLI argument parsing (using clap)
   - Main entry point
   - Command execution

## Key Features Implemented

- ✅ Complete POM model structure
- ✅ Artifact coordinate system
- ✅ Dependency management structures
- ✅ Build lifecycle phases
- ✅ CLI interface with common Maven commands
- ✅ **ProjectBuilder** for building projects from POM files
- ✅ **LifecycleStarter** for lifecycle execution
- ✅ **MojoExecutor** for plugin execution
- ✅ **Reactor** for multi-module project support
- ✅ **DefaultMaven** execution engine
- ✅ **GraphBuilder** for dependency graph construction
- ✅ **ModelBuilder** for effective model construction
- ✅ **POM XML parser with namespace handling** - Improved parser that handles Maven XML namespaces
- ✅ **Settings.xml parser** - Parser for Maven settings files with automatic loading
- ✅ **Transitive dependency resolver** - Framework for resolving transitive dependencies
- ✅ **LifecycleExecutor** - Executes lifecycle phases with plugin bindings
- ✅ **ArtifactDownloader** - Downloads artifacts from remote repositories via HTTP
- ✅ **GoalParser** - Parses Maven goals and maps them to lifecycle phases
- ✅ **EffectiveModelBuilder** - Builds effective POM with parent resolution and inheritance

## Status

This is an ongoing migration that provides the foundational structure and core execution engine. 

See [TODO.md](TODO.md) for the current list of remaining work items.

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --bin mvn -- [maven-args]
```

## Architecture Notes

The Rust implementation follows a similar module structure to the Java version but leverages Rust's type system and safety features:

- Strong typing for artifact coordinates
- Error handling with `anyhow` and `thiserror`
- Async support with `tokio` for I/O operations
- Structured logging with `tracing`
- All modules consolidated in a single crate under `src/`

## Module Organization

All code is organized under `src/`:
- `src/model/` - POM model structures
- `src/artifact/` - Artifact handling
- `src/core/` - Core execution engine
- `src/resolver/` - Dependency resolution
- `src/settings/` - Settings management
- `src/plugin_api/` - Plugin API
- `src/main.rs` - CLI entry point
- `src/lib.rs` - Library root

## Next Steps

1. Implement complete POM parser with proper XML namespace handling
2. Implement full dependency resolution with transitive dependencies
3. Add plugin loading and actual execution
4. Integrate with Java compiler (javac)
5. Add test execution support
6. Implement packaging (jar, war, etc.)
7. Add settings.xml parsing
8. Implement remote repository artifact downloading
