//! # Section 09 — Overwrite, sync, truncate
//!
//! What you'll learn: `OpenOptions`, `Write`, `Seek`, and `sync_all`.
//! Builds on: Section 04 (the pattern) and Section 01 (errors).
//! Used by: Section 10, which deletes the name only after this succeeds.
//! Functions: F11, F12.
//!
//! The file is opened without `truncate`, so opening it does not throw the
//! old bytes away. We write exactly the original length, then call `sync_all`.
//! On macOS, Rust's standard library implements `sync_all` with `F_FULLFSYNC`,
//! which asks the drive for a full flush. An error here is fatal for that file:
//! we do not truncate it and we do not delete it.
//!
//! After a successful sync we truncate to zero and sync again. We still do not
//! delete. Deleting is Section 10.
//!
//! The final symlink is not followed. On Linux and macOS the open uses
//! `O_NOFOLLOW`. That number is not the same on every CPU: 32-bit ARM and
//! aarch64 use `0x8000`, and x86 uses `0x20000`. On Windows the open uses
//! `FILE_FLAG_OPEN_REPARSE_POINT`. Before any write, the open file is compared
//! with a `symlink_metadata` check so a followed link is refused.

use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::Path;

use crate::error::{Error, Result};
use crate::pattern::Pattern;

/// How many bytes to fill before one `write` call. The file's length does not
/// change because of this size; the last write is shorter when it needs to be.
const CHUNK: usize = 64 * 1024;

/// Open `path` for writing without following a final symlink.
///
/// # Errors
///
/// The file is unchanged when the open fails. The error says `NOT securely deleted`.
pub fn open_no_follow(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    // `read` is on so the length can be read back from this same handle.
    // We do not read the old contents into a buffer, and we never print them.
    // Look at the name before opening it. `symlink_metadata` does not follow
    // a final symlink, so this refuses the link even if the open flag is wrong.
    let named = std::fs::symlink_metadata(path).map_err(|source| {
        Error::NotSecurelyDeleted {
            path: path.to_path_buf(),
            reason: format!(
                "could not read the path before writing: {source}. The file was kept and was not changed by this step."
            ),
        }
    })?;
    if named.file_type().is_symlink() || crate::classify::is_reparse(&named) {
        return Err(Error::NotSecurelyDeleted {
            path: path.to_path_buf(),
            reason: "the path is a symlink. It was not overwritten.".to_owned(),
        });
    }
    options.read(true).write(true).create(false).truncate(false);
    apply_no_follow(&mut options);
    let file = options.open(path).map_err(|source| Error::NotSecurelyDeleted {
        path: path.to_path_buf(),
        reason: format!(
            "could not open the file for writing: {source}. The file was kept and was not changed by this step."
        ),
    })?;
    let meta = file.metadata().map_err(|source| Error::io(path, source))?;
    if meta.file_type().is_symlink() {
        return Err(Error::NotSecurelyDeleted {
            path: path.to_path_buf(),
            reason: "the path is a symlink. It was not overwritten.".to_owned(),
        });
    }
    // The open handle must be the same file the name pointed at. If they
    // differ, the open followed a link and nothing has been written yet.
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.dev() != named.dev() || meta.ino() != named.ino() {
            return Err(Error::NotSecurelyDeleted {
                path: path.to_path_buf(),
                reason: "the open file is not the named file. It was not overwritten.".to_owned(),
            });
        }
        if meta.nlink() > 1 {
            return Err(Error::NotSecurelyDeleted {
                path: path.to_path_buf(),
                reason:
                    "more than one hard link was visible before writing. The file was not changed."
                        .to_owned(),
            });
        }
    }
    Ok(file)
}

fn apply_no_follow(options: &mut OpenOptions) {
    // Windows: FILE_FLAG_OPEN_REPARSE_POINT is 0x00200000.
    // macOS: O_NOFOLLOW is 0x0100.
    // Linux x86: O_NOFOLLOW is 0400000 octal (0x20000), from asm-generic.
    // Linux ARM and aarch64: O_NOFOLLOW is 0100000 octal (0x8000).
    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let flag = if cfg!(any(target_arch = "aarch64", target_arch = "arm")) {
            0x0000_8000
        } else {
            0x0002_0000
        };
        options.custom_flags(flag);
    }
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(0x0000_0100);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000);
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        windows
    )))]
    {
        let _ = options;
    }
}

/// Write `pattern` from the start of `file` for exactly its current length, then sync.
///
/// The length is unchanged. This does not truncate and does not delete.
///
/// # Errors
///
/// Any write or sync error. The caller must not delete the file.
pub fn overwrite_in_place(path: &Path, file: &mut File, pattern: &Pattern) -> Result<u64> {
    let len = file
        .metadata()
        .map_err(|source| Error::io(path, source))?
        .len();
    file.seek(SeekFrom::Start(0))
        .map_err(|source| Error::io(path, source))?;
    let mut offset = 0u64;
    let mut buffer = vec![0u8; CHUNK];
    while offset < len {
        let remaining = len - offset;
        let size = if remaining > CHUNK as u64 {
            CHUNK
        } else {
            usize::try_from(remaining).map_err(|_| {
                Error::io(
                    path,
                    io::Error::new(io::ErrorKind::InvalidInput, "chunk does not fit"),
                )
            })?
        };
        if let Err(error) = pattern.fill(&mut buffer[..size], offset) {
            return Err(attach_path(path, error));
        }
        if let Err(source) = file.write_all(&buffer[..size]) {
            return Err(kept(
                path,
                format!(
                    "stopped during overwrite: {source}. The file was kept and was not deleted. It may already contain pattern bytes. Nothing was restored."
                ),
            ));
        }
        offset += size as u64;
    }
    if let Err(source) = file.sync_all() {
        return Err(kept(
            path,
            format!(
                "could not sync the overwrite: {source}. On macOS this asks for a full drive flush. The file was kept and was not deleted. It may already contain pattern bytes. Nothing was restored."
            ),
        ));
    }
    let now = file
        .metadata()
        .map_err(|source| Error::io(path, source))?
        .len();
    if now != len {
        return Err(kept(
            path,
            format!(
                "the length changed from {len} to {now} during the overwrite. The file was kept and was not truncated or deleted. Nothing was restored."
            ),
        ));
    }
    Ok(len)
}

/// Overwrite, sync, truncate to zero, and sync again. Does not delete the name.
///
/// # Errors
///
/// The name is kept. See the error text for whether the bytes may have changed.
pub fn overwrite_then_truncate(path: &Path, pattern: &Pattern) -> Result<()> {
    let mut file = open_no_follow(path)?;
    overwrite_in_place(path, &mut file, pattern)?;
    if let Err(source) = file.set_len(0) {
        return Err(kept(
            path,
            format!(
                "the overwrite was synced, but truncating failed: {source}. The file was kept and was not deleted. Nothing was restored."
            ),
        ));
    }
    if let Err(source) = file.sync_all() {
        return Err(kept(
            path,
            format!(
                "truncated the file, but the second sync failed: {source}. The file was kept and was not deleted. Nothing was restored."
            ),
        ));
    }
    Ok(())
}

fn kept(path: &Path, reason: String) -> Error {
    Error::NotSecurelyDeleted {
        path: path.to_path_buf(),
        reason,
    }
}

fn attach_path(path: &Path, error: Error) -> Error {
    match error {
        Error::NotSecurelyDeleted { reason, .. } => kept(path, reason),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, OpenOptions};
    use std::io::{Read, Seek, SeekFrom};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{overwrite_in_place, overwrite_then_truncate};
    use crate::pattern::Pattern;

    fn temp_dir() -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("rmbin-overwrite-{n}-{nanos}"));
        fs::create_dir_all(&path).expect("temp");
        fs::canonicalize(&path).unwrap_or(path)
    }

    fn make_writable(path: &std::path::Path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o644));
        }
        #[cfg(windows)]
        {
            if let Ok(meta) = fs::metadata(path) {
                let mut permissions = meta.permissions();
                #[allow(clippy::permissions_set_readonly_false)]
                {
                    permissions.set_readonly(false);
                }
                let _ = fs::set_permissions(path, permissions);
            }
        }
    }

    fn expected(pattern: &str, len: usize) -> Vec<u8> {
        let bytes = pattern.as_bytes();
        (0..len).map(|index| bytes[index % bytes.len()]).collect()
    }

    #[test]
    fn paint_keeps_the_exact_length_and_uses_the_pattern() {
        let dir = temp_dir();
        let path = dir.join("notes.txt");
        let original_len = (64 * 1024) + 10;
        let original = vec![b'Q'; original_len];
        fs::write(&path, &original).expect("write");
        let pattern = Pattern::from_text("lol").expect("pattern");
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open");
        let len = overwrite_in_place(&path, &mut file, &pattern).expect("paint");
        assert_eq!(len, original_len as u64);
        file.seek(SeekFrom::Start(0)).expect("seek");
        let mut got = Vec::new();
        file.read_to_end(&mut got).expect("read");
        assert_eq!(got.len(), original_len);
        assert_eq!(got, expected("lol", original_len));
        assert_eq!(
            fs::metadata(&path).expect("meta").len(),
            original_len as u64
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn custom_pattern_and_empty_file() {
        let dir = temp_dir();
        let path = dir.join("short.txt");
        fs::write(&path, b"abcdef").expect("write");
        let pattern = Pattern::from_text("101").expect("pattern");
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open");
        overwrite_in_place(&path, &mut file, &pattern).expect("paint");
        file.seek(SeekFrom::Start(0)).expect("seek");
        let mut got = Vec::new();
        file.read_to_end(&mut got).expect("read");
        assert_eq!(got, b"101101");

        let empty = dir.join("empty.txt");
        fs::write(&empty, b"").expect("empty");
        overwrite_then_truncate(&empty, &pattern).expect("truncate");
        assert_eq!(fs::metadata(&empty).expect("meta").len(), 0);
        assert!(empty.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn symlink_is_not_overwritten() {
        let dir = temp_dir();
        let target = dir.join("target.txt");
        fs::write(&target, b"secret").expect("write");
        let link = dir.join("link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &link).expect("symlink");
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&target, &link).expect("symlink");
        let pattern = Pattern::from_text("lol").expect("pattern");
        let error = overwrite_then_truncate(&link, &pattern).expect_err("refused");
        assert!(error.to_string().contains("NOT securely deleted"));
        assert_eq!(fs::read(&target).expect("target"), b"secret");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_only_file_is_kept_unchanged() {
        let dir = temp_dir();
        let path = dir.join("locked.txt");
        fs::write(&path, b"secret").expect("write");
        let mut permissions = fs::metadata(&path).expect("meta").permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&path, permissions).expect("chmod");
        let pattern = Pattern::from_text("lol").expect("pattern");
        let error = overwrite_then_truncate(&path, &pattern);
        // Running as root can ignore the mode bit. Only assert when the open fails.
        if fs::OpenOptions::new().write(true).open(&path).is_err() {
            let error = error.expect_err("read only");
            assert!(error.to_string().contains("NOT securely deleted"));
            assert_eq!(fs::read(&path).expect("read"), b"secret");
        }
        make_writable(&path);
        let _ = fs::remove_dir_all(&dir);
    }
}
