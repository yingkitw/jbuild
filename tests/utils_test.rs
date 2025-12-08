//! Unit tests for utility functions

use jbuild::checkstyle::utils::common_util::{length_expanded_tabs, line_length_expanded};

#[test]
fn test_length_expanded_tabs_no_tabs() {
    let line = "hello world";
    let result = length_expanded_tabs(line, line.len(), 8);
    assert_eq!(result, 11, "Line without tabs should have length 11");
}

#[test]
fn test_length_expanded_tabs_with_tabs() {
    let line = "hello\tworld";
    let result = length_expanded_tabs(line, line.len(), 8);
    // "hello" = 5 chars, tab expands to column 8, then "world" = 5 more
    assert_eq!(result, 13, "Tab should expand to next tab stop");
}

#[test]
fn test_length_expanded_tabs_multiple_tabs() {
    let line = "\t\tx";
    let result = length_expanded_tabs(line, line.len(), 4);
    // First tab: 0 -> 4, second tab: 4 -> 8, then 'x' = 1 more
    assert_eq!(result, 9, "Multiple tabs should expand correctly");
}

#[test]
fn test_length_expanded_tabs_partial_column() {
    let line = "hello\tworld";
    let result = length_expanded_tabs(line, 5, 8); // Only up to "hello"
    assert_eq!(
        result, 5,
        "Partial column should only count up to specified position"
    );
}

#[test]
fn test_line_length_expanded() {
    let line = "hello\tworld";
    let result = line_length_expanded(line, 8);
    assert_eq!(result, 13, "Full line length with tab expansion");
}

#[test]
fn test_line_length_expanded_no_tabs() {
    let line = "hello world";
    let result = line_length_expanded(line, 8);
    assert_eq!(result, 11, "Line without tabs");
}

// Note: AST utility tests that require complex AST construction are tested
// in integration tests. Here we focus on pure utility functions.
