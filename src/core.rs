//! # Section 20 — The dry-run rm
//!
//! What you'll learn: calling the earlier sections in order, then stopping.
//! Builds on: Sections 01 through 19.
//! Used by: Section 00, through `run`.
//! Functions: the read-only plan for F08, F09, F15, F17, F18, F23, F24, F25, F30.
//!
//! This is the whole command. The steps below are the order to read.
//!
//! This version does not call:
//! - Section 08 (the yes prompt). A yes would sound like permission to delete.
//! - Section 09 (overwrite and truncate).
//! - Section 10 (rename and delete).
//!
//! The bin (`--recycle`) and filling free space are not flags yet. They are
//! later, and they are dangerous. They are named here so you can see they
//! are missing on purpose.

use std::io::{BufRead, Write};
use std::path::PathBuf;

use crate::classify;
use crate::cli::Cli;
use crate::disk;
use crate::error::{Error, Result};
use crate::identity;
use crate::input;
use crate::journal;
use crate::names;
use crate::pattern::Pattern;
use crate::plan::{self, Action, ActionKind};
use crate::plan_text;
use crate::protect::Guard;
use crate::readback;
use crate::redact;
use crate::remain;
use crate::rename_name;
use crate::report;
use crate::sandbox;
use crate::snapshot;
use crate::sparse;

/// Run one command that has already been parsed.
///
/// The command reads. It does not overwrite, rename, move, or delete.
///
/// # Errors
///
/// A usage problem, a refused path, or a request to delete. A dry run
/// returns an error when a later version would leave a path behind.
/// Nothing is changed in either case.
pub fn execute(request: &Cli, input: &mut impl BufRead, out: &mut impl Write) -> Result<()> {
    // The prompt is not used. Reading it would throw away the answer, and
    // a later version should be the one that asks.
    let _ = input;

    // Step 1. Say what this version will not do, and state the disk limits.
    report::write_banner(out)?;

    // Step 2. Stop if this process is root, then find the sandbox folder.
    // Root could destroy the drive. Paths outside the sandbox are refused
    // before a plan is printed.
    if sandbox::running_as_root() {
        writeln!(out, "Nothing was changed.").map_err(Error::report)?;
        return Err(Error::Root);
    }
    let sandbox = match sandbox::locate() {
        Ok(sandbox) => sandbox,
        Err(error) => {
            writeln!(out, "Nothing was changed.").map_err(Error::report)?;
            return Err(error);
        }
    };

    // Step 3. Check the pattern and the path list. Still no disk writes.
    let pattern = Pattern::from_text(&request.pattern)?;
    let typed = input::collect_paths(request.list.as_deref(), &request.paths)?;
    let guard = Guard::load();

    // Step 4. Resolve each path. Refuse protected places, then refuse
    // anything that is not inside the sandbox.
    let mut problems = Vec::new();
    let mut roots = Vec::new();
    for path in &typed {
        match prepare(&guard, path) {
            Ok(root) => match sandbox.require_inside(&root) {
                Ok(()) => roots.push(root),
                Err(error) => problems.push(error),
            },
            Err(error) => problems.push(error),
        }
    }
    if !problems.is_empty() {
        writeln!(out, "Nothing was changed.").map_err(Error::report)?;
        for problem in &problems {
            writeln!(out, "NOT securely deleted  ({problem})").map_err(Error::report)?;
        }
        return Err(Error::Incomplete {
            count: problems.len(),
        });
    }

    let roots = dedup_roots(roots);

    // Step 5. Read-only file list. Symlinks are not entered.
    let mut actions = match plan::build(&roots) {
        Ok(actions) => actions,
        Err(error) => {
            writeln!(out, "Nothing was changed.").map_err(Error::report)?;
            writeln!(out, "NOT securely deleted  ({error})").map_err(Error::report)?;
            return Err(Error::Incomplete { count: 1 });
        }
    };

    // Step 6. Mark sparse or compressed files. Still only reading metadata.
    if let Err(error) = mark_sparse(&mut actions) {
        writeln!(out, "Nothing was changed.").map_err(Error::report)?;
        return Err(error);
    }

    // Step 7. Say which filesystem and, on Linux, the kernel's disk flag.
    write_volume_notes(out, &roots)?;

    // Step 8. Print the plan. Every line says "would".
    report::write_plan(out, &pattern, &actions)?;
    write_later_details(out, &pattern, &actions)?;
    write_plan_seal(out, &actions)?;

    // Step 9. One sentence for the read-back, then what a delete would miss.
    writeln!(out, "{}", readback::later_step()).map_err(Error::report)?;
    writeln!(
        out,
        "Rename, the bin, free-space fill, and delete are later versions. They are not done now."
    )
    .map_err(Error::report)?;
    for line in remain::LINES {
        writeln!(out, "{line}").map_err(Error::report)?;
    }

    if request.yes {
        writeln!(out, "Ignoring --yes. This version cannot delete.").map_err(Error::report)?;
    }
    writeln!(
        out,
        "This version only prints a plan. It does not overwrite, rename, move, or delete."
    )
    .map_err(Error::report)?;
    writeln!(out, "Nothing was changed.").map_err(Error::report)?;

    // Step 10. Stop. Sections 09 and 10 are not called.
    if request.dry_run {
        writeln!(out, "Dry run: nothing was changed.").map_err(Error::report)?;
        let skips = actions.iter().filter(|action| action.is_skip()).count();
        if skips > 0 {
            writeln!(
                out,
                "A later version would leave {skips} path(s) NOT securely deleted."
            )
            .map_err(Error::report)?;
            return Err(Error::Incomplete { count: skips });
        }
        return Ok(());
    }

    Err(Error::NotInThisVersion { action: "delete" })
}

fn write_plan_seal(out: &mut impl Write, actions: &[Action]) -> Result<()> {
    let lines: Vec<String> = actions.iter().map(Action::preview).collect();
    let text = plan_text::join(&lines);
    let seal = plan_text::fingerprint(&text);
    writeln!(
        out,
        "Plan fingerprint: {seal} (not a seal; the plan text only, no file contents)."
    )
    .map_err(Error::report)?;
    let journal_lines: Vec<String> = actions
        .iter()
        .map(|action| journal::planned(&action.path))
        .collect();
    writeln!(
        out,
        "Journal: {} line(s) say planned, not done. No journal file is written.",
        journal_lines.len()
    )
    .map_err(Error::report)?;
    let planned_names: Vec<String> = actions
        .iter()
        .map(|action| action.path.display().to_string())
        .collect();
    let present_names: Vec<String> = actions
        .iter()
        .filter(|action| std::fs::symlink_metadata(&action.path).is_ok())
        .map(|action| action.path.display().to_string())
        .collect();
    let planned_refs: Vec<&str> = planned_names.iter().map(String::as_str).collect();
    let present_refs: Vec<&str> = present_names.iter().map(String::as_str).collect();
    let kept = names::still_present(&planned_refs, &present_refs);
    writeln!(
        out,
        "Still in the sandbox: {} of {} named paths.",
        kept.len(),
        planned_refs.len()
    )
    .map_err(Error::report)?;
    Ok(())
}

fn write_volume_notes(out: &mut impl Write, roots: &[PathBuf]) -> Result<()> {
    let mut seen_fs = Vec::new();
    let mut seen_disk = Vec::new();
    for root in roots {
        let sentence = snapshot::note_for(root);
        if !seen_fs.contains(&sentence) {
            writeln!(out, "{sentence}").map_err(Error::report)?;
            seen_fs.push(sentence);
        }
        if let Some(mount) = snapshot::best(root) {
            if let Some(disk_line) = disk::note(&mount.device) {
                if !seen_disk.contains(&disk_line) {
                    writeln!(out, "{disk_line}").map_err(Error::report)?;
                    seen_disk.push(disk_line);
                }
            }
        }
    }
    Ok(())
}

fn write_later_details(out: &mut impl Write, pattern: &Pattern, actions: &[Action]) -> Result<()> {
    for action in actions {
        let ActionKind::OverwriteFile { .. } = &action.kind else {
            continue;
        };
        let card = match identity::read(&action.path) {
            Ok(card) => card,
            Err(error) => {
                writeln!(out, "Nothing was changed.").map_err(Error::report)?;
                return Err(error);
            }
        };
        let id_line = redact::line("id card", &action.path);
        writeln!(out, "  {id_line}  ({})", identity::summary(&card)).map_err(Error::report)?;
        if let Some(name) = action.path.file_name() {
            let phrase = rename_name::plan_phrase(
                action.path.parent().unwrap_or(std::path::Path::new(".")),
                name,
                pattern,
            );
            writeln!(out, "  {phrase}").map_err(Error::report)?;
        }
    }
    Ok(())
}

fn mark_sparse(actions: &mut [Action]) -> Result<()> {
    for action in actions {
        if !matches!(action.kind, ActionKind::OverwriteFile { .. }) {
            continue;
        }
        let path = action.path.clone();
        if let Some(reason) = sparse::skip_reason(&path)? {
            action.kind = ActionKind::Skip {
                reason: reason.to_owned(),
            };
        }
    }
    Ok(())
}

fn prepare(guard: &Guard, typed: &std::path::Path) -> Result<PathBuf> {
    let resolved = classify::resolve_leaf(typed, guard)?;
    if let Some(rule) = guard.denial_root(&resolved) {
        return Err(Error::Protected {
            path: resolved,
            rule,
        });
    }
    // Existence is checked here so a missing path fails before any plan line
    // pretends we could remove it. This read does not change the path.
    classify::classify(&resolved)?;
    Ok(resolved)
}

fn dedup_roots(roots: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut kept: Vec<PathBuf> = Vec::new();
    for root in roots {
        if kept
            .iter()
            .any(|earlier| earlier == &root || is_inside(earlier, &root))
        {
            continue;
        }
        kept.retain(|earlier| !is_inside(&root, earlier));
        kept.push(root);
    }
    kept
}

fn is_inside(parent: &std::path::Path, child: &std::path::Path) -> bool {
    child.starts_with(parent) && child != parent
}
