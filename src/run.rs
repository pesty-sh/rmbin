//! # One run, from the banner to the report
//!
//! What you'll learn: calling the earlier sections in order. No new language idea.
//! Builds on: Sections 01 through 11.
//! Used by: Section 00 (`main`).
//! Functions: F01–F14 for the permanent delete.
//!
//! Order, and why:
//! 1. Print the limits banner before any decision.
//! 2. Resolve paths and refuse protected ones. Nothing is changed.
//! 3. Build the read-only plan.
//! 4. Show the plan.
//! 5. On a dry run, stop.
//! 6. Otherwise wait for one `yes`, unless `--yes` was passed.
//! 7. Apply the plan. If an overwrite or delete fails, stop and keep that file.
//! 8. Print the report.

use std::io::{BufRead, Write};
use std::path::PathBuf;

use crate::classify;
use crate::cli::Cli;
use crate::confirm;
use crate::error::{Error, Result};
use crate::input;
use crate::pattern::Pattern;
use crate::plan;
use crate::protect::Guard;
use crate::remove;
use crate::report;

/// Run one command that has already been parsed.
///
/// # Errors
///
/// A usage problem, a refused path, a cancelled confirmation, or any path that
/// was not securely deleted. Dry runs return an error when a real run would
/// leave a path behind, and they still change nothing.
pub fn execute(request: &Cli, input: &mut impl BufRead, out: &mut impl Write) -> Result<()> {
    report::write_banner(out)?;

    let pattern = Pattern::from_text(&request.pattern)?;
    let typed = input::collect_paths(request.list.as_deref(), &request.paths)?;
    let guard = Guard::load();

    let mut problems = Vec::new();
    let mut roots = Vec::new();
    for path in &typed {
        match prepare(&guard, path) {
            Ok(root) => roots.push(root),
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
    let actions = match plan::build(&roots) {
        Ok(actions) => actions,
        Err(error) => {
            writeln!(out, "Nothing was changed.").map_err(Error::report)?;
            writeln!(out, "NOT securely deleted  ({error})").map_err(Error::report)?;
            return Err(Error::Incomplete { count: 1 });
        }
    };

    report::write_plan(out, &pattern, &actions)?;

    if request.dry_run {
        writeln!(out, "Dry run: nothing was changed.").map_err(Error::report)?;
        let skips = actions.iter().filter(|action| action.is_skip()).count();
        if skips > 0 {
            writeln!(
                out,
                "A real run would leave {skips} path(s) NOT securely deleted."
            )
            .map_err(Error::report)?;
            return Err(Error::Incomplete { count: skips });
        }
        return Ok(());
    }

    if !request.yes {
        if let Err(error) = confirm::ask(input, out) {
            writeln!(out, "Nothing was changed.").map_err(Error::report)?;
            return Err(error);
        }
    }

    let outcomes = remove::apply(&pattern, &actions);
    report::write_outcomes(out, &outcomes)?;
    let failed = outcomes.iter().filter(|outcome| outcome.failed()).count();
    if failed > 0 {
        writeln!(
            out,
            "Nothing was restored. A file that was already overwritten stays overwritten."
        )
        .map_err(Error::report)?;
        return Err(Error::Incomplete { count: failed });
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
    // Existence is checked here so a missing path fails before any delete.
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
