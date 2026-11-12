//! # Section 17 — What the kernel says about the disk
//!
//! What you'll learn: reading one flag from `/sys`, and not trusting it too hard.
//! Builds on: Section 16, which names the device.
//! Used by: Section 20.
//! Functions: F30, report only.
//!
//! Linux publishes `rotational` as `0` or `1` for a block device. `0` means
//! the kernel is describing the device as non-rotating. A virtual disk can
//! say `1` even when the machine behind it is an SSD. We print the flag.
//! We do not skip the later overwrite because of it, and this version does
//! not write.

#[cfg(target_os = "linux")]
use std::fs;
use std::path::Path;

use crate::snapshot;

/// A sentence about `device`, or `None` when the kernel does not publish a flag.
///
/// `device` is a mount device such as `/dev/vdc`. Names that are not under
/// `/dev` (for example `tmpfs`) have no rotational flag.
#[must_use]
pub fn note(device: &str) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let name = block_name(device)?;
        let path = format!("/sys/class/block/{name}/queue/rotational");
        let text = fs::read_to_string(&path).ok()?;
        let flag = text.trim();
        let meaning = match flag {
            "0" => "the kernel describes it as non-rotating",
            "1" => "the kernel describes it as rotating",
            _ => return None,
        };
        Some(format!(
            "Kernel rotational flag for {name} is {flag} ({meaning}). A virtual disk can report this wrong. This version does not write."
        ))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = device;
        None
    }
}

/// The disk sentence for a path, when Section 16 can see a `/dev` device.
#[must_use]
pub fn note_for_path(path: &Path) -> Option<String> {
    let mount = snapshot::best(path)?;
    note(&mount.device)
}

#[cfg(target_os = "linux")]
fn block_name(device: &str) -> Option<&str> {
    let rest = device.strip_prefix("/dev/")?;
    let name = rest.rsplit('/').next().unwrap_or(rest);
    if name.is_empty() || name.contains('.') {
        return None;
    }
    Some(name)
}

#[cfg(test)]
mod tests {
    use super::note;

    #[test]
    fn tmpfs_has_no_rotational_flag() {
        assert_eq!(note("tmpfs"), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_dev_name_is_the_last_piece() {
        assert_eq!(super::block_name("/dev/vdc"), Some("vdc"));
        assert_eq!(super::block_name("/dev/mapper/data"), Some("data"));
        assert_eq!(super::block_name("tmpfs"), None);
    }
}
