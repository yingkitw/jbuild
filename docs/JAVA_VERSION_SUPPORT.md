# Java Version Support

jbuild provides comprehensive support for all Java versions through a flexible version handling system.

## Supported Java Versions

jbuild supports **all Java versions** without hardcoded limits:

- ✅ Java 8 (1.8.x)
- ✅ Java 11
- ✅ Java 17 (LTS)
- ✅ Java 21 (LTS)
- ✅ Java 22
- ✅ Java 23
- ✅ **Java 24**
- ✅ All future Java versions

## How It Works

The `JavaVersion` value object in `src/domain/shared/value_objects.rs` provides:

1. **Flexible Parsing**: Handles both old (1.8.0) and new (17.0.1) version formats
2. **Semantic Comparison**: Properly compares versions (e.g., 17 < 21 < 24)
3. **No Version Limits**: Works with any valid version number

### Version Parsing Examples

```rust
// Old format (Java 8)
JavaVersion::from_string("1.8.0") → JavaVersion { major: 8, minor: 0, patch: 0 }

// New format (Java 17+)
JavaVersion::from_string("17.0.1") → JavaVersion { major: 17, minor: 0, patch: 1 }
JavaVersion::from_string("21") → JavaVersion { major: 21, minor: 0, patch: 0 }
JavaVersion::from_string("24") → JavaVersion { major: 24, minor: 0, patch: 0 }
JavaVersion::from_string("24.0.1") → JavaVersion { major: 24, minor: 0, patch: 1 }
```

## Using Java 24 in Your Projects

### Maven (pom.xml)

```xml
<properties>
    <maven.compiler.source>24</maven.compiler.source>
    <maven.compiler.target>24</maven.compiler.target>
</properties>
```

### Gradle (build.gradle)

```groovy
java {
    sourceCompatibility = JavaVersion.VERSION_24
    targetCompatibility = JavaVersion.VERSION_24
}
```

### jbuild.toml

```toml
[package]
name = "my-app"
version = "1.0.0"
java = "24"
```

## Creating a New Java 24 Project

```bash
# Create a new project (will use default Java version)
jbuild new my-java24-app

# Then edit pom.xml or build.gradle to set Java 24
# Or use jbuild.toml with java = "24"
```

## Example Project

See `examples/java24-example/` for a complete working example using Java 24.

## Toolchain Support

The Gradle toolchain integration (`src/gradle/toolchain.rs`) automatically detects and works with any Java version installed on your system, including Java 24.

## Testing

The test suite includes verification for Java 24:

```rust
#[test]
fn test_java_version_parsing() {
    assert_eq!(
        JavaVersion::from_string("24"),
        Some(JavaVersion::new(24, 0, 0))
    );
    assert_eq!(
        JavaVersion::from_string("24.0.1"),
        Some(JavaVersion::new(24, 0, 1))
    );
}

#[test]
fn test_java_version_comparison() {
    let v21 = JavaVersion::new(21, 0, 0);
    let v24 = JavaVersion::new(24, 0, 0);
    assert!(v21 < v24);
}
```

## Future Compatibility

jbuild is designed to support all future Java versions automatically. When new Java versions are released (Java 25, 26, etc.), they will work without any code changes to jbuild.

The version handling system is:
- **Version-agnostic**: No hardcoded version checks
- **Forward-compatible**: Works with versions that don't exist yet
- **Backward-compatible**: Supports legacy version formats

## Implementation Details

### Core Components

1. **JavaVersion Value Object** (`src/domain/shared/value_objects.rs`)
   - Parses version strings
   - Provides semantic comparison
   - Supports display formatting

2. **Toolchain Support** (`src/gradle/toolchain.rs`)
   - Detects installed Java versions
   - Matches toolchain requirements
   - Works with any major version number

3. **Project Templates** (`src/runner/cli.rs`, `src/application/project_initialization.rs`)
   - Generate projects with configurable Java versions
   - Default to LTS versions (currently 17 or 21)

### No Version Validation

jbuild intentionally does **not** validate Java version numbers. This means:
- You can specify Java 24 even if it's not released yet
- You can use preview versions or custom builds
- The actual Java compiler will validate compatibility

This design philosophy ensures jbuild never becomes a bottleneck for adopting new Java versions.
