//! # Section 04 — The repeating pattern
//!
//! What you'll learn: a slice of bytes (`&[u8]`), the remainder operator `%`,
//! and `Result` for one bad input.
//! Builds on: L0 strings and slices. A `String` owns its text. A `&str` borrows it.
//! Used by: Section 09, which writes the pattern into a file.
//! Functions: F03.
//!
//! Byte number `i` in the file becomes `pattern[i % pattern.len()]`.
//! `%` is the remainder after division, so the text repeats.
//! `lol` at offsets 0, 1, 2, 3 is `l`, `o`, `l`, `l`.
//!
//! The bytes are the UTF-8 bytes of the text you passed. `101` is the three
//! bytes `1`, `0`, and `1`. This is never random data.

use crate::error::{Error, Result};

/// The text written over every file in one run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    text: String,
}

impl Pattern {
    /// Build a pattern from text the user typed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyPattern`] when `text` is empty.
    pub fn from_text(text: impl Into<String>) -> Result<Self> {
        let text = text.into();
        if text.is_empty() {
            return Err(Error::EmptyPattern);
        }
        Ok(Self { text })
    }

    /// The pattern as text, for the plan line.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The bytes that get written. A `u8` is one byte, 0 to 255.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.text.as_bytes()
    }

    /// The byte that belongs at `index` in a file.
    #[must_use]
    pub fn byte_at(&self, index: u64) -> u8 {
        let bytes = self.as_bytes();
        // `len` is at least 1 because `from_text` rejects "".
        let len = bytes.len() as u64;
        let slot = index % len;
        match usize::try_from(slot) {
            Ok(slot) => bytes[slot],
            // `slot` is smaller than `len`, and `len` already fits in `usize`.
            Err(_) => bytes[0],
        }
    }

    /// Fill `dest` with pattern bytes starting at file offset `start`.
    ///
    /// # Errors
    ///
    /// Returns an error if `start + offset` does not fit in a `u64`.
    pub fn fill(&self, dest: &mut [u8], start: u64) -> Result<()> {
        for (offset, slot) in dest.iter_mut().enumerate() {
            let Some(index) = start.checked_add(offset as u64) else {
                return Err(Error::NotSecurelyDeleted {
                    path: std::path::PathBuf::from("<pattern>"),
                    reason: "file offset does not fit in a 64-bit number".to_owned(),
                });
            };
            *slot = self.byte_at(index);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Pattern;

    #[test]
    fn lol_repeats_by_offset() {
        let pattern = Pattern::from_text("lol").expect("pattern");
        let expected = [b'l', b'o', b'l', b'l', b'o', b'l', b'l', b'o', b'l', b'l'];
        for (index, byte) in expected.iter().enumerate() {
            assert_eq!(pattern.byte_at(index as u64), *byte);
        }
    }

    #[test]
    fn custom_pattern_101() {
        let pattern = Pattern::from_text("101").expect("pattern");
        assert_eq!(pattern.byte_at(0), b'1');
        assert_eq!(pattern.byte_at(1), b'0');
        assert_eq!(pattern.byte_at(2), b'1');
        assert_eq!(pattern.byte_at(3), b'1');
    }

    #[test]
    fn empty_pattern_is_an_error() {
        assert!(Pattern::from_text("").is_err());
    }

    #[test]
    fn unicode_uses_utf8_bytes() {
        // é is the two UTF-8 bytes 0xC3 0xA9. We repeat those bytes, not characters.
        let pattern = Pattern::from_text("é").expect("pattern");
        assert_eq!(pattern.as_bytes(), &[0xC3, 0xA9]);
        assert_eq!(pattern.byte_at(0), 0xC3);
        assert_eq!(pattern.byte_at(1), 0xA9);
        assert_eq!(pattern.byte_at(2), 0xC3);
    }

    #[test]
    fn fill_matches_byte_at() {
        let pattern = Pattern::from_text("lol").expect("pattern");
        let mut buf = [0u8; 8];
        pattern.fill(&mut buf, 5).expect("fill");
        for (offset, byte) in buf.iter().enumerate() {
            assert_eq!(*byte, pattern.byte_at(5 + offset as u64));
        }
    }
}
