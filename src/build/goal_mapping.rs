//! Goal/Task Mapping
//!
//! Maps between Maven goals/phases and Gradle tasks for compatibility.

use std::collections::HashMap;

/// Maven phase to Gradle task mapping
pub struct GoalMapper {
    maven_to_gradle: HashMap<String, Vec<String>>,
    gradle_to_maven: HashMap<String, Vec<String>>,
}

impl Default for GoalMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalMapper {
    pub fn new() -> Self {
        let mut maven_to_gradle = HashMap::new();
        let mut gradle_to_maven = HashMap::new();

        // Maven phases -> Gradle tasks
        maven_to_gradle.insert("clean".to_string(), vec!["clean".to_string()]);
        maven_to_gradle.insert("validate".to_string(), vec!["check".to_string()]);
        maven_to_gradle.insert("compile".to_string(), vec!["compileJava".to_string()]);
        maven_to_gradle.insert("test-compile".to_string(), vec!["compileTestJava".to_string()]);
        maven_to_gradle.insert("test".to_string(), vec!["test".to_string()]);
        maven_to_gradle.insert("package".to_string(), vec!["jar".to_string()]);
        maven_to_gradle.insert("verify".to_string(), vec!["check".to_string()]);
        maven_to_gradle.insert("install".to_string(), vec!["publishToMavenLocal".to_string()]);
        maven_to_gradle.insert("deploy".to_string(), vec!["publish".to_string()]);
        
        // Common Maven plugin goals
        maven_to_gradle.insert("compiler:compile".to_string(), vec!["compileJava".to_string()]);
        maven_to_gradle.insert("compiler:testCompile".to_string(), vec!["compileTestJava".to_string()]);
        maven_to_gradle.insert("surefire:test".to_string(), vec!["test".to_string()]);
        maven_to_gradle.insert("jar:jar".to_string(), vec!["jar".to_string()]);
        maven_to_gradle.insert("war:war".to_string(), vec!["war".to_string()]);
        maven_to_gradle.insert("resources:resources".to_string(), vec!["processResources".to_string()]);
        maven_to_gradle.insert("resources:testResources".to_string(), vec!["processTestResources".to_string()]);

        // Gradle tasks -> Maven phases
        gradle_to_maven.insert("clean".to_string(), vec!["clean".to_string()]);
        gradle_to_maven.insert("compileJava".to_string(), vec!["compile".to_string()]);
        gradle_to_maven.insert("compileTestJava".to_string(), vec!["test-compile".to_string()]);
        gradle_to_maven.insert("processResources".to_string(), vec!["process-resources".to_string()]);
        gradle_to_maven.insert("processTestResources".to_string(), vec!["process-test-resources".to_string()]);
        gradle_to_maven.insert("test".to_string(), vec!["test".to_string()]);
        gradle_to_maven.insert("jar".to_string(), vec!["package".to_string()]);
        gradle_to_maven.insert("war".to_string(), vec!["package".to_string()]);
        gradle_to_maven.insert("check".to_string(), vec!["verify".to_string()]);
        gradle_to_maven.insert("build".to_string(), vec!["package".to_string()]);
        gradle_to_maven.insert("assemble".to_string(), vec!["package".to_string()]);
        gradle_to_maven.insert("publishToMavenLocal".to_string(), vec!["install".to_string()]);
        gradle_to_maven.insert("publish".to_string(), vec!["deploy".to_string()]);

        Self {
            maven_to_gradle,
            gradle_to_maven,
        }
    }

    /// Map Maven goal/phase to Gradle tasks
    pub fn maven_to_gradle(&self, maven_goal: &str) -> Vec<String> {
        self.maven_to_gradle
            .get(maven_goal)
            .cloned()
            .unwrap_or_else(|| vec![maven_goal.to_string()])
    }

    /// Map Gradle task to Maven phases
    pub fn gradle_to_maven(&self, gradle_task: &str) -> Vec<String> {
        self.gradle_to_maven
            .get(gradle_task)
            .cloned()
            .unwrap_or_else(|| vec![gradle_task.to_string()])
    }

    /// Convert a list of Maven goals to Gradle tasks
    pub fn convert_maven_goals(&self, goals: &[String]) -> Vec<String> {
        goals.iter()
            .flat_map(|g| self.maven_to_gradle(g))
            .collect()
    }

    /// Convert a list of Gradle tasks to Maven goals
    pub fn convert_gradle_tasks(&self, tasks: &[String]) -> Vec<String> {
        tasks.iter()
            .flat_map(|t| self.gradle_to_maven(t))
            .collect()
    }

    /// Check if a goal/task is a lifecycle phase (vs plugin goal)
    pub fn is_lifecycle_phase(goal: &str) -> bool {
        matches!(goal, 
            "validate" | "initialize" | "generate-sources" | "process-sources" |
            "generate-resources" | "process-resources" | "compile" | "process-classes" |
            "generate-test-sources" | "process-test-sources" | "generate-test-resources" |
            "process-test-resources" | "test-compile" | "process-test-classes" | "test" |
            "prepare-package" | "package" | "pre-integration-test" | "integration-test" |
            "post-integration-test" | "verify" | "install" | "deploy" | "clean" | "site"
        )
    }

    /// Check if a task is a standard Gradle task
    pub fn is_standard_gradle_task(task: &str) -> bool {
        matches!(task,
            "clean" | "assemble" | "build" | "check" | "test" |
            "compileJava" | "compileTestJava" | "processResources" | "processTestResources" |
            "jar" | "war" | "classes" | "testClasses" | "javadoc" |
            "publish" | "publishToMavenLocal" | "run" | "bootRun"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maven_to_gradle() {
        let mapper = GoalMapper::new();
        
        assert_eq!(mapper.maven_to_gradle("compile"), vec!["compileJava"]);
        assert_eq!(mapper.maven_to_gradle("test"), vec!["test"]);
        assert_eq!(mapper.maven_to_gradle("package"), vec!["jar"]);
    }

    #[test]
    fn test_gradle_to_maven() {
        let mapper = GoalMapper::new();
        
        assert_eq!(mapper.gradle_to_maven("compileJava"), vec!["compile"]);
        assert_eq!(mapper.gradle_to_maven("build"), vec!["package"]);
    }

    #[test]
    fn test_convert_goals() {
        let mapper = GoalMapper::new();
        
        let maven_goals = vec!["clean".to_string(), "compile".to_string(), "test".to_string()];
        let gradle_tasks = mapper.convert_maven_goals(&maven_goals);
        
        assert!(gradle_tasks.contains(&"clean".to_string()));
        assert!(gradle_tasks.contains(&"compileJava".to_string()));
        assert!(gradle_tasks.contains(&"test".to_string()));
    }

    #[test]
    fn test_is_lifecycle_phase() {
        assert!(GoalMapper::is_lifecycle_phase("compile"));
        assert!(GoalMapper::is_lifecycle_phase("test"));
        assert!(!GoalMapper::is_lifecycle_phase("compileJava"));
    }

    #[test]
    fn test_is_standard_gradle_task() {
        assert!(GoalMapper::is_standard_gradle_task("compileJava"));
        assert!(GoalMapper::is_standard_gradle_task("build"));
        assert!(!GoalMapper::is_standard_gradle_task("compile"));
    }
}
