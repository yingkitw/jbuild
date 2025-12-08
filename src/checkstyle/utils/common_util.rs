//! Common utility functions for Checkstyle-rs

/// Calculate the length of a string with tabs expanded to spaces
///
/// This is equivalent to CommonUtil.lengthExpandedTabs in Java Checkstyle.
///
/// # Arguments
/// * `line` - The line of text
/// * `column` - The column number (0-based) up to which to calculate
/// * `tab_width` - The width of a tab character in spaces
///
/// # Returns
/// The expanded length in columns
pub fn length_expanded_tabs(line: &str, column: usize, tab_width: usize) -> usize {
    let mut expanded_length = 0;
    let chars: Vec<char> = line.chars().collect();
    let max_pos = column.min(chars.len());

    for i in 0..max_pos {
        match chars[i] {
            '\t' => {
                // Expand tab to next tab stop
                let next_tab_stop = ((expanded_length / tab_width) + 1) * tab_width;
                expanded_length = next_tab_stop;
            }
            _ => {
                expanded_length += 1;
            }
        }
    }

    expanded_length
}

/// Calculate the expanded length of an entire line
pub fn line_length_expanded(line: &str, tab_width: usize) -> usize {
    length_expanded_tabs(line, line.len(), tab_width)
}
