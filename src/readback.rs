//! # Section 13 — Read the bytes back (later)
//!
//! What you'll learn: a function that returns text, and nothing else.
//! Builds on: Section 04, the pattern. A later version compares file bytes
//! with `pattern.byte_at`.
//! Used by: Section 20, which prints this sentence.
//! Functions: F15, described only.
//!
//! A later version, after it has overwritten a file and called `sync_all`,
//! reads the file from the start and checks each byte. If one byte does not
//! match the pattern, it keeps the file and does not delete it. The message
//! would say the offset, never the byte, because the byte might be a secret.
//!
//! That read can come from the page cache. It is not proof the disk stored
//! the pattern. This version does not read file contents at all.

/// The plan line for the read-back step.
///
/// There is no path and no buffer argument. This function cannot leak a file.
#[must_use]
pub fn later_step() -> &'static str {
    "Read-back is later. A later version would compare visible bytes with the pattern and keep the file if they differ. It would not print the bytes. The read can come from the page cache, not the disk. This version does not read file contents."
}

#[cfg(test)]
mod tests {
    use super::later_step;

    #[test]
    fn the_sentence_does_not_invite_a_read() {
        let text = later_step();
        assert!(text.contains("does not read file contents"));
        assert!(text.contains("page cache"));
        assert!(!text.contains("EXAMPLE_TOKEN"));
    }
}
