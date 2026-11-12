//! # rmbin
//!
//! `rmbin` prints a plan for removing files and folders. This version does
//! not overwrite, rename, move, or delete. A later version will.
//!
//! This crate is also a short Rust tutorial. Read the modules in section
//! order. `docs/TUTORIAL.md` is the guide. Each module teaches one idea.
//!
//! The running program does not open a socket. It has no update check and no
//! telemetry. Builds can download crates; the program you run does not.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod classify;
pub mod cli;
pub mod confirm;
pub mod core;
pub mod disk;
pub mod error;
pub mod identity;
pub mod input;
pub mod overwrite;
pub mod pattern;
pub mod plan;
pub mod protect;
pub mod readback;
pub mod redact;
pub mod remain;
pub mod remove;
pub mod rename_name;
pub mod report;
pub mod run;
pub mod snapshot;
pub mod sparse;

pub use error::{Error, Result};
pub use run::execute;
