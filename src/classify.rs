//! # Section 05 — What is this path?
//!
//! What you'll learn: `std::fs::symlink_metadata` and `match`.
//! Builds on: Section 03 (paths as `PathBuf`) and Section 01 (`Result`).
//! Used by: Section 07, the read-only plan.
//! Functions: F05.
//!
//! `symlink_metadata` looks at the path itself. `metadata` would follow a
//! symlink and describe the target. We use `symlink_metadata` so a link is
//! reported as a link.
//!
//! A symlink in the *middle* of a path is refused. On macOS, `/tmp` is a
//! symlink to `/private/tmp`. rmbin will not silently walk through it. Pass
//! the path that does not contain the link, or name the link itself when you
//! only want the link removed.
//!
//! The last part of a path may be a symlink. The plan then says "delete the
//! link only".

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::error::{Error, Result};
use crate::protect::Guard;

/// What a path is, without following a final symlink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// A normal file. `len` is the byte length. `hard_links` is how many names
    /// point at the same bytes. On Windows 0.0.1 this count is always 1,
    /// because the stable library cannot read it. The limits banner says so.
    File {
        /// Byte length.
        len: u64,
        /// Number of directory names for these bytes.
        hard_links: u64,
    },
    /// A real directory, not a symlink to one.
    Directory {
        /// Unix device id, used to avoid walking into another filesystem.
        /// `None` on Windows in 0.0.1.
        device: Option<u64>,
    },
    /// A symlink. The plan removes the link, not the target.
    Symlink,
    /// A fifo, socket, device, or other node we do not overwrite.
    Other {
        /// Short fixed reason.
        reason: &'static str,
    },
}

/// Turn `user_path` into an absolute path and refuse to walk through a symlink.
///
/// `.` and `..` are applied as the walk goes. `..` does not jump out through
/// a symlink we already refused.
///
/// # Errors
///
/// Missing folders, a symlink in the middle, or a protected place that the
/// symlink would have led to.
pub fn resolve_leaf(user_path: &Path, guard: &Guard) -> Result<PathBuf> {
    if user_path.as_os_str().is_empty() {
        return Err(Error::EmptyListItem);
    }
    let absolute = if user_path.is_absolute() {
        user_path.to_path_buf()
    } else {
        let current = std::env::current_dir().map_err(|source| Error::Io { path: None, source })?;
        current.join(user_path)
    };
    let components: Vec<Component<'_>> = absolute.components().collect();
    let mut built = PathBuf::new();
    for (index, component) in components.iter().enumerate() {
        match component {
            Component::CurDir => continue,
            Component::ParentDir => {
                built.pop();
                continue;
            }
            Component::Prefix(_) | Component::RootDir => {
                built.push(component.as_os_str());
                continue;
            }
            Component::Normal(name) => built.push(name),
        }
        if !later_step_remains(&components, index) {
            continue;
        }
        match fs::symlink_metadata(&built) {
            Ok(meta) if meta.file_type().is_symlink() || is_reparse(&meta) => {
                let points_at = fs::read_link(&built).ok();
                let rest = &components[index + 1..];
                // Refuse with the protected-path error when the link would land
                // in a denied place. Otherwise say we will not follow the link.
                // Do not treat the link's own folder (for example `/tmp`) as the
                // thing the user asked to delete.
                if let Some(combined) = join_link(&built, points_at.as_ref(), rest) {
                    if let Some(rule) = guard.denial_root(&combined) {
                        return Err(Error::Protected {
                            path: combined,
                            rule,
                        });
                    }
                }
                return Err(Error::SymlinkInPath {
                    typed: absolute,
                    link: built,
                    points_at,
                });
            }
            Ok(_) => {}
            Err(source) if source.kind() == io::ErrorKind::NotFound => {
                return Err(Error::NotFound { path: built });
            }
            Err(source) => return Err(Error::io(&built, source)),
        }
    }
    if built.as_os_str().is_empty() {
        return Err(Error::NotFound {
            path: user_path.to_path_buf(),
        });
    }
    Ok(built)
}

fn later_step_remains(components: &[Component<'_>], index: usize) -> bool {
    components
        .iter()
        .skip(index + 1)
        .any(|component| !matches!(component, Component::CurDir))
}

fn join_link(link: &Path, target: Option<&PathBuf>, rest: &[Component<'_>]) -> Option<PathBuf> {
    let target = target?;
    let mut combined = if target.is_absolute() {
        target.clone()
    } else {
        link.parent()?.join(target)
    };
    for component in rest {
        match component {
            Component::Normal(name) => combined.push(name),
            Component::ParentDir => {
                combined.pop();
            }
            Component::CurDir | Component::Prefix(_) | Component::RootDir => {}
        }
    }
    Some(combined)
}

/// Classify the final path. Does not follow a final symlink.
///
/// # Errors
///
/// Returns [`Error::NotFound`] when the path is missing, or [`Error::Io`] for
/// other filesystem errors.
pub fn classify(path: &Path) -> Result<Kind> {
    let meta = fs::symlink_metadata(path).map_err(|source| {
        if source.kind() == io::ErrorKind::NotFound {
            Error::NotFound {
                path: path.to_path_buf(),
            }
        } else {
            Error::io(path, source)
        }
    })?;
    let file_type = meta.file_type();
    if file_type.is_symlink() {
        return Ok(Kind::Symlink);
    }
    if is_reparse(&meta) {
        return Ok(Kind::Other {
            reason: "reparse point",
        });
    }
    if file_type.is_dir() {
        return Ok(Kind::Directory {
            device: device_of(&meta),
        });
    }
    if file_type.is_file() {
        return Ok(Kind::File {
            len: meta.len(),
            hard_links: hard_links(&meta),
        });
    }
    Ok(Kind::Other {
        reason: other_reason(&file_type),
    })
}

pub(crate) fn is_reparse(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
        meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    {
        let _ = meta;
        false
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

fn hard_links(meta: &fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        meta.nlink()
    }
    #[cfg(not(unix))]
    {
        // Stable Rust on Windows cannot read this number without an unsafe call.
        // Section 11's banner says a second hard-link name would be overwritten.
        let _ = meta;
        1
    }
}

fn other_reason(file_type: &fs::FileType) -> &'static str {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        if file_type.is_fifo() {
            "fifo"
        } else if file_type.is_socket() {
            "socket"
        } else if file_type.is_block_device() {
            "block device"
        } else if file_type.is_char_device() {
            "character device"
        } else {
            "not a regular file"
        }
    }
    #[cfg(not(unix))]
    {
        let _ = file_type;
        "not a regular file"
    }
}

/// Two device ids differ, so a walk should not enter the child.
#[must_use]
pub fn different_device(root: u64, child: u64) -> bool {
    root != child
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{classify, different_device, resolve_leaf, Kind};
    use crate::protect::Guard;

    fn temp_dir() -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("rmbin-classify-{n}-{nanos}"));
        fs::create_dir_all(&path).expect("temp");
        fs::canonicalize(&path).unwrap_or(path)
    }

    #[test]
    fn file_directory_and_symlink() {
        let dir = temp_dir();
        let file = dir.join("notes.txt");
        fs::write(&file, b"abc").expect("write");
        let sub = dir.join("sub");
        fs::create_dir(&sub).expect("dir");
        let link = dir.join("link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&file, &link).expect("symlink");
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&file, &link).expect("symlink");

        match classify(&file).expect("file") {
            Kind::File { len, .. } => assert_eq!(len, 3),
            other => panic!("expected file, got {other:?}"),
        }
        assert!(matches!(
            classify(&sub).expect("dir"),
            Kind::Directory { .. }
        ));
        assert_eq!(classify(&link).expect("link"), Kind::Symlink);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn intermediate_symlink_is_refused() {
        let dir = temp_dir();
        let real = dir.join("real");
        fs::create_dir(&real).expect("dir");
        fs::write(real.join("file.txt"), b"secret").expect("write");
        let link = dir.join("linkdir");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&real, &link).expect("symlink");
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&real, &link).expect("symlink");

        let through = link.join("file.txt");
        let error = resolve_leaf(&through, &Guard::load()).expect_err("refused");
        let text = error.to_string();
        assert!(text.contains("refusing to follow symlink"), "{text}");
        assert_eq!(fs::read(real.join("file.txt")).expect("read"), b"secret");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn dot_dot_stays_in_the_tree() {
        let dir = temp_dir();
        let file = dir.join("notes.txt");
        fs::write(&file, b"abc").expect("write");
        let via = dir.join("missing").join("..").join("notes.txt");
        // `missing` does not exist, so the walk stops there.
        assert!(resolve_leaf(&via, &Guard::load()).is_err());
        let via_real = dir.join(".").join("notes.txt");
        let resolved = resolve_leaf(&via_real, &Guard::load()).expect("resolve");
        assert_eq!(resolved, file);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn different_device_is_a_plain_compare() {
        assert!(different_device(1, 2));
        assert!(!different_device(7, 7));
    }
}
