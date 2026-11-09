//! # Section 03 — Splitting the list
//!
//! What you'll learn: `&str` versus `String`, `split`, iterators, and `Vec`.
//! Builds on: L0 strings. An iterator yields one item at a time.
//! A `Vec` is a growable list.
//! Used by: Section 07, when the plan is built.
//! Functions: F02.
//!
//! `--list 'a.txt,build/;secrets/.env'` is split on every comma and semicolon.
//! A positional path is never split, so `odd,name.txt` stays one name.
//! Quote values in the shell so the shell does not split them first.

use std::path::PathBuf;

use crate::error::{Error, Result};

/// Paths from `--list`, then positional paths, in the order the user gave them.
///
/// # Errors
///
/// Returns [`Error::NoPaths`] when nothing was named, or [`Error::EmptyListItem`]
/// when `--list` has an empty piece such as `a,,b`.
pub fn collect_paths(list: Option<&str>, positional: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if let Some(list) = list {
        if list.trim().is_empty() {
            return Err(Error::EmptyListItem);
        }
        for piece in list.split([',', ';']) {
            let piece = piece.trim();
            if piece.is_empty() {
                return Err(Error::EmptyListItem);
            }
            paths.push(PathBuf::from(piece));
        }
    }
    for path in positional {
        if path.as_os_str().is_empty() {
            return Err(Error::EmptyListItem);
        }
        paths.push(path.clone());
    }
    if paths.is_empty() {
        return Err(Error::NoPaths);
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::collect_paths;

    #[test]
    fn splits_commas_and_semicolons() {
        let paths = collect_paths(Some("a.txt,build/;secrets/.env"), &[]).expect("paths");
        assert_eq!(
            paths,
            vec![
                PathBuf::from("a.txt"),
                PathBuf::from("build/"),
                PathBuf::from("secrets/.env"),
            ]
        );
    }

    #[test]
    fn trims_space_around_separators() {
        let paths = collect_paths(Some(" a.txt ; b.txt "), &[]).expect("paths");
        assert_eq!(paths, vec![PathBuf::from("a.txt"), PathBuf::from("b.txt")]);
    }

    #[test]
    fn empty_piece_is_an_error() {
        assert!(collect_paths(Some("a,,b"), &[]).is_err());
        assert!(collect_paths(Some(""), &[]).is_err());
    }

    #[test]
    fn positional_paths_are_not_split() {
        let positional = vec![PathBuf::from("odd,name.txt"), PathBuf::from("other;file")];
        let paths = collect_paths(None, &positional).expect("paths");
        assert_eq!(paths, positional);
    }

    #[test]
    fn list_and_positional_paths_are_both_kept() {
        let paths = collect_paths(Some("a;b"), &[PathBuf::from("odd,name.txt")]).expect("paths");
        assert_eq!(
            paths,
            vec![
                PathBuf::from("a"),
                PathBuf::from("b"),
                PathBuf::from("odd,name.txt"),
            ]
        );
    }

    #[test]
    fn no_paths_is_an_error() {
        assert!(collect_paths(None, &[]).is_err());
    }
}
