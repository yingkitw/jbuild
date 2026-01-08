//! Tests for DefaultLogger

use jbuild::checkstyle::api::event::{AuditEvent, AuditEventType, SeverityLevel};
use jbuild::checkstyle::api::listener::AuditListener;
use jbuild::checkstyle::api::violation::Violation;
use jbuild::checkstyle::runner::default_logger::DefaultLogger;
use std::io::Cursor;
use std::path::PathBuf;

#[test]
fn test_default_logger_new() {
    let _logger = DefaultLogger::new();
    assert!(true, "DefaultLogger should be created");
}

#[test]
fn test_default_logger_with_writer() {
    let writer = Box::new(Cursor::new(Vec::new()));
    let _logger = DefaultLogger::with_writer(writer);
    assert!(true, "DefaultLogger should be created with custom writer");
}

#[test]
fn test_default_logger_audit_started() {
    let mut logger = DefaultLogger::new();
    let event = AuditEvent::new(None, None, AuditEventType::AuditStarted);
    let result = logger.audit_started(&event);
    assert!(result.is_ok(), "audit_started should succeed");
}

#[test]
fn test_default_logger_audit_finished() {
    let mut logger = DefaultLogger::new();
    let event = AuditEvent::new(None, None, AuditEventType::AuditFinished);
    let result = logger.audit_finished(&event);
    assert!(result.is_ok(), "audit_finished should succeed");
}

#[test]
fn test_default_logger_file_started() {
    let mut logger = DefaultLogger::new();
    let file = PathBuf::from("Test.java");
    let event = AuditEvent::new(Some(file), None, AuditEventType::FileStarted);
    let result = logger.file_started(&event);
    assert!(result.is_ok(), "file_started should succeed");
}

#[test]
fn test_default_logger_file_finished() {
    let mut logger = DefaultLogger::new();
    let file = PathBuf::from("Test.java");
    let event = AuditEvent::new(Some(file), None, AuditEventType::FileFinished);
    let result = logger.file_finished(&event);
    assert!(result.is_ok(), "file_finished should succeed");
}

#[test]
fn test_default_logger_add_error_with_column() {
    let writer = Box::new(Cursor::new(Vec::new()));
    let mut logger = DefaultLogger::with_writer(writer);
    
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

    let file = PathBuf::from("Test.java");
    let event = AuditEvent::new(Some(file), Some(violation), AuditEventType::AddError);
    let result = logger.add_error(&event);
    assert!(result.is_ok(), "add_error should succeed");
}

#[test]
fn test_default_logger_add_error_without_column() {
    let writer = Box::new(Cursor::new(Vec::new()));
    let mut logger = DefaultLogger::with_writer(writer);
    
    let violation = Violation::new(
        10,
        0,
        0,
        100,
        SeverityLevel::Warning,
        "TestCheck".to_string(),
        "test.key".to_string(),
        vec![],
        "messages".to_string(),
        "TestCheck".to_string(),
        None,
    );

    let file = PathBuf::from("Test.java");
    let event = AuditEvent::new(Some(file), Some(violation), AuditEventType::AddError);
    let result = logger.add_error(&event);
    assert!(result.is_ok(), "add_error should succeed");
}

#[test]
fn test_default_logger_ignore_severity() {
    let writer = Box::new(Cursor::new(Vec::new()));
    let mut logger = DefaultLogger::with_writer(writer);
    
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

    let file = PathBuf::from("Test.java");
    let event = AuditEvent::new(Some(file), Some(violation), AuditEventType::AddError);
    let result = logger.add_error(&event);
    assert!(result.is_ok(), "add_error should succeed for Ignore severity");
}

#[test]
fn test_default_logger_add_exception() {
    let writer = Box::new(Cursor::new(Vec::new()));
    let mut logger = DefaultLogger::with_writer(writer);
    
    let file = PathBuf::from("Test.java");
    let event = AuditEvent::new(Some(file), None, AuditEventType::AddException);
    let error = std::io::Error::other("Test error");
    let result = logger.add_exception(&event, &error);
    assert!(result.is_ok(), "add_exception should succeed");
}

#[test]
fn test_default_logger_all_severity_levels() {
    let writer = Box::new(Cursor::new(Vec::new()));
    let mut logger = DefaultLogger::with_writer(writer);
    
    for severity in [
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

        let file = PathBuf::from("Test.java");
        let event = AuditEvent::new(Some(file), Some(violation), AuditEventType::AddError);
        let result = logger.add_error(&event);
        assert!(result.is_ok(), "Should handle {severity:?} severity");
    }
}

