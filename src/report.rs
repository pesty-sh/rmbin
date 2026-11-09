//! # Section 11 — The report and the limits banner
//!
//! What you'll learn: writing text with `writeln!`, and telling the truth when
//! a step fails.
//! Builds on: Sections 01 and 07.
//! Used by: every real run, including a dry run.
//! Functions: F06, F14.
//!
//! The banner is printed first. It says overwrite is best effort. Assurance
//! level B means "we overwrote the bytes the filesystem showed us". It does
//! not mean a certified wipe.
//!
//! `NOT securely deleted` is the phrase for a path we did not finish. A dry run
//! uses `would` and does not use `securely deleted`.

use std::io::Write;

use crate::error::{Error, Result};
use crate::pattern::Pattern;
use crate::plan::Action;
use crate::remove::{Outcome, State};

/// Print the limits banner.
///
/// # Errors
///
/// Returns an error if the report cannot be written.
pub fn write_banner(out: &mut impl Write) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    writeln!(
        out,
        "No warranty. Deletion is permanent and cannot be undone. See DISCLAIMER.md."
    )
    .map_err(Error::report)?;
    writeln!(
        out,
        "rmbin {version} alpha. Assurance level B: best-effort overwrite. This is not a certified wipe."
    )
    .map_err(Error::report)?;
    writeln!(
        out,
        "On SSDs, APFS, snapshots and backups, overwriting cannot guarantee that the old bytes are gone."
    )
    .map_err(Error::report)?;
    writeln!(out).map_err(Error::report)?;
    writeln!(out, "Overwriting a file cannot reach:").map_err(Error::report)?;
    writeln!(
        out,
        "- copies made by APFS or another copy-on-write filesystem"
    )
    .map_err(Error::report)?;
    writeln!(out, "- snapshots, Time Machine, and server backups").map_err(Error::report)?;
    writeln!(
        out,
        "- blocks remapped by SSD wear levelling or the drive controller"
    )
    .map_err(Error::report)?;
    writeln!(
        out,
        "- compressed or sparse copies, or older bytes left when a program swaps a file"
    )
    .map_err(Error::report)?;
    writeln!(
        out,
        "- git history, caches, editor history, logs, swap, and CI artefacts"
    )
    .map_err(Error::report)?;
    writeln!(out).map_err(Error::report)?;
    writeln!(
        out,
        "One pass of repeating text is written. Random data is never used."
    )
    .map_err(Error::report)?;
    writeln!(
        out,
        "NIST SP 800-88 Rev 2: overwriting gives SSDs very little confidentiality protection."
    )
    .map_err(Error::report)?;
    writeln!(out, "{}", platform_note()).map_err(Error::report)?;
    Ok(())
}

fn platform_note() -> &'static str {
    #[cfg(windows)]
    {
        "This 0.0.1 Windows build cannot count extra hard-link names, and it cannot yet tell that a folder is on another volume. A second name for the same bytes would be overwritten too. Those checks are later."
    }
    #[cfg(not(windows))]
    {
        "Files with more than one hard link are skipped and kept. A directory on a different device is not entered."
    }
}

/// Print the plan. Nothing has been changed yet.
///
/// # Errors
///
/// Returns an error if the report cannot be written.
pub fn write_plan(out: &mut impl Write, pattern: &Pattern, actions: &[Action]) -> Result<()> {
    writeln!(out, "Plan. Nothing has been changed yet.").map_err(Error::report)?;
    writeln!(out, "Pattern: {}", pattern_label(pattern)).map_err(Error::report)?;
    if actions.is_empty() {
        writeln!(out, "(no paths)").map_err(Error::report)?;
    }
    for action in actions {
        writeln!(out, "- {}", action.preview()).map_err(Error::report)?;
    }
    Ok(())
}

fn pattern_label(pattern: &Pattern) -> String {
    let text = pattern.as_str();
    if text.chars().all(|character| !character.is_control()) {
        format!("'{text}'")
    } else {
        format!("{} text bytes", pattern.as_bytes().len())
    }
}

/// Print what happened.
///
/// # Errors
///
/// Returns an error if the report cannot be written.
pub fn write_outcomes(out: &mut impl Write, outcomes: &[Outcome]) -> Result<()> {
    writeln!(out, "Report.").map_err(Error::report)?;
    for outcome in outcomes {
        match &outcome.state {
            State::Deleted { kind } => {
                writeln!(out, "securely deleted {kind}  {}", outcome.path.display())
                    .map_err(Error::report)?
            }
            State::NotSecurelyDeleted { reason } => writeln!(
                out,
                "NOT securely deleted  {}  ({reason})",
                outcome.path.display()
            )
            .map_err(Error::report)?,
            State::NotAttempted => writeln!(
                out,
                "not attempted  {}  (the run stopped after an earlier failure)",
                outcome.path.display()
            )
            .map_err(Error::report)?,
        }
    }
    Ok(())
}
