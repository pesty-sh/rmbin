//! # Section 22 — The plan as text, and a fingerprint
//!
//! What you'll learn: turning a list of lines into one string, then a number.
//! Builds on: Section 07, which builds the lines.
//! Used by: Section 20, which prints the fingerprint.
//! Functions: F21, without a plan file and without `apply`.
//!
//! A plan file on disk is another place a path list could be copied. This
//! version keeps the text in memory and prints a fingerprint of that text.
//! The fingerprint is not a seal and not a warranty. It changes when the
//! plan text changes. It is not a hash of anyone's file contents, because
//! the file contents are not an argument.

/// Join plan lines with newlines. No extra field is accepted.
#[must_use]
pub fn join(lines: &[String]) -> String {
    lines.join("\n")
}

/// A hex fingerprint of `text`.
///
/// This is FNV-1a, 64 bits. It is a checksum of the plan text so two runs
/// can be told apart. It is not a cryptographic signature.
#[must_use]
pub fn fingerprint(text: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::{fingerprint, join};

    #[test]
    fn the_fingerprint_covers_the_plan_text_and_not_a_secret() {
        let lines = vec!["would overwrite 4 bytes  notes.txt".to_owned()];
        let text = join(&lines);
        let print = format!("Plan fingerprint: {}", fingerprint(&text));
        assert!(print.contains("Plan fingerprint"));
        assert!(!print.contains("EXAMPLE_TOKEN_123"));
        assert_ne!(fingerprint(&text), fingerprint("other"));
        assert_eq!(fingerprint(&text), fingerprint(&text));
    }
}
