//! # Section 15 — Sparse and compressed files
//!
//! What you'll learn: comparing two numbers from `metadata`.
//! Builds on: Section 05.
//! Used by: Section 20, which marks these files as skip. It does not write.
//! Functions: F24, the read-only check.
//!
//! A sparse file's length is larger than the blocks the disk has given it.
//! A compressed file can look the same. Overwriting one can miss the old
//! blocks, or it can allocate the holes and fill the disk. A later version
//! skips them and says so. This version only detects them.
//!
//! `blocks` is a count of 512-byte units. That is a Unix convention, even
//! when the disk's own block is 4096 bytes.

use std::fs;
use std::io;
use std::path::Path;

use crate::error::{Error, Result};

/// Why this file should not be overwritten, or `None` when it looks ordinary.
///
/// # Errors
///
/// The path cannot be looked up. A missing path is [`Error::NotFound`].
pub fn skip_reason(path: &Path) -> Result<Option<&'static str>> {
    let meta = fs::symlink_metadata(path).map_err(|source| {
        if source.kind() == io::ErrorKind::NotFound {
            Error::NotFound {
                path: path.to_path_buf(),
            }
        } else {
            Error::io(path, source)
        }
    })?;
    if meta.file_type().is_symlink() || !meta.is_file() {
        return Ok(None);
    }
    if compressed(&meta) {
        return Ok(Some(
            "compressed file. A later version would skip it. This version did not write.",
        ));
    }
    if fewer_blocks_than_length(&meta) {
        return Ok(Some(
            "fewer disk blocks than the length (sparse or compressed). A later version would skip it. This version did not write.",
        ));
    }
    Ok(None)
}

fn fewer_blocks_than_length(meta: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        meta.blocks().saturating_mul(512) < meta.len()
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        false
    }
}

fn compressed(meta: &fs::Metadata) -> bool {
    #[cfg(target_os = "macos")]
    {
        use std::os::darwin::fs::MetadataExt;
        // UF_COMPRESSED, from the macOS stat flags. A compressed file.
        const UF_COMPRESSED: u32 = 0x0000_0020;
        meta.st_flags() & UF_COMPRESSED != 0
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = meta;
        false
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::skip_reason;

    fn temp_dir() -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("rmbin-sparse-{n}-{nanos}"));
        fs::create_dir_all(&path).expect("temp");
        fs::canonicalize(&path).unwrap_or(path)
    }

    #[test]
    fn a_small_file_is_not_called_sparse() {
        let dir = temp_dir();
        let file = dir.join("notes.txt");
        fs::write(&file, b"hello").expect("write");
        assert_eq!(skip_reason(&file).expect("reason"), None);
        assert_eq!(fs::read(&file).expect("read"), b"hello");
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_hole_is_reported_and_left_alone() {
        let dir = temp_dir();
        let file = dir.join("hole.bin");
        let handle = File::create(&file).expect("create");
        handle.set_len(1_000_000).expect("len");
        drop(handle);
        let reason = skip_reason(&file).expect("reason");
        assert!(reason.is_some(), "expected a sparse file");
        assert_eq!(fs::metadata(&file).expect("meta").len(), 1_000_000);
        let _ = fs::remove_dir_all(&dir);
    }
}
