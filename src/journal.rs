//! # Section 23 — A journal line that says the work was not done
//!
//! What you'll learn: formatting one record.
//! Builds on: Section 01.
//! Used by: Section 20, which prints the line and does not write a file.
//! Functions: F22, the line only. There is no resume of a wipe.
//!
//! A journal file would be a copy of the path list. This version does not
//! create one. The line says the path was planned and not changed.

use std::path::Path;

/// One journal line. It names the path and says nothing was done.
#[must_use]
pub fn planned(path: &Path) -> String {
    format!("planned, not done  {}", path.display())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::planned;

    #[test]
    fn the_line_does_not_claim_a_delete_or_carry_file_bytes() {
        let line = planned(Path::new("notes.txt"));
        assert!(line.contains("planned, not done"));
        assert!(line.contains("notes.txt"));
        assert!(!line.contains("securely deleted file"));
        assert!(!line.contains("EXAMPLE_TOKEN_123"));
    }
}
