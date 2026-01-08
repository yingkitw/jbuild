use std::path::PathBuf;
use anyhow::{Context, Result};
use crate::testing::TestResult;

/// Test report generator
pub struct TestReporter;

impl TestReporter {
    /// Generate a test report
    pub fn generate_report(results: &[TestResult], output_dir: &PathBuf) -> Result<()> {
        std::fs::create_dir_all(output_dir)
            .context("Failed to create test report directory")?;

        // Generate summary
        let summary = Self::generate_summary(results);
        let summary_path = output_dir.join("summary.txt");
        std::fs::write(&summary_path, summary)
            .context("Failed to write test summary")?;

        // Generate detailed report
        let detailed = Self::generate_detailed_report(results);
        let detailed_path = output_dir.join("detailed.txt");
        std::fs::write(&detailed_path, detailed)
            .context("Failed to write detailed test report")?;

        Ok(())
    }

    /// Generate test summary
    fn generate_summary(results: &[TestResult]) -> String {
        let total_tests = results.len();
        let passed = results.iter().filter(|r| r.success).count();
        let failed = total_tests - passed;
        
        let total_test_methods = results.iter().map(|r| r.tests_run).sum::<usize>();
        let passed_methods = results.iter().map(|r| r.tests_passed).sum::<usize>();
        let failed_methods = results.iter().map(|r| r.tests_failed).sum::<usize>();

        format!(
            "Test Summary\n\
            ============\n\
            Tests run: {total_tests}\n\
            Tests passed: {passed}\n\
            Tests failed: {failed}\n\
            \n\
            Test methods run: {total_test_methods}\n\
            Test methods passed: {passed_methods}\n\
            Test methods failed: {failed_methods}\n"
        )
    }

    /// Generate detailed test report
    fn generate_detailed_report(results: &[TestResult]) -> String {
        let mut report = String::from("Detailed Test Report\n");
        report.push_str("===================\n\n");

        for result in results {
            report.push_str(&format!("Test Class: {}\n", result.class_name));
            report.push_str(&format!("  Status: {}\n", if result.success { "PASSED" } else { "FAILED" }));
            report.push_str(&format!("  Tests run: {}\n", result.tests_run));
            report.push_str(&format!("  Tests passed: {}\n", result.tests_passed));
            report.push_str(&format!("  Tests failed: {}\n", result.tests_failed));
            
            if !result.output.is_empty() {
                report.push_str(&format!("  Output:\n{}\n", result.output));
            }
            
            if !result.error_output.is_empty() {
                report.push_str(&format!("  Errors:\n{}\n", result.error_output));
            }
            
            report.push('\n');
        }

        report
    }
}

