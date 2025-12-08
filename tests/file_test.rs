//! Tests for File types

use jbuild::checkstyle::api::file::{FileContents, FileText};
use std::path::PathBuf;

#[test]
fn test_file_text_new() {
    let file_text = FileText::new(PathBuf::from("Test.java"), "content".to_string());
    assert_eq!(file_text.line_count(), 1);
}

#[test]
fn test_file_text_multiline() {
    let content = "line1\nline2\nline3";
    let file_text = FileText::new(PathBuf::from("Test.java"), content.to_string());
    assert_eq!(file_text.line_count(), 3);
}

#[test]
fn test_file_text_get_line() {
    let content = "line1\nline2\nline3";
    let file_text = FileText::new(PathBuf::from("Test.java"), content.to_string());
    assert_eq!(file_text.get_line(1), Some("line1"));
    assert_eq!(file_text.get_line(2), Some("line2"));
    assert_eq!(file_text.get_line(3), Some("line3"));
}

#[test]
fn test_file_text_get_line_out_of_range() {
    let file_text = FileText::new(PathBuf::from("Test.java"), "line1".to_string());
    assert_eq!(file_text.get_line(0), None);
    assert_eq!(file_text.get_line(2), None);
}

#[test]
fn test_file_text_empty() {
    let file_text = FileText::new(PathBuf::from("Test.java"), String::new());
    assert_eq!(file_text.line_count(), 0);
    assert_eq!(file_text.get_line(1), None);
}

#[test]
fn test_file_text_trailing_newline() {
    let content = "line1\nline2\n";
    let file_text = FileText::new(PathBuf::from("Test.java"), content.to_string());
    // Rust's lines() iterator doesn't include trailing empty line from trailing newline
    assert_eq!(file_text.line_count(), 2);
}

#[test]
fn test_file_contents_new() {
    let file_text = FileText::new(PathBuf::from("Test.java"), "content".to_string());
    let file_contents = FileContents::new(file_text);
    assert_eq!(file_contents.get_text().line_count(), 1);
}

#[test]
fn test_file_contents_get_text() {
    let file_text = FileText::new(PathBuf::from("Test.java"), "test content".to_string());
    let file_contents = FileContents::new(file_text);
    let text = file_contents.get_text();
    assert_eq!(text.get_line(1), Some("test content"));
}

