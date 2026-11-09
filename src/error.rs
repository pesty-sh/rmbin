//! # Section 01 — Results and our error type
//!
//! What you'll learn: an `enum`, `Result`, `Display`, and the `?` operator.
//! Builds on: L0.
//! Used by: every later section.
//! Functions: F04.
//!
//! A `Result` is either `Ok(value)` or `Err(error)`. The `?` operator means
//! "if this is an error, return it now". Library code never panics because a
//! path was missing or a write was refused. The caller reports it.

use std::fmt;
use std::io;
use std::path::PathBuf;

/// The result of an rmbin function.
pub type Result<T> = std::result::Result<T, Error>;

/// Why a run did not finish as a full secure delete.
#[derive(Debug)]
pub enum Error {
    /// `--pattern` was present but empty. An empty pattern cannot fill a file.
    EmptyPattern,
    /// The command named no paths.
    NoPaths,
    /// `--list` contained an empty piece, for example `a,,b`.
    EmptyListItem,
    /// A path we will not touch, such as `/` or a system directory.
    Protected {
        /// The path that was refused.
        path: PathBuf,
        /// Which rule matched. This is a fixed phrase, not file contents.
        rule: &'static str,
    },
    /// The path does not exist.
    NotFound {
        /// The missing path.
        path: PathBuf,
    },
    /// A folder in the middle of the path is a symlink. We do not follow it.
    SymlinkInPath {
        /// The path the user typed, made absolute.
        typed: PathBuf,
        /// The symlink we refused to walk through.
        link: PathBuf,
        /// Where that symlink points, when the operating system told us.
        points_at: Option<PathBuf>,
    },
    /// A filesystem call failed. `path` is set when the call was about one file.
    Io {
        /// The file or folder, when there is one.
        path: Option<PathBuf>,
        /// The operating system's error.
        source: io::Error,
    },
    /// This path was not securely deleted. `reason` says which step failed.
    NotSecurelyDeleted {
        /// The path we kept, or tried to keep.
        path: PathBuf,
        /// What went wrong, in plain language.
        reason: String,
    },
    /// The user did not type `yes`.
    Cancelled,
    /// One or more paths were not securely deleted. `count` is how many.
    Incomplete {
        /// How many paths were not securely deleted.
        count: usize,
    },
}

impl Error {
    /// Process exit code. `0` is not used here.
    ///
    /// - `1` means a path was refused or not securely deleted.
    /// - `2` means the confirmation was not `yes`.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Cancelled => 2,
            _ => 1,
        }
    }

    pub(crate) fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: Some(path.into()),
            source,
        }
    }

    pub(crate) fn report(source: io::Error) -> Self {
        Self::Io { path: None, source }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPattern => write!(
                f,
                "the pattern is empty. Pass text, for example --pattern lol"
            ),
            Self::NoPaths => {
                write!(f, "no paths given. Pass paths, or --list 'a.txt,b.txt'")
            }
            Self::EmptyListItem => write!(
                f,
                "the --list value has an empty item. Check for a doubled comma or semicolon"
            ),
            Self::Protected { path, rule } => {
                write!(f, "refusing protected path {} ({rule})", path.display())
            }
            Self::NotFound { path } => write!(f, "path not found: {}", path.display()),
            Self::SymlinkInPath {
                typed,
                link,
                points_at,
            } => {
                write!(
                    f,
                    "refusing to follow symlink {} while looking up {}",
                    link.display(),
                    typed.display()
                )?;
                if let Some(target) = points_at {
                    write!(
                        f,
                        ". That symlink points at {}. Name the link itself if you only want the link removed, or pass a path that does not go through a symlink",
                        target.display()
                    )?;
                }
                Ok(())
            }
            Self::Io { path, source } => match path {
                Some(path) => write!(f, "{}: {source}", path.display()),
                None => write!(f, "{source}"),
            },
            Self::NotSecurelyDeleted { path, reason } => {
                write!(f, "NOT securely deleted: {} ({reason})", path.display())
            }
            Self::Cancelled => {
                write!(f, "confirmation was not yes. Nothing was changed")
            }
            Self::Incomplete { count } => write!(
                f,
                "{count} path(s) were NOT securely deleted. The report above lists them"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
