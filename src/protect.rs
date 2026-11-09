//! # Section 06 — Protected paths
//!
//! What you'll learn: `Path` parts, `#[cfg]` for one operating system, and a list
//! you can read. `cfg` means "include this code only when building for that system".
//! Builds on: Section 05, which tells us what a path points at.
//! Used by: Section 07, before any delete.
//! Functions: F07.
//!
//! rmbin refuses to delete a filesystem root, your home directory itself, a
//! mount point you named, or a system directory. A file *inside* your home
//! directory is allowed. That is the point of the tool.
//!
//! The lists below are the whole deny list. Nothing is added at runtime except
//! mount points and volume roots reported by the operating system, and the
//! current home directory.

use std::fs;
use std::path::{Component, Path, PathBuf};

/// Prefixes of system directories. A path matches when its name parts *start*
/// with one of these rows. `/usr/bin/env` matches `usr`. `/usr-local` does not.
#[cfg(unix)]
const SYSTEM_PREFIXES: &[&[&str]] = &[
    &["bin"],
    &["sbin"],
    &["usr"],
    &["etc"],
    &["dev"],
    &["proc"],
    &["sys"],
    &["boot"],
    &["run"],
    &["root"],
    &["Library"],
    &["Applications"],
    &["System", "Library"],
    &["System", "Applications"],
    &["private", "etc"],
    &["private", "var", "db"],
    &["private", "var", "root"],
    &["var", "db"],
    &["var", "root"],
    &["var", "log"],
];

/// Windows system directories, matched from the root of a drive.
/// `C:\Windows\System32` matches `Windows`. `C:\Users\me\Windows` does not,
/// because `Users` is the first name.
#[cfg(windows)]
const SYSTEM_PREFIXES: &[&[&str]] = &[
    &["Windows"],
    &["Program Files"],
    &["Program Files (x86)"],
    &["ProgramData"],
    &["System Volume Information"],
];

/// Directories we refuse to delete as a whole, while still allowing a child.
/// `/tmp/file` is allowed. `/tmp` itself is not.
#[cfg(unix)]
const EXACT_DIRECTORIES: &[&[&str]] = &[
    &["tmp"],
    &["private"],
    &["private", "tmp"],
    &["private", "var"],
    &["private", "var", "folders"],
    &["var"],
    &["var", "tmp"],
    &["var", "folders"],
    &["home"],
    &["mnt"],
    &["media"],
    &["Volumes"],
    &["System"],
    &["System", "Volumes"],
];

/// `C:\Users` itself is refused. A folder inside a user profile is not, unless
/// it is that user's home directory exactly.
#[cfg(windows)]
const EXACT_DIRECTORIES: &[&[&str]] = &[&["Users"]];

/// Loaded once per run: home, mount points, and volume roots.
#[derive(Debug)]
pub struct Guard {
    home: Option<PathBuf>,
    mounts: Vec<PathBuf>,
    volumes: Vec<PathBuf>,
}

impl Guard {
    /// Read the deny-list extras from the environment and the operating system.
    #[must_use]
    pub fn load() -> Self {
        Self {
            home: home_dir(),
            mounts: mount_points(),
            volumes: volume_roots(),
        }
    }

    /// Why `path` must not be a root target. `None` means it is allowed.
    ///
    /// `path` should already be absolute. Children of your home directory are
    /// allowed. The home directory itself is not.
    #[must_use]
    pub fn denial_root(&self, path: &Path) -> Option<&'static str> {
        if is_filesystem_root(path) {
            return Some("filesystem root");
        }
        let names = name_parts(path);
        if prefixes_match(&names) {
            return Some("system directory");
        }
        if exact_match(&names) {
            return Some("this directory itself");
        }
        if let Some(home) = &self.home {
            if same_place(path, home) {
                return Some("your home directory");
            }
        }
        if self.volumes.iter().any(|volume| same_place(path, volume)) {
            return Some("a volume root");
        }
        if self.mounts.iter().any(|mount| same_place(path, mount)) {
            return Some("a mount point");
        }
        None
    }
}

fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    let key = "USERPROFILE";
    #[cfg(not(windows))]
    let key = "HOME";
    std::env::var_os(key)
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
}

fn is_filesystem_root(path: &Path) -> bool {
    let mut saw_root = false;
    for component in path.components() {
        match component {
            Component::Prefix(_) => {}
            Component::RootDir => saw_root = true,
            _ => return false,
        }
    }
    saw_root
}

fn name_parts(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(fold(&name.to_string_lossy())),
            _ => None,
        })
        .collect()
}

fn fold(text: &str) -> String {
    // macOS and Windows usually treat `Windows` and `windows` as the same folder.
    #[cfg(any(windows, target_os = "macos"))]
    {
        text.to_ascii_lowercase()
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        text.to_owned()
    }
}

fn prefixes_match(names: &[String]) -> bool {
    SYSTEM_PREFIXES
        .iter()
        .any(|prefix| starts_with_names(names, prefix))
}

fn exact_match(names: &[String]) -> bool {
    EXACT_DIRECTORIES
        .iter()
        .any(|exact| names.len() == exact.len() && starts_with_names(names, exact))
}

fn starts_with_names(names: &[String], expected: &[&str]) -> bool {
    if names.len() < expected.len() {
        return false;
    }
    names
        .iter()
        .zip(expected.iter())
        .all(|(got, want)| got == &fold(want))
}

fn same_place(left: &Path, right: &Path) -> bool {
    #[cfg(unix)]
    {
        if let (Ok(left_meta), Ok(right_meta)) =
            (fs::symlink_metadata(left), fs::symlink_metadata(right))
        {
            use std::os::unix::fs::MetadataExt;
            return left_meta.dev() == right_meta.dev() && left_meta.ino() == right_meta.ino();
        }
    }
    fold(&left.to_string_lossy()) == fold(&right.to_string_lossy())
}

fn mount_points() -> Vec<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        read_proc_mounts()
    }
    #[cfg(not(target_os = "linux"))]
    {
        Vec::new()
    }
}

#[cfg(target_os = "linux")]
fn read_proc_mounts() -> Vec<PathBuf> {
    let Ok(text) = fs::read_to_string("/proc/mounts") else {
        return Vec::new();
    };
    let mut mounts = Vec::new();
    for line in text.lines() {
        let mut fields = line.split_whitespace();
        let _device = fields.next();
        if let Some(point) = fields.next() {
            mounts.push(PathBuf::from(unescape_mount(point)));
        }
    }
    mounts
}

/// `/proc/mounts` writes a space as `\040`. Turn those escapes back into characters.
#[cfg(target_os = "linux")]
fn unescape_mount(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        let digits: String = chars.by_ref().take(3).collect();
        if let Ok(byte) = u8::from_str_radix(&digits, 8) {
            out.push(char::from(byte));
        } else {
            out.push('\\');
            out.push_str(&digits);
        }
    }
    out
}

fn volume_roots() -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let mut roots = Vec::new();
        if let Ok(entries) = fs::read_dir("/Volumes") {
            for entry in entries.flatten() {
                roots.push(entry.path());
            }
        }
        roots
    }
    #[cfg(not(target_os = "macos"))]
    {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Guard;

    #[cfg(unix)]
    #[test]
    fn system_prefixes_and_exact_directories() {
        let guard = Guard::load();
        assert_eq!(guard.denial_root(Path::new("/")), Some("filesystem root"));
        assert_eq!(
            guard.denial_root(Path::new("/etc/passwd")),
            Some("system directory")
        );
        assert_eq!(
            guard.denial_root(Path::new("/usr/bin/env")),
            Some("system directory")
        );
        assert_eq!(guard.denial_root(Path::new("/etc-backup")), None);
        assert_eq!(
            guard.denial_root(Path::new("/tmp")),
            Some("this directory itself")
        );
        assert_eq!(guard.denial_root(Path::new("/tmp/rmbin-test")), None);
        assert_eq!(
            guard.denial_root(Path::new("/System")),
            Some("this directory itself")
        );
        assert_eq!(
            guard.denial_root(Path::new("/System/Library/CoreServices")),
            Some("system directory")
        );
        // The writable data volume lives under /System/Volumes/Data. Do not
        // treat every file there as a system file.
        assert_eq!(
            guard.denial_root(Path::new("/System/Volumes/Data/Users/me/notes")),
            None
        );
    }

    #[test]
    fn home_itself_is_refused_and_a_child_is_not() {
        let Some(home) = super::home_dir() else {
            return;
        };
        let guard = Guard::load();
        assert!(guard.denial_root(&home).is_some());
        let child = home.join("rmbin-not-the-home-dir");
        // A child of a system home such as /root is still a system path.
        if guard.denial_root(&child) == Some("system directory") {
            return;
        }
        assert_eq!(guard.denial_root(&child), None);
    }

    #[cfg(windows)]
    #[test]
    fn windows_system_paths() {
        let guard = Guard::load();
        assert_eq!(
            guard.denial_root(Path::new(r"C:\")),
            Some("filesystem root")
        );
        assert_eq!(
            guard.denial_root(Path::new(r"C:\Windows\System32\cmd.exe")),
            Some("system directory")
        );
        assert_eq!(
            guard.denial_root(Path::new(r"C:\Users")),
            Some("this directory itself")
        );
        assert_eq!(guard.denial_root(Path::new(r"D:\work\notes.txt")), None);
    }
}
