# Java 24 Example Project

This example demonstrates that jbuild fully supports Java 24 and all future Java versions.

## Configuration

The `pom.xml` is configured to use Java 24:

```xml
<properties>
    <maven.compiler.source>24</maven.compiler.source>
    <maven.compiler.target>24</maven.compiler.target>
</properties>
```

## Building

```bash
# Compile the project
jbuild compile

# Build and run
jbuild build
jbuild run
```

## Java Version Support

jbuild uses a flexible `JavaVersion` value object that can parse and work with any Java version:
- Java 8 (1.8.0)
- Java 11
- Java 17
- Java 21
- Java 24
- Any future Java versions

The system has no hardcoded version limits and will work with any valid Java version number.
