//! # Section 16 — Which filesystem, and a snapshot warning
//!
//! What you'll learn: reading a small text file the kernel publishes.
//! Builds on: Section 06, which already reads `/proc/mounts` for the deny list.
//! Used by: Section 17 and Section 20.
//! Functions: F25, report only.
//!
//! `/proc/mounts` is a text list. Each line names a device, a folder, and
//! a filesystem type such as `ext4`. We do not run a program and we do not
//! change a mount. Snapshots are not deleted. This version does not write.

use std::fs;
use std::path::{Path, PathBuf};

/// One mount line we understood.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    /// The device column, such as `/dev/vdc` or `tmpfs`.
    pub device: String,
    /// Where it is mounted.
    pub point: PathBuf,
    /// The filesystem type, such as `ext4`.
    pub kind: String,
}

/// Read `/proc/mounts` on Linux. Anywhere else, return an empty list.
#[must_use]
pub fn load() -> Vec<Mount> {
    #[cfg(target_os = "linux")]
    {
        read_proc_mounts()
    }
    #[cfg(not(target_os = "linux"))]
    {
        Vec::new()
    }
}

/// The mount that contains `path`, when we can see one.
///
/// The longest mount point wins. `/` contains everything, and `/tmp` is
/// a better answer for a file in `/tmp` when both are listed.
#[must_use]
pub fn best(path: &Path) -> Option<Mount> {
    let mounts = load();
    best_of(path, &mounts)
}

/// One sentence for the plan. It does not claim a snapshot was removed.
#[must_use]
pub fn note_for(path: &Path) -> String {
    match best(path) {
        Some(mount) => format!(
            "{} is on {} ({}).{} Snapshots and backups are not deleted. This version does not change the disk.",
            path.display(),
            mount.kind,
            mount.device,
            extra(&mount.kind)
        ),
        None => format!(
            "Could not see which filesystem holds {}. Snapshots and backups are not deleted. This version does not change the disk.",
            path.display()
        ),
    }
}

fn extra(kind: &str) -> &'static str {
    match kind {
        "btrfs" | "zfs" => {
            " This filesystem can keep snapshots. They are not listed and not deleted."
        }
        "overlay" | "overlayfs" => " An overlay can hide a lower copy. That copy is not deleted.",
        _ => "",
    }
}

/// Pick from a list you already loaded. Useful in tests, and the same rule as [`best`].
#[must_use]
pub fn best_of(path: &Path, mounts: &[Mount]) -> Option<Mount> {
    let mut found: Option<&Mount> = None;
    for mount in mounts {
        if path != mount.point && !path.starts_with(&mount.point) {
            continue;
        }
        let replace = match found {
            None => true,
            Some(current) => mount.point.components().count() > current.point.components().count(),
        };
        if replace {
            found = Some(mount);
        }
    }
    found.cloned()
}

#[cfg(target_os = "linux")]
fn read_proc_mounts() -> Vec<Mount> {
    let Ok(text) = fs::read_to_string("/proc/mounts") else {
        return Vec::new();
    };
    let mut mounts = Vec::new();
    for line in text.lines() {
        let mut fields = line.split_whitespace();
        let Some(device) = fields.next() else {
            continue;
        };
        let Some(point) = fields.next() else {
            continue;
        };
        let Some(kind) = fields.next() else {
            continue;
        };
        mounts.push(Mount {
            device: unescape(device),
            point: PathBuf::from(unescape(point)),
            kind: unescape(kind),
        });
    }
    mounts
}

/// `/proc/mounts` writes a space as `\040`. Turn those escapes back into characters.
#[cfg(target_os = "linux")]
fn unescape(text: &str) -> String {
    let mut out = String::new();
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\\' && index + 4 <= bytes.len() {
            let octal = &text[index + 1..index + 4];
            if let Ok(byte) = u8::from_str_radix(octal, 8) {
                out.push(char::from(byte));
                index += 4;
                continue;
            }
        }
        out.push(char::from(bytes[index]));
        index += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{best_of, note_for, Mount};

    #[test]
    fn the_longest_mount_point_wins() {
        let mounts = vec![
            Mount {
                device: "/dev/root".to_owned(),
                point: Path::new("/").to_path_buf(),
                kind: "ext4".to_owned(),
            },
            Mount {
                device: "tmpfs".to_owned(),
                point: Path::new("/tmp").to_path_buf(),
                kind: "tmpfs".to_owned(),
            },
        ];
        let found = best_of(Path::new("/tmp/notes.txt"), &mounts).expect("mount");
        assert_eq!(found.kind, "tmpfs");
        let root = best_of(Path::new("/etc/passwd"), &mounts).expect("root");
        assert_eq!(root.kind, "ext4");
    }

    #[test]
    fn a_note_does_not_claim_we_deleted_a_snapshot() {
        let note = note_for(Path::new("/"));
        assert!(
            note.contains("not deleted") || note.contains("Snapshots"),
            "{note}"
        );
        assert!(note.contains("does not change the disk"), "{note}");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn space_escape_is_a_space() {
        assert_eq!(super::unescape(r"/mnt/my\040disk"), "/mnt/my disk");
    }
}
