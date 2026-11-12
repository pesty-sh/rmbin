//! # Section 08 — One confirmation
//!
//! What you'll learn: reading one line from standard input.
//! Builds on: Section 07, because the plan is what the person confirms.
//! Used by: a later version. Section 20 does not call this file.
//! Functions: F10.
//!
//! This version does not ask you to type `yes`. A yes would sound like
//! permission to delete, and this version does not delete. The function
//! stays so a later version can ask one question before it changes anything.
//!
//! The only accepted answer is the word `yes`, in lowercase, after trimming
//! space. Anything else, including an empty reply, leaves every file in place.
//! `--yes` is the switch for a script. There is no second, hidden prompt.

use std::io::{BufRead, Write};

use crate::error::{Error, Result};

/// Ask for one `yes` on `input`.
///
/// # Errors
///
/// Returns [`Error::Cancelled`] when the line is not `yes`, or an IO error
/// when the reply cannot be read.
pub fn ask(input: &mut impl BufRead, out: &mut impl Write) -> Result<()> {
    writeln!(
        out,
        "This permanently destroys the paths above. Deleted files cannot be recovered."
    )
    .map_err(Error::report)?;
    writeln!(
        out,
        "There is no warranty. By typing yes you accept that risk. See DISCLAIMER.md."
    )
    .map_err(Error::report)?;
    writeln!(out, "Type yes to continue:").map_err(Error::report)?;
    out.flush().map_err(Error::report)?;
    let mut line = String::new();
    input
        .read_line(&mut line)
        .map_err(|source| Error::Io { path: None, source })?;
    if line.trim() == "yes" {
        Ok(())
    } else {
        Err(Error::Cancelled)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::ask;

    #[test]
    fn only_yes_is_accepted() {
        let mut output = Vec::new();
        assert!(ask(&mut Cursor::new(b"yes\n"), &mut output).is_ok());
        assert!(ask(&mut Cursor::new(b" yes \n"), &mut Vec::new()).is_ok());
        assert!(ask(&mut Cursor::new(b"no\n"), &mut Vec::new()).is_err());
        assert!(ask(&mut Cursor::new(b"YES\n"), &mut Vec::new()).is_err());
        assert!(ask(&mut Cursor::new(b""), &mut Vec::new()).is_err());
        let text = String::from_utf8(output).expect("utf8");
        assert!(text.contains("permanently destroys"));
        assert!(text.contains("accept that risk"));
        assert!(text.contains("Type yes to continue:"));
    }
}
