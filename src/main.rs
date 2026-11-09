//! # Section 00 — The entry point
//!
//! What you'll learn: `fn main`, a `match`, and an exit code.
//! Builds on: L0, a Cargo binary.
//! Used by: nothing else. This file only starts the program.
//! Functions: F01.
//!
//! `main` does no deleting itself. It parses the command, calls `rmbin::execute`,
//! and turns the result into a process exit code.

#![forbid(unsafe_code)]

use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let request = match rmbin::cli::parse() {
        Ok(request) => request,
        Err(code) => return ExitCode::from(code),
    };
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    match rmbin::execute(&request, &mut input, &mut output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // The long report is already on stdout. This line is for scripts.
            let _ = writeln!(io::stderr(), "rmbin error: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}
