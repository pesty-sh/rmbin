//! # Section 24 — Which planned names are still in a list
//!
//! What you'll learn: comparing two lists.
//! Builds on: Section 01.
//! Used by: Section 20, after the paths have been jailed to the sandbox.
//! Functions: F23, the list half. This function does not look at the disk.
//!
//! The caller passes the names it already has. Names that appear in both
//! lists are still present. This does not delete the names that are missing.

/// Planned names that also appear in `present`, in planned order.
#[must_use]
pub fn still_present<'a>(planned: &'a [&str], present: &[&str]) -> Vec<&'a str> {
    planned
        .iter()
        .copied()
        .filter(|name| present.contains(name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::still_present;

    #[test]
    fn only_names_in_both_lists_are_returned() {
        let planned = ["a.txt", "b.txt", "c.txt"];
        let present = ["c.txt", "a.txt"];
        let kept = still_present(&planned, &present);
        assert_eq!(kept, vec!["a.txt", "c.txt"]);
        let shown = kept.join(" ");
        assert!(!shown.contains("EXAMPLE_TOKEN_123"));
    }
}
