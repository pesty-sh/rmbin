//! # Section 10 — Delete names, folders last
//!
//! What you'll learn: `remove_file`, `remove_dir`, and doing the last step last.
//! Builds on: Section 09. The bytes are already overwritten before we get here.
//! Used by: the run that prints the report.
//! Functions: F13.
//!
//! A symlink is removed with the call that unlinks the link itself. On Unix that
//! is `remove_file`, even when the link points at a folder. The target stays.
//!
//! After a name is removed we sync the parent folder on Unix so the directory
//! entry is flushed too. If that sync fails, the report says the name is gone
//! and the folder entry may not be durable. Nothing is restored.
//!
//! Windows 0.0.1 syncs the file itself in Section 09 and does not sync the
//! parent folder. The limits banner is the place that difference is stated
//! for hard links and other volumes; parent-folder sync is noted in SECURITY.md.

use std::fs;
#[cfg(unix)]
use std::fs::File;
use std::path::Path;

use crate::error::{Error, Result};
use crate::overwrite;
use crate::pattern::Pattern;
use crate::plan::{Action, ActionKind};

/// What happened to one plan entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// The path from the plan.
    pub path: std::path::PathBuf,
    /// Deleted, kept, or not started.
    pub state: State,
}

/// The three report states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// The step for this path finished.
    Deleted {
        /// `file`, `symlink`, or `folder`.
        kind: &'static str,
    },
    /// The path was not securely deleted. It should still exist, unless the
    /// reason says the name was removed and a later sync failed.
    NotSecurelyDeleted {
        /// Plain-language reason.
        reason: String,
    },
    /// An earlier failure stopped the run before this path.
    NotAttempted,
}

impl Outcome {
    /// `true` when this path was not fully securely deleted.
    #[must_use]
    pub fn failed(&self) -> bool {
        !matches!(self.state, State::Deleted { .. })
    }
}

enum Control {
    Done(&'static str),
    /// Leave the path and keep going. Used for planned skips.
    Keep(String),
    /// Leave the path and do not start the rest.
    Stop(String),
}

/// Apply `actions` in order. Stops after the first overwrite or delete error.
#[must_use]
pub fn apply(pattern: &Pattern, actions: &[Action]) -> Vec<Outcome> {
    let mut outcomes = Vec::new();
    for (index, action) in actions.iter().enumerate() {
        match apply_one(pattern, action) {
            Control::Done(kind) => outcomes.push(Outcome {
                path: action.path.clone(),
                state: State::Deleted { kind },
            }),
            Control::Keep(reason) => outcomes.push(Outcome {
                path: action.path.clone(),
                state: State::NotSecurelyDeleted { reason },
            }),
            Control::Stop(reason) => {
                outcomes.push(Outcome {
                    path: action.path.clone(),
                    state: State::NotSecurelyDeleted { reason },
                });
                for rest in actions.iter().skip(index + 1) {
                    outcomes.push(Outcome {
                        path: rest.path.clone(),
                        state: State::NotAttempted,
                    });
                }
                break;
            }
        }
    }
    outcomes
}

fn apply_one(pattern: &Pattern, action: &Action) -> Control {
    match &action.kind {
        ActionKind::Skip { reason } => Control::Keep(reason.clone()),
        ActionKind::OverwriteFile { .. } => {
            match overwrite::overwrite_then_truncate(&action.path, pattern)
                .and_then(|()| unlink_file(&action.path))
            {
                Ok(()) => Control::Done("file"),
                Err(error) => Control::Stop(explain(error)),
            }
        }
        ActionKind::DeleteSymlink => match unlink_symlink(&action.path) {
            Ok(()) => Control::Done("symlink"),
            Err(error) => Control::Stop(explain(error)),
        },
        ActionKind::RemoveDirectory => match unlink_dir(&action.path) {
            Ok(()) => Control::Done("folder"),
            Err(error) => Control::Stop(explain(error)),
        },
    }
}

fn explain(error: Error) -> String {
    match error {
        Error::NotSecurelyDeleted { reason, .. } => reason,
        other => other.to_string(),
    }
}

fn unlink_file(path: &Path) -> Result<()> {
    fs::remove_file(path).map_err(|source| {
        Error::NotSecurelyDeleted {
            path: path.to_path_buf(),
            reason: format!(
                "could not delete the name after overwrite: {source}. The file was kept. It may already contain only the pattern, or be empty if truncate finished. Nothing was restored."
            ),
        }
    })?;
    sync_parent(path)
}

fn unlink_symlink(path: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(path).map_err(|source| Error::io(path, source))?;
    if !meta.file_type().is_symlink() && !crate::classify::is_reparse(&meta) {
        return Err(Error::NotSecurelyDeleted {
            path: path.to_path_buf(),
            reason: "expected a symlink. The path was left in place.".to_owned(),
        });
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x0000_0010;
        if meta.file_attributes() & FILE_ATTRIBUTE_DIRECTORY != 0 {
            fs::remove_dir(path).map_err(|source| link_error(path, source))?;
        } else {
            fs::remove_file(path).map_err(|source| link_error(path, source))?;
        }
    }
    #[cfg(not(windows))]
    {
        fs::remove_file(path).map_err(|source| link_error(path, source))?;
    }
    sync_parent(path)
}

fn link_error(path: &Path, source: std::io::Error) -> Error {
    Error::NotSecurelyDeleted {
        path: path.to_path_buf(),
        reason: format!("could not delete the symlink: {source}. It was left in place."),
    }
}

fn unlink_dir(path: &Path) -> Result<()> {
    fs::remove_dir(path).map_err(|source| Error::NotSecurelyDeleted {
        path: path.to_path_buf(),
        reason: format!(
            "could not remove the folder: {source}. It was left in place. Entries already handled were not restored."
        ),
    })?;
    sync_parent(path)
}

fn sync_parent(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let Some(parent) = path.parent() else {
            return Ok(());
        };
        if parent.as_os_str().is_empty() {
            return Ok(());
        }
        let directory = match File::open(parent) {
            Ok(directory) => directory,
            Err(source) => {
                return Err(Error::NotSecurelyDeleted {
                    path: path.to_path_buf(),
                    reason: format!(
                        "the name was removed, but the parent folder could not be opened to sync: {source}. Nothing was restored."
                    ),
                });
            }
        };
        if let Err(source) = directory.sync_all() {
            return Err(Error::NotSecurelyDeleted {
                path: path.to_path_buf(),
                reason: format!(
                    "the name was removed, but syncing the parent folder failed: {source}. The file data was synced before the name was removed. Nothing was restored."
                ),
            });
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}
