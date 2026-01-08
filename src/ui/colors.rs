//! Colored terminal output utilities

use colored::*;

/// Check if colors should be enabled
pub fn should_colorize() -> bool {
    // Check if we're in a TTY and not in CI
    let is_tty = atty::is(atty::Stream::Stdout);
    let is_ci = std::env::var("CI").is_ok();
    let no_color = std::env::var("NO_COLOR").is_ok();
    is_tty && !is_ci && !no_color
}

/// Print an info message (green)
pub fn info(msg: &str) {
    if should_colorize() {
        println!("{}", msg.green().bold());
    } else {
        println!("[INFO] {msg}");
    }
}

/// Print a success message (green)
pub fn success(msg: &str) {
    if should_colorize() {
        println!("{}", format!("✓ {msg}").green().bold());
    } else {
        println!("[SUCCESS] {msg}");
    }
}

/// Print an error message (red)
pub fn error(msg: &str) {
    if should_colorize() {
        eprintln!("{}", format!("✗ {msg}").red().bold());
    } else {
        eprintln!("[ERROR] {msg}");
    }
}

/// Print a warning message (yellow)
pub fn warn(msg: &str) {
    if should_colorize() {
        println!("{}", format!("⚠ {msg}").yellow().bold());
    } else {
        println!("[WARN] {msg}");
    }
}

/// Print a build success message
pub fn build_success() {
    if should_colorize() {
        println!("{}", "BUILD SUCCESS".green().bold());
    } else {
        println!("[INFO] BUILD SUCCESS");
    }
}

/// Print a build failure message
pub fn build_failure() {
    if should_colorize() {
        eprintln!("{}", "BUILD FAILURE".red().bold());
    } else {
        eprintln!("[ERROR] BUILD FAILURE");
    }
}

/// Print formatted dependency information
pub fn dependency_info(group: &str, artifact: &str, version: &str) {
    if should_colorize() {
        println!("  {}:{}:{}", 
            group.cyan(), 
            artifact.cyan(), 
            version.yellow()
        );
    } else {
        println!("  {group}:{artifact}:{version}");
    }
}

