use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result};
use crate::compiler::ClasspathBuilder;
use crate::testing::{TestClass, TestFramework};

/// Test execution result
#[derive(Debug, Clone)]
pub struct TestResult {
    pub class_name: String,
    pub success: bool,
    pub output: String,
    pub error_output: String,
    pub tests_run: usize,
    pub tests_passed: usize,
    pub tests_failed: usize,
}

/// Test runner for executing Java tests
#[allow(dead_code)]
pub struct TestRunner {
    classpath: ClasspathBuilder,
    test_classes_dir: PathBuf,
}

impl TestRunner {
    pub fn new(test_classes_dir: PathBuf) -> Self {
        Self {
            classpath: ClasspathBuilder::new(),
            test_classes_dir,
        }
    }

    /// Add to test classpath
    pub fn add_to_classpath(mut self, entry: PathBuf) -> Self {
        self.classpath = self.classpath.add_entry(entry);
        self
    }

    /// Add JAR to test classpath
    pub fn add_jar(mut self, jar: PathBuf) -> Self {
        self.classpath = self.classpath.add_jar(jar);
        self
    }

    /// Execute a single test class
    pub fn run_test(&self, test_class: &TestClass) -> Result<TestResult> {
        tracing::info!("Running test class: {}", test_class.class_name);

        match test_class.framework {
            TestFramework::JUnit4 => self.run_junit4_test(test_class),
            TestFramework::JUnit5 => self.run_junit5_test(test_class),
            TestFramework::TestNG => self.run_testng_test(test_class),
            TestFramework::Unknown => {
                // Try JUnit4 as default
                self.run_junit4_test(test_class)
            }
        }
    }

    /// Execute all test classes
    pub fn run_tests(&self, test_classes: &[TestClass]) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();

        for test_class in test_classes {
            match self.run_test(test_class) {
                Ok(result) => results.push(result),
                Err(e) => {
                    tracing::error!("Failed to run test {}: {}", test_class.class_name, e);
                    results.push(TestResult {
                        class_name: test_class.class_name.clone(),
                        success: false,
                        output: String::new(),
                        error_output: format!("Test execution failed: {}", e),
                        tests_run: 0,
                        tests_passed: 0,
                        tests_failed: 1,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Run JUnit 4 test
    fn run_junit4_test(&self, test_class: &TestClass) -> Result<TestResult> {
        // Find JUnit runner
        let junit_runner = self.find_junit_runner()?;
        
        let classpath = self.classpath.build();
        
        let output = Command::new("java")
            .arg("-cp")
            .arg(&classpath)
            .arg(&junit_runner)
            .arg(&test_class.class_name)
            .output()
            .context("Failed to execute JUnit test")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Parse output to determine results
        let (tests_run, tests_passed, tests_failed) = self.parse_junit_output(&stdout);

        Ok(TestResult {
            class_name: test_class.class_name.clone(),
            success: output.status.success() && tests_failed == 0,
            output: stdout,
            error_output: stderr,
            tests_run,
            tests_passed,
            tests_failed,
        })
    }

    /// Run JUnit 5 test
    fn run_junit5_test(&self, test_class: &TestClass) -> Result<TestResult> {
        // JUnit 5 uses ConsoleLauncher
        let classpath = self.classpath.build();
        
        let output = Command::new("java")
            .arg("-cp")
            .arg(&classpath)
            .arg("org.junit.platform.console.ConsoleLauncher")
            .arg("--select-class")
            .arg(&test_class.class_name)
            .output()
            .context("Failed to execute JUnit 5 test")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let (tests_run, tests_passed, tests_failed) = self.parse_junit5_output(&stdout);

        Ok(TestResult {
            class_name: test_class.class_name.clone(),
            success: output.status.success() && tests_failed == 0,
            output: stdout,
            error_output: stderr,
            tests_run,
            tests_passed,
            tests_failed,
        })
    }

    /// Run TestNG test
    fn run_testng_test(&self, test_class: &TestClass) -> Result<TestResult> {
        let classpath = self.classpath.build();
        
        let output = Command::new("java")
            .arg("-cp")
            .arg(&classpath)
            .arg("org.testng.TestNG")
            .arg("-testclass")
            .arg(&test_class.class_name)
            .output()
            .context("Failed to execute TestNG test")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let (tests_run, tests_passed, tests_failed) = self.parse_testng_output(&stdout);

        Ok(TestResult {
            class_name: test_class.class_name.clone(),
            success: output.status.success() && tests_failed == 0,
            output: stdout,
            error_output: stderr,
            tests_run,
            tests_passed,
            tests_failed,
        })
    }

    /// Find JUnit 4 runner class
    fn find_junit_runner(&self) -> Result<String> {
        // Try to find org.junit.runner.JUnitCore in classpath
        // For now, return the class name
        Ok("org.junit.runner.JUnitCore".to_string())
    }

    /// Parse JUnit 4 output
    fn parse_junit_output(&self, output: &str) -> (usize, usize, usize) {
        // Simple parsing - in a full implementation, would parse XML reports
        let tests_run = output.matches("Tests run:").count();
        let tests_passed = output.matches("OK").count();
        let tests_failed = output.matches("FAILURES!!!").count();
        
        (tests_run, tests_passed, tests_failed)
    }

    /// Parse JUnit 5 output
    fn parse_junit5_output(&self, output: &str) -> (usize, usize, usize) {
        // Parse JUnit 5 console output
        let tests_run = output.matches("tests found").count();
        let tests_passed = output.matches("tests successful").count();
        let tests_failed = output.matches("tests failed").count();
        
        (tests_run, tests_passed, tests_failed)
    }

    /// Parse TestNG output
    fn parse_testng_output(&self, output: &str) -> (usize, usize, usize) {
        // Parse TestNG output
        let tests_run = output.matches("Total tests run:").count();
        let tests_passed = output.matches("Passed:").count();
        let tests_failed = output.matches("Failed:").count();
        
        (tests_run, tests_passed, tests_failed)
    }
}

