//! # Section 19 — What a line is allowed to say
//!
//! What you'll learn: a function whose arguments cannot be file bytes.
//! Builds on: Section 01.
//! Used by: Section 20, when it adds a short line under a path.
//! Functions: F17, without a log file.
//!
//! A log file is a place a secret could land, so this version does not
//! write one. The report is the terminal. This function takes a fixed
//! label and a path. There is no parameter for the bytes inside the file,
//! and none for the environment. If a value is not an argument, this
//! function cannot print it.

use std::path::Path;

/// A report line made only from a label and a path.
#[must_use]
pub fn line(label: &str, path: &Path) -> String {
    format!("{label}  {}", path.display())
}

/// True when `report` does not contain `secret`.
///
/// An empty secret is not a leak. There is no file argument, so this cannot
/// read a disk. Callers pass a secret they already know, such as a test token.
#[must_use]
pub fn keeps_secret(report: &str, secret: &str) -> bool {
    secret.is_empty() || !report.contains(secret)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::line;

    #[test]
    fn a_line_is_the_label_and_the_path() {
        let text = line("id card", Path::new("/tmp/notes.txt"));
        assert!(text.contains("id card"));
        assert!(text.contains("/tmp/notes.txt"));
        assert!(!text.contains("EXAMPLE_TOKEN_123"));
        assert!(super::keeps_secret(&text, "EXAMPLE_TOKEN_123"));
        assert!(!super::keeps_secret(
            "token EXAMPLE_TOKEN_123",
            "EXAMPLE_TOKEN_123"
        ));
        assert!(super::keeps_secret("anything", ""));
    }
}
