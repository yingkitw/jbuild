# Multi-Language Support

jbuild supports multiple JVM languages beyond Java, including Kotlin and Scala.

## Kotlin Support

### Overview
Full Kotlin compiler integration with support for mixed Java/Kotlin projects.

### Features

- **Kotlin Compiler Integration**: Direct kotlinc invocation
- **Mixed Source Compilation**: Compile Java and Kotlin together
- **Compiler Plugins**: Support for all-open, no-arg, Spring, etc.
- **JVM Target Configuration**: Configurable JVM target version
- **API/Language Versions**: Control Kotlin language features

### Configuration

#### Maven (pom.xml)

```xml
<properties>
    <kotlin.version>1.9.21</kotlin.version>
    <kotlin.compiler.jvmTarget>17</kotlin.compiler.jvmTarget>
</properties>

<dependencies>
    <dependency>
        <groupId>org.jetbrains.kotlin</groupId>
        <artifactId>kotlin-stdlib</artifactId>
        <version>${kotlin.version}</version>
    </dependency>
</dependencies>

<build>
    <plugins>
        <plugin>
            <groupId>org.jetbrains.kotlin</groupId>
            <artifactId>kotlin-maven-plugin</artifactId>
            <version>${kotlin.version}</version>
        </plugin>
    </plugins>
</build>
```

#### Gradle (build.gradle.kts)

```kotlin
plugins {
    kotlin("jvm") version "1.9.21"
}

kotlin {
    jvmToolchain(17)
}

dependencies {
    implementation(kotlin("stdlib"))
}
```

### Usage

```rust
use jbuild::compiler::kotlin::{KotlinCompiler, KotlinCompilerConfig};

let config = KotlinCompilerConfig {
    jvm_target: "17".to_string(),
    progressive: true,
    ..Default::default()
};

let compiler = KotlinCompiler::new(config);
let result = compiler.compile(&kotlin_sources, &output_dir, &classpath)?;
```

### Kotlin Plugins

#### Spring Plugin

```rust
use jbuild::compiler::kotlin::KotlinPlugins;

let mut config = KotlinCompilerConfig::default();
config.plugins = KotlinPlugins::spring();
```

This enables:
- `all-open` for `@Component`, `@Service`, `@Repository`, `@SpringBootApplication`
- `no-arg` for `@Entity` (JPA)

#### Custom Plugins

```rust
use jbuild::compiler::kotlin::KotlinPlugin;

let plugin = KotlinPlugin {
    id: "my-plugin".to_string(),
    path: PathBuf::from("my-plugin.jar"),
    options: vec![
        ("key1".to_string(), "value1".to_string()),
    ],
};

config.plugins.push(plugin);
```

### Mixed Java/Kotlin Projects

```rust
let result = compiler.compile_mixed(
    &kotlin_sources,
    &java_sources,
    &output_dir,
    &classpath
)?;
```

The Kotlin compiler handles both Kotlin and Java files in a single compilation pass.

## Scala Support

### Overview
Full Scala compiler integration with support for Scala 2.x and 3.x.

### Features

- **Scala Compiler Integration**: Direct scalac invocation
- **Version Support**: Scala 2.12, 2.13, and 3.x
- **Mixed Source Compilation**: Compile Java and Scala together
- **Optimization Flags**: Built-in optimization support
- **Compiler Options**: Full access to scalac options

### Configuration

#### Maven (pom.xml)

```xml
<properties>
    <scala.version>2.13.12</scala.version>
    <scala.binary.version>2.13</scala.binary.version>
</properties>

<dependencies>
    <dependency>
        <groupId>org.scala-lang</groupId>
        <artifactId>scala-library</artifactId>
        <version>${scala.version}</version>
    </dependency>
</dependencies>

<build>
    <plugins>
        <plugin>
            <groupId>net.alchim31.maven</groupId>
            <artifactId>scala-maven-plugin</artifactId>
            <version>4.8.1</version>
        </plugin>
    </plugins>
</build>
```

#### Gradle (build.gradle)

```groovy
plugins {
    id 'scala'
}

scala {
    zincVersion = '1.9.0'
}

dependencies {
    implementation 'org.scala-lang:scala-library:2.13.12'
}
```

### Usage

```rust
use jbuild::compiler::scala::{ScalaCompiler, ScalaCompilerConfig};

let config = ScalaCompilerConfig {
    scala_version: "2.13".to_string(),
    target: "17".to_string(),
    optimize: true,
    ..Default::default()
};

let compiler = ScalaCompiler::new(config);
let result = compiler.compile(&scala_sources, &output_dir, &classpath)?;
```

### Compiler Options

```rust
use jbuild::compiler::scala::ScalaOptions;

let mut config = ScalaCompilerConfig::default();

// Enable all warnings
config.options = ScalaOptions::all_warnings();

// Strict mode (warnings as errors)
config.options = ScalaOptions::strict();

// Custom options
config.options.push("-Ypartial-unification".to_string());
```

### Mixed Java/Scala Projects

```rust
let result = compiler.compile_mixed(
    &scala_sources,
    &java_sources,
    &output_dir,
    &classpath
)?;
```

Scala sources are compiled first, then Java sources with Scala classes in the classpath.

## Annotation Processing

### Overview
JSR 269 annotation processing support for code generation.

### Features

- **Processor Configuration**: Classpath and processor specification
- **Generated Sources**: Automatic handling of generated code
- **Processor Options**: Pass options to annotation processors
- **Common Processors**: Built-in support for popular processors

### Configuration

```rust
use jbuild::compiler::annotation_processor::{
    AnnotationProcessor, AnnotationProcessorConfig, CommonProcessors
};

let mut config = AnnotationProcessorConfig::new(
    PathBuf::from("target/generated-sources"),
    PathBuf::from("target/generated-classes"),
);

// Add processors
config.add_processor(CommonProcessors::lombok());
config.add_processor(CommonProcessors::mapstruct());

// Add processor options
config.add_option("mapstruct.defaultComponentModel".to_string(), "spring".to_string());

let processor = AnnotationProcessor::new(config);
```

### Common Processors

#### Lombok

```rust
config.add_processor(CommonProcessors::lombok());
```

Supports: `@Data`, `@Builder`, `@Getter`, `@Setter`, etc.

#### MapStruct

```rust
config.add_processor(CommonProcessors::mapstruct());
config.add_option("mapstruct.defaultComponentModel".to_string(), "spring".to_string());
```

#### Dagger

```rust
config.add_processor(CommonProcessors::dagger());
```

#### AutoValue

```rust
config.add_processor(CommonProcessors::autovalue());
```

#### Immutables

```rust
config.add_processor(CommonProcessors::immutables());
```

### Running Annotation Processing

```rust
let result = processor.process(
    &java_home,
    &source_files,
    &classpath,
    "17",  // source version
    "17",  // target version
)?;

if result.success {
    println!("Generated {} sources", result.generated_sources.len());
    println!("Generated {} classes", result.generated_classes.len());
}
```

## Language Detection

jbuild automatically detects languages based on file extensions:

- `.java` → Java
- `.kt` → Kotlin
- `.scala` → Scala

## Build Integration

### Maven

jbuild automatically detects and uses the appropriate compiler based on dependencies and plugins in `pom.xml`.

### Gradle

jbuild detects language plugins in `build.gradle`:

```groovy
plugins {
    id 'java'
    id 'org.jetbrains.kotlin.jvm'
    id 'scala'
}
```

## Performance Considerations

### Compilation Order

1. **Scala** → Compiled first (can reference Java)
2. **Kotlin** → Compiled second (can reference Java and Scala)
3. **Java** → Compiled last (can reference all)

### Incremental Compilation

Each language compiler supports incremental compilation:

- **Java**: Built-in incremental support
- **Kotlin**: Kotlin incremental compilation
- **Scala**: Zinc incremental compiler

### Parallel Compilation

Different language sources can be compiled in parallel when there are no cross-language dependencies.

## Examples

### Spring Boot with Kotlin

```kotlin
// src/main/kotlin/com/example/Application.kt
@SpringBootApplication
class Application

fun main(args: Array<String>) {
    runApplication<Application>(*args)
}
```

### Scala with Java Interop

```scala
// src/main/scala/com/example/ScalaService.scala
class ScalaService {
  def process(data: String): String = {
    // Can call Java classes
    val javaUtil = new JavaUtility()
    javaUtil.transform(data)
  }
}
```

### Mixed Project Structure

```
src/
├── main/
│   ├── java/
│   │   └── com/example/JavaClass.java
│   ├── kotlin/
│   │   └── com/example/KotlinClass.kt
│   └── scala/
│       └── com/example/ScalaClass.scala
└── test/
    ├── java/
    ├── kotlin/
    └── scala/
```

## Troubleshooting

### Compiler Not Found

```bash
# Set compiler home
export KOTLIN_HOME=/path/to/kotlin
export SCALA_HOME=/path/to/scala

# Or add to PATH
export PATH=$PATH:/path/to/kotlin/bin:/path/to/scala/bin
```

### Version Conflicts

Ensure all language versions are compatible with the target JVM version.

### Compilation Errors

Use verbose mode for detailed compiler output:

```bash
jbuild build --verbose
```

## Future Enhancements

- [ ] Groovy support
- [ ] Clojure support
- [ ] Ceylon support
- [ ] Language-specific optimization hints
- [ ] Cross-language refactoring support
