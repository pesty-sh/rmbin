//! # Section 14 — The name a later version would use
//!
//! What you'll learn: turning bytes into a new file name, with no disk calls.
//! Builds on: Section 04, the pattern.
//! Used by: Section 20, which prints the name. It does not rename.
//! Functions: F18, the name only.
//!
//! A later version renames a file to a name of the same length, built from
//! the pattern, and only then deletes it. The new name is not the old name
//! with a prefix. This file only builds that name. Creating it on disk is
//! a later version.

use std::ffi::{OsStr, OsString};
use std::path::Path;

use crate::pattern::Pattern;

/// The name a later version would rename `original` to.
///
/// The result has the same number of bytes as `original` on Unix. It is
/// made from the pattern. Characters that are not safe in a file name
/// become `x`. `.`, `..`, Windows reserved names, and a copy of the old
/// name are adjusted so the result is a single ordinary name.
#[must_use]
pub fn pattern_name(original: &OsStr, pattern: &Pattern) -> OsString {
    let bytes = name_bytes(original);
    let mut out = Vec::with_capacity(bytes.len());
    for index in 0..bytes.len() {
        out.push(safe_name_byte(pattern.byte_at(index as u64)));
    }
    finish(&mut out, &bytes);
    from_ascii_bytes(&out)
}

/// A plan phrase. `directory` is not used. The name is not created there.
///
/// The argument is accepted so the call site shows which folder the later
/// rename would happen in, and so this version cannot "helpfully" create it.
#[must_use]
pub fn plan_phrase(directory: &Path, original: &OsStr, pattern: &Pattern) -> String {
    let _ = directory;
    let new_name = pattern_name(original, pattern);
    format!(
        "later rename, not done: {} -> {}",
        original.to_string_lossy(),
        new_name.to_string_lossy()
    )
}

fn name_bytes(original: &OsStr) -> Vec<u8> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        original.as_bytes().to_vec()
    }
    #[cfg(not(unix))]
    {
        original.to_string_lossy().as_bytes().to_vec()
    }
}

fn from_ascii_bytes(bytes: &[u8]) -> OsString {
    // `safe_name_byte` only keeps ASCII, so this is a real file name.
    match std::str::from_utf8(bytes) {
        Ok(text) => OsString::from(text),
        Err(_) => OsString::from("x".repeat(bytes.len().max(1))),
    }
}

fn safe_name_byte(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'.' | b'_' | b'-' => byte,
        _ => b'x',
    }
}

fn finish(out: &mut Vec<u8>, original: &[u8]) {
    if out.is_empty() {
        out.push(b'x');
    }
    if out.as_slice() == b"." || out.as_slice() == b".." {
        let len = original.len().max(1);
        out.clear();
        out.resize(len, b'x');
    }
    avoid_reserved(out);
    if out.as_slice() == original && !out.is_empty() {
        let last = out.len() - 1;
        out[last] = if out[last] == b'x' { b'y' } else { b'x' };
    }
}

fn avoid_reserved(bytes: &mut [u8]) {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return;
    };
    let stem = text.split('.').next().unwrap_or(text);
    let upper = stem.to_ascii_uppercase();
    let reserved = matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL") || is_com_or_lpt(&upper);
    if reserved && !bytes.is_empty() {
        bytes[0] = if bytes[0] == b'x' { b'y' } else { b'x' };
    }
}

fn is_com_or_lpt(upper: &str) -> bool {
    let bytes = upper.as_bytes();
    bytes.len() == 4
        && (upper.starts_with("COM") || upper.starts_with("LPT"))
        && bytes[3].is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::Path;

    use super::{pattern_name, plan_phrase};
    use crate::pattern::Pattern;

    #[test]
    fn lol_name_matches_the_length_and_drops_the_old_name() {
        let pattern = Pattern::from_text("lol").expect("pattern");
        let new_name = pattern_name(OsStr::new("secret.txt"), &pattern);
        // 10 bytes of "lol": l o l l o l l o l l.
        assert_eq!(new_name, OsStr::new("lollolloll"));
        let phrase = plan_phrase(Path::new("/tmp"), OsStr::new("secret.txt"), &pattern);
        assert!(phrase.contains("not done"));
        assert!(phrase.contains("lollolloll"));
        assert!(!phrase.contains("EXAMPLE_TOKEN"));
    }

    #[test]
    fn slash_in_the_pattern_cannot_become_a_path() {
        let pattern = Pattern::from_text("a/b").expect("pattern");
        let new_name = pattern_name(OsStr::new("abcd"), &pattern);
        let text = new_name.to_string_lossy();
        assert_eq!(text.len(), 4);
        assert!(!text.contains('/'));
        assert!(!text.contains('\\'));
    }

    #[test]
    fn reserved_windows_name_is_adjusted() {
        let pattern = Pattern::from_text("CON").expect("pattern");
        let new_name = pattern_name(OsStr::new("CON"), &pattern);
        assert_ne!(new_name, OsStr::new("CON"));
        assert_eq!(new_name.to_string_lossy().len(), 3);
    }

    #[test]
    fn same_name_as_the_pattern_is_changed() {
        let pattern = Pattern::from_text("lol").expect("pattern");
        let new_name = pattern_name(OsStr::new("lollolloll"), &pattern);
        assert_ne!(new_name, OsStr::new("lollolloll"));
        assert_eq!(new_name.to_string_lossy().len(), 10);
    }
}
