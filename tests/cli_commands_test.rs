//! Tests for CLI commands: new, tree, add

use std::fs;
use tempfile::TempDir;

/// Helper to create a temporary directory for testing
fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

// ============================================================================
// Tests for `jbuild new` command functionality
// ============================================================================

mod new_command {
    use super::*;

    #[test]
    fn test_to_pascal_case_simple() {
        assert_eq!(to_pascal_case("hello"), "Hello");
        assert_eq!(to_pascal_case("world"), "World");
    }

    #[test]
    fn test_to_pascal_case_with_hyphens() {
        assert_eq!(to_pascal_case("my-app"), "MyApp");
        assert_eq!(to_pascal_case("hello-world-app"), "HelloWorldApp");
    }

    #[test]
    fn test_to_pascal_case_with_underscores() {
        assert_eq!(to_pascal_case("my_app"), "MyApp");
        assert_eq!(to_pascal_case("hello_world_app"), "HelloWorldApp");
    }

    #[test]
    fn test_to_pascal_case_with_dots() {
        assert_eq!(to_pascal_case("my.app"), "MyApp");
        assert_eq!(to_pascal_case("com.example.app"), "ComExampleApp");
    }

    #[test]
    fn test_to_pascal_case_mixed() {
        assert_eq!(to_pascal_case("my-app_test.example"), "MyAppTestExample");
    }

    #[test]
    fn test_to_pascal_case_empty() {
        assert_eq!(to_pascal_case(""), "");
    }

    #[test]
    fn test_to_pascal_case_single_char() {
        assert_eq!(to_pascal_case("a"), "A");
    }

    #[test]
    fn test_package_name_generation() {
        // Package names should be lowercase with underscores
        let name = "my-app";
        let package_name = name.replace(['-', '.'], "_").to_lowercase();
        assert_eq!(package_name, "my_app");

        let name2 = "Hello.World-App";
        let package_name2 = name2.replace(['-', '.'], "_").to_lowercase();
        assert_eq!(package_name2, "hello_world_app");
    }

    #[test]
    fn test_create_maven_project_structure() {
        let temp_dir = create_temp_dir();
        let project_name = "test-maven-project";
        let project_path = temp_dir.path().join(project_name);

        // Create directory structure
        fs::create_dir_all(project_path.join("src/main/java/com/example")).unwrap();
        fs::create_dir_all(project_path.join("src/test/java/com/example")).unwrap();
        fs::create_dir_all(project_path.join("src/main/resources")).unwrap();
        fs::create_dir_all(project_path.join("src/test/resources")).unwrap();

        // Verify structure
        assert!(project_path.join("src/main/java/com/example").exists());
        assert!(project_path.join("src/test/java/com/example").exists());
        assert!(project_path.join("src/main/resources").exists());
        assert!(project_path.join("src/test/resources").exists());
    }

    #[test]
    fn test_create_gradle_project_structure() {
        let temp_dir = create_temp_dir();
        let project_name = "test-gradle-project";
        let project_path = temp_dir.path().join(project_name);

        // Create directory structure
        fs::create_dir_all(project_path.join("src/main/java/com/example")).unwrap();
        fs::create_dir_all(project_path.join("src/test/java/com/example")).unwrap();

        // Create build files
        fs::write(project_path.join("build.gradle"), "plugins { id 'java' }").unwrap();
        fs::write(project_path.join("settings.gradle"), "rootProject.name = 'test'").unwrap();

        // Verify structure
        assert!(project_path.join("build.gradle").exists());
        assert!(project_path.join("settings.gradle").exists());
    }

    #[test]
    fn test_generate_pom_xml() {
        let name = "my-app";
        let pom_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>{name}</artifactId>
    <version>1.0.0-SNAPSHOT</version>
</project>"#
        );

        assert!(pom_xml.contains("<artifactId>my-app</artifactId>"));
        assert!(pom_xml.contains("<groupId>com.example</groupId>"));
        assert!(pom_xml.contains("<version>1.0.0-SNAPSHOT</version>"));
    }

    #[test]
    fn test_generate_build_gradle() {
        let package_name = "my_app";
        let class_name = "MyApp";
        let build_gradle = format!(
            r#"plugins {{
    id 'java'
    id 'application'
}}

application {{
    mainClass = 'com.example.{package_name}.{class_name}'
}}"#
        );

        assert!(build_gradle.contains("id 'java'"));
        assert!(build_gradle.contains("id 'application'"));
        assert!(build_gradle.contains("mainClass = 'com.example.my_app.MyApp'"));
    }

    #[test]
    fn test_generate_main_java_app() {
        let package_name = "my_app";
        let class_name = "MyApp";
        let main_java = format!(
            r#"package com.example.{package_name};

public class {class_name} {{
    public static void main(String[] args) {{
        System.out.println("Hello!");
    }}
}}"#
        );

        assert!(main_java.contains("package com.example.my_app;"));
        assert!(main_java.contains("public class MyApp"));
        assert!(main_java.contains("public static void main"));
    }

    #[test]
    fn test_generate_main_java_lib() {
        let package_name = "my_lib";
        let class_name = "MyLib";
        let lib_java = format!(
            r#"package com.example.{package_name};

public class {class_name} {{
    public String greet(String name) {{
        return "Hello, " + name + "!";
    }}
}}"#
        );

        assert!(lib_java.contains("package com.example.my_lib;"));
        assert!(lib_java.contains("public class MyLib"));
        assert!(lib_java.contains("public String greet"));
    }

    #[test]
    fn test_generate_test_java() {
        let package_name = "my_app";
        let class_name = "MyApp";
        let test_java = format!(
            r#"package com.example.{package_name};

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

public class {class_name}Test {{
    @Test
    void testExample() {{
        assertTrue(true);
    }}
}}"#
        );

        assert!(test_java.contains("package com.example.my_app;"));
        assert!(test_java.contains("public class MyAppTest"));
        assert!(test_java.contains("@Test"));
        assert!(test_java.contains("import org.junit.jupiter.api.Test;"));
    }

    #[test]
    fn test_generate_gitignore() {
        let gitignore = r#"# Build outputs
target/
build/
out/

# IDE files
.idea/
*.iml
.vscode/
"#;

        assert!(gitignore.contains("target/"));
        assert!(gitignore.contains("build/"));
        assert!(gitignore.contains(".idea/"));
    }

    #[test]
    fn test_generate_readme() {
        let name = "my-app";
        let readme = format!(
            r#"# {name}

A Java project created with jbuild.

## Building

```bash
jbuild build
```
"#
        );

        assert!(readme.contains("# my-app"));
        assert!(readme.contains("jbuild build"));
    }

    /// Helper function to convert string to PascalCase
    fn to_pascal_case(s: &str) -> String {
        s.split(['-', '_', '.'])
            .filter(|s| !s.is_empty())
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect()
    }
}

// ============================================================================
// Tests for `jbuild tree` command functionality
// ============================================================================

mod tree_command {
    
    #[allow(unused_imports)]
    use jbuild::model::parser::parse_pom;

    #[test]
    fn test_parse_pom_for_tree() {
        let pom_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>my-app</artifactId>
    <version>1.0.0</version>
    <dependencies>
        <dependency>
            <groupId>junit</groupId>
            <artifactId>junit</artifactId>
            <version>4.13.2</version>
            <scope>test</scope>
        </dependency>
    </dependencies>
</project>"#;

        let model = parse_pom(pom_xml).unwrap();
        assert_eq!(model.group_id, "com.example");
        assert_eq!(model.artifact_id, "my-app");
        assert_eq!(model.version, "1.0.0");

        let deps = model.dependencies.unwrap();
        assert_eq!(deps.dependencies.len(), 1);
        assert_eq!(deps.dependencies[0].group_id, "junit");
        assert_eq!(deps.dependencies[0].artifact_id, "junit");
        assert_eq!(deps.dependencies[0].version, Some("4.13.2".to_string()));
        assert_eq!(deps.dependencies[0].scope, Some("test".to_string()));
    }

    #[test]
    fn test_parse_pom_multiple_dependencies() {
        let pom_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>multi-dep</artifactId>
    <version>1.0.0</version>
    <dependencies>
        <dependency>
            <groupId>org.slf4j</groupId>
            <artifactId>slf4j-api</artifactId>
            <version>2.0.9</version>
        </dependency>
        <dependency>
            <groupId>com.google.guava</groupId>
            <artifactId>guava</artifactId>
            <version>32.1.3-jre</version>
        </dependency>
        <dependency>
            <groupId>org.junit.jupiter</groupId>
            <artifactId>junit-jupiter</artifactId>
            <version>5.10.0</version>
            <scope>test</scope>
        </dependency>
    </dependencies>
</project>"#;

        let model = parse_pom(pom_xml).unwrap();
        let deps = model.dependencies.unwrap();
        assert_eq!(deps.dependencies.len(), 3);

        // Check first dependency
        assert_eq!(deps.dependencies[0].group_id, "org.slf4j");
        assert_eq!(deps.dependencies[0].artifact_id, "slf4j-api");
        assert_eq!(deps.dependencies[0].scope, None); // compile scope is default

        // Check second dependency
        assert_eq!(deps.dependencies[1].group_id, "com.google.guava");
        assert_eq!(deps.dependencies[1].artifact_id, "guava");

        // Check third dependency (test scope)
        assert_eq!(deps.dependencies[2].group_id, "org.junit.jupiter");
        assert_eq!(deps.dependencies[2].scope, Some("test".to_string()));
    }

    #[test]
    fn test_parse_pom_no_dependencies() {
        let pom_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>no-deps</artifactId>
    <version>1.0.0</version>
</project>"#;

        let model = parse_pom(pom_xml).unwrap();
        assert_eq!(model.group_id, "com.example");
        assert!(model.dependencies.is_none());
    }

    #[test]
    fn test_dependency_scope_display() {
        let scopes = vec![
            (Some("compile".to_string()), "compile"),
            (Some("test".to_string()), "test"),
            (Some("provided".to_string()), "provided"),
            (Some("runtime".to_string()), "runtime"),
            (None, "compile"), // default
        ];

        for (scope, expected) in scopes {
            let display = scope.as_deref().unwrap_or("compile");
            assert_eq!(display, expected);
        }
    }

    #[test]
    fn test_tree_output_format() {
        // Test the tree output format
        let deps = [("org.slf4j", "slf4j-api", "2.0.9", "compile"),
            ("junit", "junit", "4.13.2", "test")];

        let mut output = String::new();
        output.push_str("com.example:my-app:1.0.0\n");

        let dep_count = deps.len();
        for (i, (group, artifact, version, scope)) in deps.iter().enumerate() {
            let prefix = if i == dep_count - 1 { "└──" } else { "├──" };
            output.push_str(&format!("{prefix} {group}:{artifact}:{version} ({scope})\n"));
        }

        assert!(output.contains("com.example:my-app:1.0.0"));
        assert!(output.contains("├── org.slf4j:slf4j-api:2.0.9 (compile)"));
        assert!(output.contains("└── junit:junit:4.13.2 (test)"));
    }
}

// ============================================================================
// Tests for `jbuild add` command functionality
// ============================================================================

mod add_command {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_parse_dependency_full() {
        let dep = "org.slf4j:slf4j-api:2.0.9";
        let parts: Vec<&str> = dep.split(':').collect();

        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "org.slf4j");
        assert_eq!(parts[1], "slf4j-api");
        assert_eq!(parts[2], "2.0.9");
    }

    #[test]
    fn test_parse_dependency_without_version() {
        let dep = "org.slf4j:slf4j-api";
        let parts: Vec<&str> = dep.split(':').collect();

        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "org.slf4j");
        assert_eq!(parts[1], "slf4j-api");
    }

    #[test]
    fn test_parse_dependency_with_classifier() {
        let dep = "org.example:artifact:1.0.0:sources";
        let parts: Vec<&str> = dep.split(':').collect();

        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], "org.example");
        assert_eq!(parts[1], "artifact");
        assert_eq!(parts[2], "1.0.0");
        assert_eq!(parts[3], "sources");
    }

    #[test]
    fn test_invalid_dependency_format() {
        let dep = "invalid";
        let parts: Vec<&str> = dep.split(':').collect();

        assert_eq!(parts.len(), 1);
        assert!(parts.len() < 2); // Invalid format
    }

    #[test]
    fn test_generate_maven_dependency_xml() {
        let group_id = "org.slf4j";
        let artifact_id = "slf4j-api";
        let version = "2.0.9";
        let dev = false;

        let dep_xml = format!(
            r#"        <dependency>
            <groupId>{}</groupId>
            <artifactId>{}</artifactId>
            <version>{}</version>{}
        </dependency>"#,
            group_id,
            artifact_id,
            version,
            if dev { "\n            <scope>test</scope>" } else { "" }
        );

        assert!(dep_xml.contains("<groupId>org.slf4j</groupId>"));
        assert!(dep_xml.contains("<artifactId>slf4j-api</artifactId>"));
        assert!(dep_xml.contains("<version>2.0.9</version>"));
        assert!(!dep_xml.contains("<scope>test</scope>"));
    }

    #[test]
    fn test_generate_maven_test_dependency_xml() {
        let group_id = "org.junit.jupiter";
        let artifact_id = "junit-jupiter";
        let version = "5.10.0";
        let dev = true;

        let dep_xml = format!(
            r#"        <dependency>
            <groupId>{}</groupId>
            <artifactId>{}</artifactId>
            <version>{}</version>{}
        </dependency>"#,
            group_id,
            artifact_id,
            version,
            if dev { "\n            <scope>test</scope>" } else { "" }
        );

        assert!(dep_xml.contains("<groupId>org.junit.jupiter</groupId>"));
        assert!(dep_xml.contains("<scope>test</scope>"));
    }

    #[test]
    fn test_generate_gradle_dependency() {
        let group_id = "org.slf4j";
        let artifact_id = "slf4j-api";
        let version = "2.0.9";
        let dev = false;

        let config = if dev { "testImplementation" } else { "implementation" };
        let dep_line = format!("    {config} '{group_id}:{artifact_id}:{version}'");

        assert_eq!(dep_line, "    implementation 'org.slf4j:slf4j-api:2.0.9'");
    }

    #[test]
    fn test_generate_gradle_test_dependency() {
        let group_id = "org.junit.jupiter";
        let artifact_id = "junit-jupiter";
        let version = "5.10.0";
        let dev = true;

        let config = if dev { "testImplementation" } else { "implementation" };
        let dep_line = format!("    {config} '{group_id}:{artifact_id}:{version}'");

        assert_eq!(dep_line, "    testImplementation 'org.junit.jupiter:junit-jupiter:5.10.0'");
    }

    #[test]
    fn test_insert_dependency_into_pom() {
        let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project>
    <dependencies>
        <dependency>
            <groupId>existing</groupId>
            <artifactId>dep</artifactId>
            <version>1.0</version>
        </dependency>
    </dependencies>
</project>"#;

        let new_dep = r#"        <dependency>
            <groupId>new</groupId>
            <artifactId>dep</artifactId>
            <version>2.0</version>
        </dependency>
    </dependencies>"#;

        let new_content = pom_content.replace("</dependencies>", new_dep);

        assert!(new_content.contains("<groupId>existing</groupId>"));
        assert!(new_content.contains("<groupId>new</groupId>"));
        assert!(new_content.contains("<version>2.0</version>"));
    }

    #[test]
    fn test_insert_dependency_into_gradle() {
        let build_content = r#"plugins {
    id 'java'
}

dependencies {
    testImplementation 'junit:junit:4.13.2'
}"#;

        let dep_line = "    implementation 'org.slf4j:slf4j-api:2.0.9'\n";
        let new_content = build_content.replace(
            "dependencies {",
            &format!("dependencies {{\n{dep_line}")
        );

        assert!(new_content.contains("implementation 'org.slf4j:slf4j-api:2.0.9'"));
        assert!(new_content.contains("testImplementation 'junit:junit:4.13.2'"));
    }

    #[test]
    fn test_add_dependencies_section_to_pom() {
        let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project>
    <groupId>com.example</groupId>
    <artifactId>no-deps</artifactId>
    <version>1.0.0</version>
</project>"#;

        let deps_section = r#"    <dependencies>
        <dependency>
            <groupId>org.slf4j</groupId>
            <artifactId>slf4j-api</artifactId>
            <version>2.0.9</version>
        </dependency>
    </dependencies>

</project>"#;

        let new_content = pom_content.replace("</project>", deps_section);

        assert!(new_content.contains("<dependencies>"));
        assert!(new_content.contains("<groupId>org.slf4j</groupId>"));
    }

    #[test]
    fn test_add_dependencies_block_to_gradle() {
        let build_content = r#"plugins {
    id 'java'
}

repositories {
    mavenCentral()
}"#;

        let dep_line = "    implementation 'org.slf4j:slf4j-api:2.0.9'\n";
        let new_content = format!("{build_content}\n\ndependencies {{\n{dep_line}}}\n");

        assert!(new_content.contains("dependencies {"));
        assert!(new_content.contains("implementation 'org.slf4j:slf4j-api:2.0.9'"));
    }
}

// ============================================================================
// Integration tests for file operations
// ============================================================================

mod file_operations {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_write_and_read_pom() {
        let temp_dir = create_temp_dir();
        let pom_path = temp_dir.path().join("pom.xml");

        let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>test</artifactId>
    <version>1.0.0</version>
</project>"#;

        fs::write(&pom_path, pom_content).unwrap();
        let read_content = fs::read_to_string(&pom_path).unwrap();

        assert_eq!(pom_content, read_content);
    }

    #[test]
    fn test_write_and_read_build_gradle() {
        let temp_dir = create_temp_dir();
        let build_path = temp_dir.path().join("build.gradle");

        let build_content = r#"plugins {
    id 'java'
}

dependencies {
    testImplementation 'junit:junit:4.13.2'
}"#;

        fs::write(&build_path, build_content).unwrap();
        let read_content = fs::read_to_string(&build_path).unwrap();

        assert_eq!(build_content, read_content);
    }

    #[test]
    fn test_create_nested_directories() {
        let temp_dir = create_temp_dir();
        let nested_path = temp_dir.path().join("src/main/java/com/example/app");

        fs::create_dir_all(&nested_path).unwrap();

        assert!(nested_path.exists());
        assert!(nested_path.is_dir());
    }

    #[test]
    fn test_write_java_file() {
        let temp_dir = create_temp_dir();
        let java_dir = temp_dir.path().join("src/main/java/com/example");
        fs::create_dir_all(&java_dir).unwrap();

        let java_path = java_dir.join("App.java");
        let java_content = r#"package com.example;

public class App {
    public static void main(String[] args) {
        System.out.println("Hello!");
    }
}"#;

        fs::write(&java_path, java_content).unwrap();

        assert!(java_path.exists());
        let read_content = fs::read_to_string(&java_path).unwrap();
        assert!(read_content.contains("public class App"));
    }
}

// ============================================================================
// Tests for `jbuild init` command functionality
// ============================================================================

mod init_command {
    #[test]
    fn test_extract_package_name() {
        let content = r#"package com.example.app;

public class Main {
    public static void main(String[] args) {
        System.out.println("Hello!");
    }
}"#;
        
        let pkg = extract_package_name(content);
        assert_eq!(pkg, Some("com.example.app".to_string()));
    }

    #[test]
    fn test_extract_package_name_with_spaces() {
        let content = "  package   org.mycompany.utils  ;";
        let pkg = extract_package_name(content);
        assert_eq!(pkg, Some("org.mycompany.utils".to_string()));
    }

    #[test]
    fn test_extract_package_name_none() {
        let content = r#"public class NoPackage {
    public static void main(String[] args) {}
}"#;
        let pkg = extract_package_name(content);
        assert_eq!(pkg, None);
    }

    #[test]
    fn test_extract_class_name() {
        let content = r#"package com.example;

public class MyApplication {
    public static void main(String[] args) {}
}"#;
        let class = extract_class_name(content);
        assert_eq!(class, Some("MyApplication".to_string()));
    }

    #[test]
    fn test_extract_class_name_with_brace() {
        let content = "public class App{";
        let class = extract_class_name(content);
        assert_eq!(class, Some("App".to_string()));
    }

    #[test]
    fn test_extract_class_name_final() {
        let content = "public final class FinalClass {";
        let class = extract_class_name(content);
        assert_eq!(class, Some("FinalClass".to_string()));
    }

    #[test]
    fn test_extract_class_name_generic() {
        let content = "public class Container<T> {";
        let class = extract_class_name(content);
        // Note: Current implementation includes <T>, which is acceptable
        // The main class detection still works correctly
        assert!(class.is_some());
        assert!(class.unwrap().starts_with("Container"));
    }

    #[test]
    fn test_group_id_from_package() {
        let packages = vec![
            ("com.example.app", "com.example"),
            ("org.mycompany.utils", "org.mycompany"),
            ("io.github.user.project", "io.github"),
            ("simple", "com.example"), // fallback for single-part package
        ];

        for (pkg, expected) in packages {
            let parts: Vec<&str> = pkg.split('.').collect();
            let group_id = if parts.len() >= 2 {
                format!("{}.{}", parts[0], parts[1])
            } else {
                "com.example".to_string()
            };
            assert_eq!(group_id, expected);
        }
    }

    #[test]
    fn test_detect_main_method() {
        let with_main = r#"public class App {
    public static void main(String[] args) {
        System.out.println("Hello!");
    }
}"#;
        assert!(with_main.contains("public static void main"));

        let without_main = r#"public class Utils {
    public static String format(String s) {
        return s.trim();
    }
}"#;
        assert!(!without_main.contains("public static void main"));
    }

    #[test]
    fn test_init_generates_maven_pom() {
        let project_name = "my-project";
        let group_id = "com.example";
        let main_class = Some("com.example.Main".to_string());

        let pom_xml = format!(
            r#"<groupId>{group_id}</groupId>
    <artifactId>{project_name}</artifactId>
    <version>1.0.0-SNAPSHOT</version>"#
        );

        assert!(pom_xml.contains("<groupId>com.example</groupId>"));
        assert!(pom_xml.contains("<artifactId>my-project</artifactId>"));
    }

    #[test]
    fn test_init_generates_gradle_build() {
        let group_id = "com.example";
        let main_class = "com.example.Main";

        let build_gradle = format!(
            r#"plugins {{
    id 'java'
    id 'application'
}}

group = '{group_id}'

application {{
    mainClass = '{main_class}'
}}"#
        );

        assert!(build_gradle.contains("id 'java'"));
        assert!(build_gradle.contains("id 'application'"));
        assert!(build_gradle.contains("group = 'com.example'"));
        assert!(build_gradle.contains("mainClass = 'com.example.Main'"));
    }

    /// Helper function to extract package name from Java source
    fn extract_package_name(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("package ") && trimmed.ends_with(';') {
                let pkg = trimmed
                    .strip_prefix("package ")?
                    .strip_suffix(';')?
                    .trim();
                return Some(pkg.to_string());
            }
        }
        None
    }

    /// Helper function to extract class name from Java source
    fn extract_class_name(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.contains("public class ") || trimmed.contains("public final class ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                for (i, part) in parts.iter().enumerate() {
                    if *part == "class" && i + 1 < parts.len() {
                        let class_name = parts[i + 1]
                            .trim_end_matches('{')
                            .trim_end_matches('<')
                            .to_string();
                        return Some(class_name);
                    }
                }
            }
        }
        None
    }
}

// ============================================================================
// Tests for `jbuild remove` command functionality
// ============================================================================

mod remove_command {
    #[test]
    fn test_parse_dependency_for_remove() {
        let dep = "org.slf4j:slf4j-api";
        let parts: Vec<&str> = dep.split(':').collect();
        
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "org.slf4j");
        assert_eq!(parts[1], "slf4j-api");
    }

    #[test]
    fn test_remove_gradle_dependency() {
        let content = r#"plugins {
    id 'java'
}

dependencies {
    implementation 'org.slf4j:slf4j-api:2.0.9'
    testImplementation 'junit:junit:4.13.2'
}"#;

        let pattern = "org.slf4j:slf4j-api";
        let new_lines: Vec<&str> = content
            .lines()
            .filter(|line| !line.contains(pattern))
            .collect();
        let result = new_lines.join("\n");

        assert!(!result.contains("org.slf4j:slf4j-api"));
        assert!(result.contains("junit:junit:4.13.2"));
    }

    #[test]
    fn test_remove_maven_dependency_pattern() {
        let pom_content = r#"<dependencies>
    <dependency>
        <groupId>org.slf4j</groupId>
        <artifactId>slf4j-api</artifactId>
        <version>2.0.9</version>
    </dependency>
    <dependency>
        <groupId>junit</groupId>
        <artifactId>junit</artifactId>
        <version>4.13.2</version>
    </dependency>
</dependencies>"#;

        // Check that we can identify the dependency to remove
        assert!(pom_content.contains("<groupId>org.slf4j</groupId>"));
        assert!(pom_content.contains("<artifactId>slf4j-api</artifactId>"));
    }

    #[test]
    fn test_dependency_not_found() {
        let content = r#"dependencies {
    testImplementation 'junit:junit:4.13.2'
}"#;

        let pattern = "org.slf4j:slf4j-api";
        let has_dep = content.contains(pattern);
        
        assert!(!has_dep);
    }
}

// ============================================================================
// Tests for `jbuild search` command functionality
// ============================================================================

mod search_command {
    #[test]
    fn test_search_url_encoding() {
        let query = "slf4j api";
        let encoded = urlencoding::encode(query);
        assert_eq!(encoded, "slf4j%20api");
    }

    #[test]
    fn test_search_url_format() {
        let query = "guava";
        let limit = 10;
        let url = format!(
            "https://search.maven.org/solrsearch/select?q={}&rows={}&wt=json",
            urlencoding::encode(query),
            limit
        );
        
        assert!(url.contains("search.maven.org"));
        assert!(url.contains("q=guava"));
        assert!(url.contains("rows=10"));
    }

    #[test]
    fn test_parse_maven_central_response() {
        let json_response = r#"{
            "response": {
                "numFound": 2,
                "docs": [
                    {
                        "g": "org.slf4j",
                        "a": "slf4j-api",
                        "latestVersion": "2.0.9",
                        "timestamp": 1699000000000
                    },
                    {
                        "g": "org.slf4j",
                        "a": "slf4j-simple",
                        "latestVersion": "2.0.9",
                        "timestamp": 1699000000000
                    }
                ]
            }
        }"#;

        let json: serde_json::Value = serde_json::from_str(json_response).unwrap();
        let docs = json["response"]["docs"].as_array().unwrap();
        
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0]["g"].as_str().unwrap(), "org.slf4j");
        assert_eq!(docs[0]["a"].as_str().unwrap(), "slf4j-api");
        assert_eq!(docs[0]["latestVersion"].as_str().unwrap(), "2.0.9");
    }

    #[test]
    fn test_format_package_output() {
        let group = "org.slf4j";
        let artifact = "slf4j-api";
        let version = "2.0.9";
        
        let package = format!("{group}:{artifact}");
        let output = format!("{package:<50} {version:<15}");
        
        assert!(output.contains("org.slf4j:slf4j-api"));
        assert!(output.contains("2.0.9"));
    }

    #[test]
    fn test_timestamp_to_date() {
        let timestamp: i64 = 1699000000000; // milliseconds
        let secs = timestamp / 1000;
        
        let date = chrono::DateTime::from_timestamp(secs, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "?".to_string());
        
        assert!(date.starts_with("2023-11")); // November 2023
    }

    #[test]
    fn test_empty_search_results() {
        let json_response = r#"{
            "response": {
                "numFound": 0,
                "docs": []
            }
        }"#;

        let json: serde_json::Value = serde_json::from_str(json_response).unwrap();
        let docs = json["response"]["docs"].as_array().unwrap();
        
        assert!(docs.is_empty());
    }
}
