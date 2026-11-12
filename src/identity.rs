//! # Section 12 — The file's id card
//!
//! What you'll learn: a struct that records a few numbers from `metadata`.
//! Builds on: Section 05, which already asks what a path is.
//! Used by: Section 20, which prints the card. It does not write.
//! Functions: the read-only half of F19 and F28.
//!
//! A path is a name. The id card is how we will know, in a later version,
//! that the name still points at the same bytes. We record:
//! - the length in bytes
//! - how many names share those bytes (hard links)
//! - the device and the inode, on macOS and Linux
//!
//! An inode is the file's number on that disk. If the number changes, the
//! name was swapped for a different file. This version only reads the card
//! and prints it. It does not open the file for writing.

use std::fmt;
use std::fs;
use std::path::Path;

use crate::error::{Error, Result};

/// Numbers that identify a file without reading its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdCard {
    /// Byte length. Not the contents.
    pub len: u64,
    /// How many directory names point at these bytes.
    ///
    /// On Windows this stays `1`. The stable library cannot read the real
    /// count without an unsafe call. The banner says so.
    pub links: u64,
    /// Unix device id. `None` on Windows.
    pub dev: Option<u64>,
    /// Unix inode. `None` on Windows.
    pub ino: Option<u64>,
}

impl fmt::Display for IdCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", summary(self))
    }
}

/// One short line for the plan. It never includes file bytes.
#[must_use]
pub fn summary(card: &IdCard) -> String {
    #[cfg(windows)]
    {
        format!(
            "{} bytes. Link count and inode are not read on this Windows build.",
            card.len
        )
    }
    #[cfg(not(windows))]
    {
        let links = if card.links == 1 {
            "1 link".to_owned()
        } else {
            format!("{} links", card.links)
        };
        match (card.dev, card.ino) {
            (Some(dev), Some(ino)) => {
                format!("{} bytes, {links}, device {dev}, inode {ino}", card.len)
            }
            _ => format!("{} bytes, {links}", card.len),
        }
    }
}

/// Read the id card. Does not follow a final symlink. Does not read bytes.
///
/// # Errors
///
/// The path is missing, or the operating system refused the lookup.
pub fn read(path: &Path) -> Result<IdCard> {
    let meta = fs::symlink_metadata(path).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            Error::NotFound {
                path: path.to_path_buf(),
            }
        } else {
            Error::io(path, source)
        }
    })?;
    Ok(IdCard {
        len: meta.len(),
        links: links_of(&meta),
        dev: device_of(&meta),
        ino: inode_of(&meta),
    })
}

fn links_of(meta: &fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        meta.nlink()
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        1
    }
}

fn device_of(meta: &fs::Metadata) -> Option<u64> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Some(meta.dev())
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        None
    }
}

fn inode_of(meta: &fs::Metadata) -> Option<u64> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Some(meta.ino())
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        None
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::read;
    use crate::sandbox::Scratch;

    #[test]
    fn card_records_length_and_not_the_bytes() {
        let scratch = Scratch::new("identity").expect("sandbox");
        let dir = scratch.path();
        let file = dir.join("notes.txt");
        let secret = b"EXAMPLE_TOKEN_123";
        fs::write(&file, secret).expect("write");
        let card = read(&file).expect("card");
        assert_eq!(card.len, secret.len() as u64);
        let shown = card.to_string();
        assert!(!shown.contains("EXAMPLE_TOKEN_123"), "{shown}");
        assert!(shown.contains(&card.len.to_string()), "{shown}");
        #[cfg(unix)]
        {
            assert!(card.dev.is_some());
            assert!(card.ino.is_some());
            assert_eq!(card.links, 1);
        }
    }
}
