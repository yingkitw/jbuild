//! Tests for Event types

use jbuild::checkstyle::api::event::{AuditEvent, AuditEventType, SeverityLevel};
use jbuild::checkstyle::api::violation::Violation;
use std::path::PathBuf;

#[test]
fn test_audit_event_new() {
    let event = AuditEvent::new(None, None, AuditEventType::AuditStarted);
    assert_eq!(event.event_type, AuditEventType::AuditStarted);
    assert!(event.file.is_none());
    assert!(event.violation.is_none());
}

#[test]
fn test_audit_event_with_file() {
    let file = PathBuf::from("Test.java");
    let event = AuditEvent::new(Some(file.clone()), None, AuditEventType::FileStarted);
    assert_eq!(event.event_type, AuditEventType::FileStarted);
    assert_eq!(event.file, Some(file));
}

#[test]
fn test_audit_event_with_violation() {
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

    let event = AuditEvent::new(None, Some(violation.clone()), AuditEventType::AddError);
    assert_eq!(event.event_type, AuditEventType::AddError);
    assert_eq!(event.violation, Some(violation));
}

#[test]
fn test_audit_event_types() {
    let event_types = [
        AuditEventType::AuditStarted,
        AuditEventType::AuditFinished,
        AuditEventType::FileStarted,
        AuditEventType::FileFinished,
        AuditEventType::AddError,
        AuditEventType::AddException,
    ];

    for event_type in event_types {
        let event = AuditEvent::new(None, None, event_type);
        assert_eq!(event.event_type, event_type);
    }
}

#[test]
fn test_severity_level_default() {
    let default = SeverityLevel::default();
    assert_eq!(default, SeverityLevel::Error);
}

#[test]
fn test_severity_level_ordering() {
    assert!(SeverityLevel::Ignore < SeverityLevel::Info);
    assert!(SeverityLevel::Info < SeverityLevel::Warning);
    assert!(SeverityLevel::Warning < SeverityLevel::Error);
}

#[test]
fn test_severity_level_equality() {
    assert_eq!(SeverityLevel::Error, SeverityLevel::Error);
    assert_ne!(SeverityLevel::Error, SeverityLevel::Warning);
}

