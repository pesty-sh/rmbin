//! # rmbin
//!
//! `rmbin` overwrites a file with a repeating text pattern, asks the operating
//! system to flush that write, and only then deletes the name.
//!
//! This crate is also a short Rust tutorial. Read the modules in section order.
//! `docs/TUTORIAL.md` is the guide. Each module teaches one idea.
//!
//! The running program does not open a socket. It has no update check and no
//! telemetry. Builds can download crates; the program you run does not.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod classify;
pub mod cli;
pub mod confirm;
pub mod error;
pub mod input;
pub mod overwrite;
pub mod pattern;
pub mod plan;
pub mod protect;
pub mod remove;
pub mod report;
pub mod run;

pub use error::{Error, Result};
pub use run::execute;
