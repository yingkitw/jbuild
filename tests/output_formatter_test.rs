//! Tests for output formatters

use jbuild::checkstyle::api::event::SeverityLevel;
use jbuild::checkstyle::api::violation::Violation;
use jbuild::checkstyle::runner::output_formatter::{OutputFormatter, PlainFormatter, XmlFormatter};
use std::io::Cursor;

#[test]
fn test_plain_formatter_new() {
    let _formatter = PlainFormatter::new();
    // Just verify it can be created
    assert!(true, "PlainFormatter should be created");
}

#[test]
fn test_plain_formatter_format_violation_with_column() {
    let formatter = PlainFormatter::new();
    let violation = Violation::new(
        10,
        5,
        4,
        100,
        SeverityLevel::Error,
        "EmptyCatchBlock".to_string(),
        "catch.empty".to_string(),
        vec![],
        "messages".to_string(),
        "EmptyCatchBlockCheck".to_string(),
        None,
    );

    let formatted = formatter.format_violation(&violation, "Test.java");
    assert!(formatted.contains("[ERROR]"), "Should contain ERROR severity");
    assert!(formatted.contains("Test.java"), "Should contain file name");
    assert!(formatted.contains("10"), "Should contain line number");
    assert!(formatted.contains("5"), "Should contain column number");
    assert!(formatted.contains("EmptyCatchBlock"), "Should contain module ID");
}

#[test]
fn test_plain_formatter_format_violation_without_column() {
    let formatter = PlainFormatter::new();
    let violation = Violation::new(
        10,
        0,
        0,
        100,
        SeverityLevel::Warning,
        "LineLength".to_string(),
        "line.length".to_string(),
        vec![],
        "messages".to_string(),
        "LineLengthCheck".to_string(),
        None,
    );

    let formatted = formatter.format_violation(&violation, "Test.java");
    assert!(formatted.contains("[WARN]"), "Should contain WARN severity");
    assert!(!formatted.contains(":0:"), "Should not show column 0");
}

#[test]
fn test_plain_formatter_write_violations() {
    let formatter = PlainFormatter::new();
    let violation = Violation::new(
        10,
        5,
        4,
        100,
        SeverityLevel::Error,
        "TestCheck".to_string(),
        "test.key".to_string(),
        vec![],
        "messages".to_string(),
        "TestCheck".to_string(),
        None,
    );

    let violations = vec![("Test.java".to_string(), vec![violation])];
    let mut writer = Cursor::new(Vec::new());
    
    let result = formatter.write_violations(&mut writer, &violations);
    assert!(result.is_ok(), "Should write violations successfully");
    
    let output = String::from_utf8(writer.into_inner()).unwrap();
    assert!(output.contains("Test.java"), "Output should contain file name");
    assert!(output.contains("[ERROR]"), "Output should contain severity");
}

#[test]
fn test_plain_formatter_ignore_severity() {
    let formatter = PlainFormatter::new();
    let violation = Violation::new(
        10,
        5,
        4,
        100,
        SeverityLevel::Ignore,
        "TestCheck".to_string(),
        "test.key".to_string(),
        vec![],
        "messages".to_string(),
        "TestCheck".to_string(),
        None,
    );

    let violations = vec![("Test.java".to_string(), vec![violation])];
    let mut writer = Cursor::new(Vec::new());
    
    formatter.write_violations(&mut writer, &violations).unwrap();
    let output = String::from_utf8(writer.into_inner()).unwrap();
    assert!(output.is_empty(), "Ignore severity violations should not be written");
}

#[test]
fn test_xml_formatter_new() {
    let _formatter = XmlFormatter::new();
    assert!(true, "XmlFormatter should be created");
}

#[test]
fn test_xml_formatter_format_violation() {
    let formatter = XmlFormatter::new();
    let violation = Violation::new(
        10,
        5,
        4,
        100,
        SeverityLevel::Error,
        "TestCheck".to_string(),
        "test.key".to_string(),
        vec![],
        "messages".to_string(),
        "TestCheck".to_string(),
        None,
    );

    let formatted = formatter.format_violation(&violation, "Test.java");
    assert!(formatted.contains(r#"line="10""#), "Should contain line number");
    assert!(formatted.contains(r#"column="5""#), "Should contain column number");
    assert!(formatted.contains(r#"severity="error""#), "Should contain severity");
    assert!(formatted.contains(r#"source="TestCheck""#), "Should contain source");
}

#[test]
fn test_xml_formatter_write_violations() {
    let formatter = XmlFormatter::new();
    let violation = Violation::new(
        10,
        5,
        4,
        100,
        SeverityLevel::Error,
        "TestCheck".to_string(),
        "test.key".to_string(),
        vec![],
        "messages".to_string(),
        "TestCheck".to_string(),
        None,
    );

    let violations = vec![("Test.java".to_string(), vec![violation])];
    let mut writer = Cursor::new(Vec::new());
    
    let result = formatter.write_violations(&mut writer, &violations);
    assert!(result.is_ok(), "Should write violations successfully");
    
    let output = String::from_utf8(writer.into_inner()).unwrap();
    assert!(output.contains("<?xml"), "Should contain XML declaration");
    assert!(output.contains("<checkstyle"), "Should contain checkstyle root");
    assert!(output.contains(r#"<file name="Test.java">"#), "Should contain file element");
    assert!(output.contains("</checkstyle>"), "Should contain closing tag");
}

#[test]
fn test_xml_formatter_escape_special_chars() {
    let formatter = XmlFormatter::new();
    let violation = Violation::new(
        10,
        5,
        4,
        100,
        SeverityLevel::Error,
        "TestCheck".to_string(),
        "test.key".to_string(),
        vec![],
        "messages".to_string(),
        "TestCheck".to_string(),
        Some("Message with <tags> & \"quotes\"".to_string()),
    );

    let formatted = formatter.format_violation(&violation, "Test.java");
    assert!(formatted.contains("&lt;"), "Should escape <");
    assert!(formatted.contains("&gt;"), "Should escape >");
    assert!(formatted.contains("&amp;"), "Should escape &");
    assert!(formatted.contains("&quot;"), "Should escape \"");
}

#[test]
fn test_xml_formatter_empty_file() {
    let formatter = XmlFormatter::new();
    let violations = vec![("Test.java".to_string(), vec![])];
    let mut writer = Cursor::new(Vec::new());
    
    formatter.write_violations(&mut writer, &violations).unwrap();
    let output = String::from_utf8(writer.into_inner()).unwrap();
    // Should not contain file element if no violations
    assert!(!output.contains(r#"<file name="Test.java">"#), "Should not include empty files");
}

