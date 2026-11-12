//! # Section 02 — Command-line flags
//!
//! What you'll learn: a struct, and a derive macro. `#[derive(Parser)]` asks the
//! `clap` crate to build the parser for us. A macro runs at compile time.
//! Builds on: L0 Cargo and crates.
//! Used by: Section 00 (`main`).
//! Functions: F01.
//!
//! Flags in this version:
//! - positional paths, never split
//! - `--list`, split on `,` and `;`
//! - `--pattern`, default `lol`
//! - `--dry-run` (the command is always a dry run; this flag sets the exit code)
//! - `--yes` is accepted and ignored. This version cannot delete.
//!
//! `--recycle`, `--verify`, `--wipe-free-space`, JSON plans, and server cleanup
//! are not flags yet. Passing an unknown flag is an error. Nothing is deleted.

use std::path::PathBuf;

use clap::Parser;

/// Parsed arguments. Validation of the pattern and the path list happens later,
/// after the limits banner is printed.
#[derive(Debug, Parser)]
#[command(
    name = "rmbin",
    version,
    about = env!("CARGO_PKG_DESCRIPTION"),
    after_help = AFTER_HELP,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Comma- or semicolon-separated paths. These are split. Quote the value.
    #[arg(long, value_name = "PATHS")]
    pub list: Option<String>,

    /// Paths to remove. A comma or semicolon here is part of the name.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Repeating text written over each file. Byte i is pattern[i % length].
    #[arg(long, value_name = "TEXT", default_value = "lol")]
    pub pattern: String,

    /// Show the plan and change nothing.
    #[arg(long)]
    pub dry_run: bool,

    /// Skip the confirmation prompt. For scripts.
    #[arg(long)]
    pub yes: bool,
}

const AFTER_HELP: &str = "\
No warranty. Deletion is permanent. Read DISCLAIMER.md before use.

Overwrite is best-effort. SSDs, APFS, snapshots and backups can keep old bytes.

Examples:
  rmbin --dry-run --list 'notes.txt,build/'
  rmbin --yes 'odd,name.txt'
  rmbin --pattern 101 secret.txt

Quote paths so the shell does not split them.
A comma or semicolon splits only the --list value.

This version only prints a plan. It does not overwrite, rename, move, or delete.
--yes is ignored. Deleting is a later version.
It refuses to run as root. It only plans paths inside the sandbox folder
next to Cargo.toml. Anything else is refused, and nothing is changed.

Not in this version: --recycle, --verify, --wipe-free-space, JSON plans,
and server cleanup. An unknown flag is an error.
rmbin will not pretend those actions ran.

Exit codes: 0 a dry run found nothing to refuse. 1 a path was refused, or a
dry run found a path a later version would skip. 2 you asked to delete and
this version did not.
";

/// Read arguments from the process.
///
/// Clap prints help, version, and usage itself. On failure the `Err` value is
/// the exit code: `0` for help and version, `2` for a usage error. Nothing is
/// deleted on that path.
pub fn parse() -> std::result::Result<Cli, u8> {
    match Cli::try_parse() {
        Ok(cli) => Ok(cli),
        Err(error) => {
            let code = u8::try_from(error.exit_code()).unwrap_or(2);
            let _ = error.print();
            Err(code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::Parser;

    #[test]
    fn defaults_are_lol_and_not_a_dry_run() {
        let cli = Cli::try_parse_from(["rmbin", "notes.txt"]).expect("parse");
        assert_eq!(cli.pattern, "lol");
        assert!(!cli.dry_run);
        assert!(!cli.yes);
        assert!(cli.list.is_none());
        assert_eq!(cli.paths.len(), 1);
    }

    #[test]
    fn flags_round_trip() {
        let cli = Cli::try_parse_from([
            "rmbin",
            "--dry-run",
            "--yes",
            "--pattern",
            "101",
            "--list",
            "a;b",
            "odd,name.txt",
        ])
        .expect("parse");
        assert!(cli.dry_run);
        assert!(cli.yes);
        assert_eq!(cli.pattern, "101");
        assert_eq!(cli.list.as_deref(), Some("a;b"));
        assert_eq!(cli.paths[0].as_os_str(), "odd,name.txt");
    }
}
