//! # Calling the dry-run
//!
//! What you'll learn: one function that hands work to Section 20.
//! Builds on: Section 20 (`core`).
//! Used by: Section 00 (`main`).
//! Functions: the command entry.
//!
//! `main` calls `execute`. `execute` calls `core::execute`. The steps live
//! in Section 20 so that file can be read as a list. This file stays thin
//! so `main` does not need to know the section number.

use std::io::{BufRead, Write};

use crate::cli::Cli;
use crate::core;
use crate::error::Result;

/// Run one command that has already been parsed.
///
/// This version only prints a plan. See Section 20.
///
/// # Errors
///
/// A usage problem, a refused path, or a request to delete.
pub fn execute(request: &Cli, input: &mut impl BufRead, out: &mut impl Write) -> Result<()> {
    core::execute(request, input, out)
}
