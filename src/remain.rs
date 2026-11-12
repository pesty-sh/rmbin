//! # Section 18 — What may still hold the old bytes
//!
//! What you'll learn: a fixed list of sentences.
//! Builds on: Section 11, the limits banner.
//! Used by: Section 20, every run.
//! Functions: F23, the "what may remain" half. We do not check that a
//! delete worked, because this version does not delete.
//!
//! The list is static. It does not name the user's files and it does not
//! read them.

/// Lines printed after the plan. They are the same on every run.
pub const LINES: &[&str] = &[
    "Nothing was removed. The paths above are still on disk.",
    "A later overwrite would still miss snapshots, backups, SSD remapped blocks, copy-on-write copies, git history, caches, editor history, logs, swap, and CI artefacts.",
];

#[cfg(test)]
mod tests {
    use super::LINES;

    #[test]
    fn the_lines_admit_the_limits_and_that_we_did_not_delete() {
        let joined = LINES.join(" ");
        assert!(joined.contains("Nothing was removed"));
        assert!(joined.contains("snapshots"));
        assert!(!joined.contains("EXAMPLE_TOKEN"));
    }
}
