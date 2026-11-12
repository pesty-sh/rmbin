//! # Section 07 — The read-only plan
//!
//! What you'll learn: walking a tree, and putting work in order.
//! Builds on: Section 05 (what a path is) and Section 06 (protected roots).
//! Used by: Section 08 (confirmation) and Section 10 (delete).
//! Functions: F08, F09.
//!
//! The words are "would". This version prints the plan and does not do it.
//!
//! The plan only reads. It does not open a file for writing. Folders are listed
//! bottom-up: a file is handled before the folder that contains it, so the
//! folder is empty when we remove it.
//!
//! Symlinks are not entered. On macOS and Linux a directory whose device id
//! differs from the directory you named is not entered. Windows 0.0.1 cannot
//! read that id. The limits banner says so.

use std::fs;
use std::path::Path;

use crate::classify::{self, Kind};
use crate::error::{Error, Result};

/// How deep a folder may be nested. Deeper than this, the run stops before
/// changing anything. 256 is far past a normal project tree.
const MAX_DEPTH: u32 = 256;

/// One thing the plan might do, still without having done it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    /// Absolute path.
    pub path: std::path::PathBuf,
    /// What would happen to it.
    pub kind: ActionKind,
}

/// The kinds of plan entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionKind {
    /// Overwrite this many bytes, sync, truncate, sync, then delete the name.
    OverwriteFile {
        /// Original length. We never write more than this.
        bytes: u64,
    },
    /// Remove the symlink itself. The target is not opened.
    DeleteSymlink,
    /// Remove a folder after its entries.
    RemoveDirectory,
    /// Leave this path in place and say why.
    Skip {
        /// Fixed reason, not file contents.
        reason: String,
    },
}

impl Action {
    /// One line for the plan, before anything is changed.
    #[must_use]
    pub fn preview(&self) -> String {
        match &self.kind {
            ActionKind::OverwriteFile { bytes } => format!(
                "would overwrite {bytes} bytes, sync, read back, truncate, rename, then delete  {}",
                self.path.display()
            ),
            ActionKind::DeleteSymlink => format!(
                "would delete symlink only; the target is not changed  {}",
                self.path.display()
            ),
            ActionKind::RemoveDirectory => {
                format!(
                    "would remove folder after its entries  {}",
                    self.path.display()
                )
            }
            ActionKind::Skip { reason } => format!(
                "skip; would be NOT securely deleted  {}  ({reason})",
                self.path.display()
            ),
        }
    }

    /// A skip is reported and does not stop the rest of the run by itself.
    #[must_use]
    pub fn is_skip(&self) -> bool {
        matches!(self.kind, ActionKind::Skip { .. })
    }
}

/// Build the plan for `roots`. Each root must already be absolute and allowed.
///
/// # Errors
///
/// A folder that cannot be listed, or a tree deeper than [`MAX_DEPTH`], stops
/// the plan. Nothing has been changed yet when this returns an error.
pub fn build(roots: &[std::path::PathBuf]) -> Result<Vec<Action>> {
    let mut actions = Vec::new();
    for root in roots {
        let root_device = match classify::classify(root)? {
            Kind::Directory { device } => device,
            _ => None,
        };
        walk(root, root_device, 0, &mut actions)?;
    }
    Ok(actions)
}

fn walk(
    path: &Path,
    root_device: Option<u64>,
    depth: u32,
    actions: &mut Vec<Action>,
) -> Result<()> {
    if depth > MAX_DEPTH {
        return Err(Error::NotSecurelyDeleted {
            path: path.to_path_buf(),
            reason: "folder is nested more than 256 levels. Nothing was changed.".to_owned(),
        });
    }
    match classify::classify(path)? {
        Kind::Symlink => actions.push(Action {
            path: path.to_path_buf(),
            kind: ActionKind::DeleteSymlink,
        }),
        Kind::Other { reason } => actions.push(Action {
            path: path.to_path_buf(),
            kind: ActionKind::Skip {
                reason: reason.to_owned(),
            },
        }),
        Kind::File { len, hard_links } => {
            if hard_links > 1 {
                actions.push(Action {
                    path: path.to_path_buf(),
                    kind: ActionKind::Skip {
                        reason: "more than one hard link".to_owned(),
                    },
                });
            } else {
                actions.push(Action {
                    path: path.to_path_buf(),
                    kind: ActionKind::OverwriteFile { bytes: len },
                });
            }
        }
        Kind::Directory { device } => {
            if let (Some(root_device), Some(device)) = (root_device, device) {
                if classify::different_device(root_device, device) {
                    actions.push(Action {
                        path: path.to_path_buf(),
                        kind: ActionKind::Skip {
                            reason: "different device; not entered".to_owned(),
                        },
                    });
                    return Ok(());
                }
            }
            let mut children = Vec::new();
            let entries = fs::read_dir(path).map_err(|source| Error::io(path, source))?;
            for entry in entries {
                let entry = entry.map_err(|source| Error::io(path, source))?;
                children.push(entry.path());
            }
            children.sort();
            for child in children {
                walk(&child, root_device, depth + 1, actions)?;
            }
            actions.push(Action {
                path: path.to_path_buf(),
                kind: ActionKind::RemoveDirectory,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{build, ActionKind};
    use crate::sandbox::Scratch;

    #[test]
    fn folders_are_listed_bottom_up_and_symlinks_are_not_entered() {
        let scratch = Scratch::new("plan").expect("sandbox");
        let root = scratch.path().to_path_buf();
        let sub = root.join("sub");
        fs::create_dir(&sub).expect("dir");
        fs::write(sub.join("b.txt"), b"bbbb").expect("write");
        fs::write(root.join("a.txt"), b"aa").expect("write");
        let outside = root.join("outside.txt");
        fs::write(&outside, b"secret").expect("write");
        let link = sub.join("link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &link).expect("symlink");
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&outside, &link).expect("symlink");

        let actions = build(&[root.clone()]).expect("plan");
        let previews: Vec<_> = actions.iter().map(super::Action::preview).collect();
        let joined = previews.join("\n");
        let file_pos = joined.find("a.txt").expect("a");
        let nested_pos = joined.find("b.txt").expect("b");
        let sub_pos = joined.find("remove folder").expect("folder lines");
        assert!(nested_pos < sub_pos, "{joined}");
        assert!(file_pos < joined.rfind("remove folder").expect("root folder"));
        assert!(joined.contains("delete symlink only"));
        assert_eq!(fs::read(&outside).expect("outside"), b"secret");
        assert!(actions.iter().any(|action| {
            matches!(action.kind, ActionKind::DeleteSymlink) && action.path == link
        }));
    }
}
