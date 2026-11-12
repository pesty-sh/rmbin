//! # Section 21 — The sandbox folder
//!
//! What you'll learn: keeping every scratch path inside one folder.
//! Builds on: Section 01.
//! Used by: the tests, and Section 20 before it plans a path.
//! Functions: the jail around the later steps.
//!
//! The sandbox is the `sandbox` folder next to `Cargo.toml`. Tests create
//! their files there. The command refuses to plan a path that is not inside
//! that folder. It also refuses to run as root, because root can change the
//! rest of the drive.
//!
//! Cleanup deletes one scratch folder, and only when that folder's parent is
//! the sandbox. It does not delete the sandbox itself, the project, or `/`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::{Error, Result};

/// The project's sandbox directory.
#[derive(Debug, Clone)]
pub struct Sandbox {
    root: PathBuf,
}

/// A folder inside the sandbox. Dropping it deletes that folder only.
#[derive(Debug)]
pub struct Scratch {
    sandbox: Sandbox,
    path: PathBuf,
}

impl Scratch {
    /// Create a new empty folder inside the project sandbox.
    ///
    /// # Errors
    ///
    /// The process is root, or the sandbox folder cannot be created.
    pub fn new(label: &str) -> Result<Self> {
        let sandbox = open_project()?;
        let path = sandbox.scratch(label)?;
        Ok(Self { sandbox, path })
    }

    /// The scratch folder. It is inside the project sandbox.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = self.sandbox.remove_scratch(&self.path);
    }
}

/// Find the sandbox by walking up from the current directory.
///
/// The walk looks for `Cargo.toml`. It does not search the whole drive.
/// The sandbox is created only as a child of that project folder.
///
/// # Errors
///
/// The process is root, the current directory cannot be read, or this
/// folder is not inside a project.
pub fn locate() -> Result<Sandbox> {
    refuse_if_root()?;
    let start = std::env::current_dir().map_err(|source| Error::Io { path: None, source })?;
    let Some(project) = find_project(&start) else {
        return Err(Error::NoSandbox);
    };
    open_at(&project)
}

/// True when this process is root.
///
/// On Linux the effective uid comes from `/proc/self/status`. If that file
/// cannot be read, this returns true so the command stops. Other systems
/// do not have that file; the path jail is still applied.
#[must_use]
pub fn running_as_root() -> bool {
    #[cfg(target_os = "linux")]
    {
        let Ok(text) = fs::read_to_string("/proc/self/status") else {
            return true;
        };
        effective_uid_is_root(&text)
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Read the effective uid from a `/proc/self/status` body.
///
/// The line looks like `Uid: 1000 1000 1000 1000`. The second number is the
/// effective uid. A missing line is treated as root.
#[must_use]
pub fn effective_uid_is_root(status: &str) -> bool {
    for line in status.lines() {
        let Some(rest) = line.strip_prefix("Uid:") else {
            continue;
        };
        let mut numbers = rest.split_whitespace();
        let _real = numbers.next();
        return numbers.next().is_none_or(|uid| uid == "0");
    }
    true
}

fn refuse_if_root() -> Result<()> {
    if running_as_root() {
        Err(Error::Root)
    } else {
        Ok(())
    }
}

fn open_project() -> Result<Sandbox> {
    refuse_if_root()?;
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    open_at(&project)
}

fn open_at(project: &Path) -> Result<Sandbox> {
    if !project.join("Cargo.toml").is_file() {
        return Err(Error::NoSandbox);
    }
    let root = project.join("sandbox");
    if !root.is_dir() {
        fs::create_dir(&root).map_err(|source| Error::io(&root, source))?;
    }
    let root = fs::canonicalize(&root).map_err(|source| Error::io(&root, source))?;
    if root.file_name().and_then(|name| name.to_str()) != Some("sandbox") {
        return Err(Error::NoSandbox);
    }
    Ok(Sandbox { root })
}

fn find_project(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("Cargo.toml").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

impl Sandbox {
    /// This directory, after it has been made absolute.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Create a child folder. The name is a short label plus a counter.
    ///
    /// # Errors
    ///
    /// The folder cannot be created inside the sandbox.
    pub fn scratch(&self, label: &str) -> Result<PathBuf> {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = format!("{}-{}-{n}", safe_label(label), std::process::id());
        let path = self.root.join(name);
        fs::create_dir(&path).map_err(|source| Error::io(&path, source))?;
        self.require_inside(&path)?;
        fs::canonicalize(&path).map_err(|source| Error::io(&path, source))
    }

    /// Accept `path` only when it is strictly inside this sandbox.
    ///
    /// A final symlink is not followed, so a link inside the sandbox is
    /// allowed and a link whose own folder is outside is refused. The
    /// sandbox folder itself is refused. That stops a later delete from
    /// removing the container.
    ///
    /// # Errors
    ///
    /// The path is missing, or it is not inside the sandbox.
    pub fn require_inside(&self, path: &Path) -> Result<()> {
        let meta = fs::symlink_metadata(path).map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                Error::NotFound {
                    path: path.to_path_buf(),
                }
            } else {
                Error::io(path, source)
            }
        })?;
        let looked_at = if meta.file_type().is_symlink() {
            path.parent().unwrap_or(path)
        } else {
            path
        };
        let canon = fs::canonicalize(looked_at).map_err(|source| Error::io(looked_at, source))?;
        let inside = canon.starts_with(&self.root) && canon != self.root;
        if !inside {
            return Err(Error::OutsideSandbox {
                path: path.to_path_buf(),
            });
        }
        Ok(())
    }

    /// Delete one scratch folder. The parent must be the sandbox.
    ///
    /// # Errors
    ///
    /// `dir` is the sandbox, the project, or any path that is not a direct
    /// child of the sandbox.
    pub fn remove_scratch(&self, dir: &Path) -> Result<()> {
        let canon = fs::canonicalize(dir).map_err(|source| Error::io(dir, source))?;
        if canon == self.root || !canon.starts_with(&self.root) {
            return Err(Error::OutsideSandbox { path: canon });
        }
        if canon.parent() != Some(self.root.as_path()) {
            return Err(Error::OutsideSandbox { path: canon });
        }
        relax(&canon);
        fs::remove_dir_all(&canon).map_err(|source| Error::io(&canon, source))
    }
}

fn safe_label(label: &str) -> String {
    let mut out = String::new();
    for ch in label.chars().take(24) {
        if ch.is_ascii_alphanumeric() || ch == '-' {
            out.push(ch);
        }
    }
    if out.is_empty() {
        out.push_str("scratch");
    }
    out
}

fn relax(path: &Path) {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return;
    };
    if meta.file_type().is_symlink() {
        return;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if meta.is_dir() { 0o755 } else { 0o644 };
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
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
    if meta.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                relax(&entry.path());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{effective_uid_is_root, running_as_root, Scratch};

    #[test]
    fn uid_zero_is_root_and_a_normal_uid_is_not() {
        assert!(effective_uid_is_root("Uid:\t0\t0\t0\t0\n"));
        assert!(effective_uid_is_root("Uid:\t1000\t0\t1000\t1000\n"));
        assert!(!effective_uid_is_root("Uid:\t1000\t1000\t1000\t1000\n"));
        assert!(effective_uid_is_root("Name:\tno uid here\n"));
    }

    #[test]
    fn this_process_is_not_root_and_scratch_stays_in_the_project() {
        assert!(!running_as_root(), "refusing to test as root");
        let scratch = Scratch::new("jail").expect("sandbox");
        let path = scratch.path().to_path_buf();
        assert!(path.starts_with(scratch.sandbox.root()));
        assert!(path.parent() == Some(scratch.sandbox.root()));
        let project = path
            .parent()
            .and_then(|sandbox| sandbox.parent())
            .expect("project");
        let marker = project.join("Cargo.toml");
        assert!(marker.is_file());
        let refused = scratch.sandbox.require_inside(project);
        assert!(refused.is_err());
        let sandbox_refused = scratch.sandbox.require_inside(scratch.sandbox.root());
        assert!(sandbox_refused.is_err());
        assert!(scratch.sandbox.remove_scratch(project).is_err());
        assert!(marker.is_file(), "the project is still here");
        assert!(project.join("src").join("lib.rs").is_file());
    }
}
