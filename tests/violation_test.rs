//! Tests for Violation type

use jbuild::checkstyle::api::event::SeverityLevel;
use jbuild::checkstyle::api::violation::Violation;
use std::collections::BTreeSet;

#[test]
fn test_violation_new() {
    let violation = Violation::new(
        10,
        5,
        4,
        100,
        SeverityLevel::Error,
        "TestCheck".to_string(),
        "test.key".to_string(),
        vec!["arg1".to_string()],
        "messages".to_string(),
        "TestCheck".to_string(),
        None,
    );

    assert_eq!(violation.line_no, 10);
    assert_eq!(violation.column_no, 5);
    assert_eq!(violation.severity_level, SeverityLevel::Error);
    assert_eq!(violation.module_id, "TestCheck");
}

#[test]
fn test_violation_get_message_custom() {
    let violation = Violation::new(
        10,
        5,
        4,
        100,
        SeverityLevel::Error,
        "TestCheck".to_string(),
        "test.key".to_string(),
        vec!["arg1".to_string(), "arg2".to_string()],
        "messages".to_string(),
        "TestCheck".to_string(),
        Some("Custom message with {0} and {1}".to_string()),
    );

    let message = violation.get_message();
    assert!(message.contains("arg1"), "Should contain first argument");
    assert!(message.contains("arg2"), "Should contain second argument");
}

#[test]
fn test_violation_get_message_default() {
    let violation = Violation::new(
        10,
        5,
        4,
        100,
        SeverityLevel::Error,
        "TestCheck".to_string(),
        "test.key".to_string(),
        vec!["arg1".to_string(), "arg2".to_string()],
        "messages".to_string(),
        "TestCheck".to_string(),
        None,
    );

    let message = violation.get_message();
    assert!(message.contains("test.key"), "Should contain key");
    assert!(message.contains("arg1"), "Should contain arguments");
}

#[test]
fn test_violation_ordering_by_line() {
    let v1 = Violation::new(
        10,
        5,
        4,
        100,
        SeverityLevel::Error,
        "TestCheck".to_string(),
        "key1".to_string(),
        vec![],
        "messages".to_string(),
        "TestCheck".to_string(),
        None,
    );

    let v2 = Violation::new(
        20,
        5,
        4,
        100,
        SeverityLevel::Error,
        "TestCheck".to_string(),
        "key2".to_string(),
        vec![],
        "messages".to_string(),
        "TestCheck".to_string(),
        None,
    );

    assert!(v1 < v2, "v1 should be less than v2 (line 10 < 20)");
}

#[test]
fn test_violation_ordering_by_column() {
    let v1 = Violation::new(
        10,
        5,
        4,
        100,
        SeverityLevel::Error,
        "TestCheck".to_string(),
        "key1".to_string(),
        vec![],
        "messages".to_string(),
        "TestCheck".to_string(),
        None,
    );

    let v2 = Violation::new(
        10,
        15,
        14,
        100,
        SeverityLevel::Error,
        "TestCheck".to_string(),
        "key2".to_string(),
        vec![],
        "messages".to_string(),
        "TestCheck".to_string(),
        None,
    );

    assert!(v1 < v2, "v1 should be less than v2 (column 5 < 15)");
}

#[test]
fn test_violation_ordering_same_position() {
    let v1 = Violation::new(
        10,
        5,
        4,
        100,
        SeverityLevel::Error,
        "CheckA".to_string(),
        "key1".to_string(),
        vec![],
        "messages".to_string(),
        "CheckA".to_string(),
        None,
    );

    let v2 = Violation::new(
        10,
        5,
        4,
        100,
        SeverityLevel::Error,
        "CheckB".to_string(),
        "key2".to_string(),
        vec![],
        "messages".to_string(),
        "CheckB".to_string(),
        None,
    );

    assert!(v1 < v2, "Should order by module_id when position is same");
}

#[test]
fn test_violation_btreeset_ordering() {
    let mut violations = BTreeSet::new();
    
    violations.insert(Violation::new(
        20,
        5,
        4,
        100,
        SeverityLevel::Error,
        "CheckB".to_string(),
        "key2".to_string(),
        vec![],
        "messages".to_string(),
        "CheckB".to_string(),
        None,
    ));
    
    violations.insert(Violation::new(
        10,
        5,
        4,
        100,
        SeverityLevel::Error,
        "CheckA".to_string(),
        "key1".to_string(),
        vec![],
        "messages".to_string(),
        "CheckA".to_string(),
        None,
    ));

    let mut iter = violations.iter();
    let first = iter.next().unwrap();
    assert_eq!(first.line_no, 10, "First violation should be line 10");
    let second = iter.next().unwrap();
    assert_eq!(second.line_no, 20, "Second violation should be line 20");
}

#[test]
fn test_violation_all_severity_levels() {
    for severity in [
        SeverityLevel::Ignore,
        SeverityLevel::Info,
        SeverityLevel::Warning,
        SeverityLevel::Error,
    ] {
        let violation = Violation::new(
            10,
            5,
            4,
            100,
            severity,
            "TestCheck".to_string(),
            "test.key".to_string(),
            vec![],
            "messages".to_string(),
            "TestCheck".to_string(),
            None,
        );
        assert_eq!(violation.severity_level, severity);
    }
}

