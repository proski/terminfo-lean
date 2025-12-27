// Copyright 2025 Pavel Roskin
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Search for terminfo database file for the terminal

use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

const TERMINFO_DIRS: &[&str] = &[
    "/etc/terminfo",
    "/lib/terminfo",
    "/usr/share/terminfo",
    "/usr/lib/terminfo",
    "/boot/system/data/terminfo", // haiku
];

/// Errors reported when looking for a terminfo database file
#[derive(thiserror::Error, Debug, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// The name of the terminal is not valid
    #[error("InvalidTerminalName")]
    InvalidTerminalName,
    /// Terminfo file for the terminal could not be found
    #[error("File not found")]
    FileNotFound,
}

fn find_in_directory(term_name: &OsStr, dir: &Path) -> Result<PathBuf, Error> {
    let Some(first_byte) = term_name.as_encoded_bytes().first() else {
        return Err(Error::InvalidTerminalName);
    };

    // Standard layout - leaf directories use the first character of the terminal name.
    let first_char = *first_byte as char;
    let filename = dir.join(first_char.to_string()).join(term_name);
    if filename.exists() {
        return Ok(filename);
    }

    // Layout for systems with non-case-sensitive filesystems (MacOS, Windows) - leaf
    // directories use the first byte of the terminal name in hexadecimal form.
    let first_byte_hex = format!("{:02x}", *first_byte);
    let filename = dir.join(first_byte_hex).join(term_name);
    if filename.exists() {
        return Ok(filename);
    }

    Err(Error::FileNotFound)
}

/// Returns all directories that are searched for terminfo files
///
/// This function does not attempt to verify if the directories to be searched actually exist.
///
/// Returns a vector of directories.
pub fn search_directories() -> Vec<PathBuf> {
    let mut search_dirs = vec![];

    // Lazily evaluated iterator, consumed at most once.
    let mut default_dirs = TERMINFO_DIRS.iter().map(PathBuf::from);

    // Search the directory from the `TERMINFO` environment variable.
    if let Ok(dir) = env::var("TERMINFO") {
        search_dirs.push(PathBuf::from(&dir));
    }

    // Search `.terminfo` in the home directory.
    if let Some(home_dir) = env::home_dir() {
        let dir = home_dir.join(".terminfo");
        search_dirs.push(dir);
    }

    // Search colon separated directories from the `TERMINFO_DIRS`
    // environment variable.
    if let Ok(dirs) = env::var("TERMINFO_DIRS") {
        for dir in dirs.split(':') {
            if dir.is_empty() {
                // Empty directory means search the default locations.
                search_dirs.extend(&mut default_dirs);
            } else {
                search_dirs.push(PathBuf::from(dir));
            }
        }
    }

    // Search default terminfo locations (nothing is added if used already).
    search_dirs.extend(&mut default_dirs);

    search_dirs
}

/// Find terminfo database file for the terminal name
///
/// # Arguments
///
/// * `term_name` - terminal name.
///
/// Returns the file path if it exist, an error otherwise.
pub fn locate(term_name: impl AsRef<OsStr>) -> Result<PathBuf, Error> {
    for dir in search_directories() {
        match find_in_directory(term_name.as_ref(), &dir) {
            Ok(file) => return Ok(file),
            Err(Error::FileNotFound) => {}
            Err(err) => return Err(err),
        }
    }

    Err(Error::FileNotFound)
}

#[cfg(test)]
mod test {
    use std::fs::{File, create_dir, exists};

    use tempfile::tempdir;

    use super::*;

    const TERM_NAME: &str = "no-such-terminal-123";

    #[test]
    fn empty_name() {
        assert_eq!(locate(""), Err(Error::InvalidTerminalName));
    }

    #[test]
    fn missing_file() {
        // Not using TERM_NAME to avoid race conditions - `temp_env::with_vars`
        // is serialized, but we are not using that function here.
        assert_eq!(locate("no-such-terminal-1"), Err(Error::FileNotFound));
    }

    #[test]
    fn found_xterm() {
        let found_file = locate("xterm");
        assert!(found_file.is_ok());
        assert!(exists(found_file.unwrap()).unwrap());
    }

    #[test]
    fn found_standard_layout_terminfo_dirs() {
        let temp_dir = tempdir().unwrap();
        let temp_dir = temp_dir.path();
        let leaf_dir = temp_dir.join("n");
        let terminfo_file = leaf_dir.join(TERM_NAME);
        create_dir(leaf_dir).unwrap();
        File::create(&terminfo_file).unwrap();
        let terminfo_dirs = format!("foo:{}:bar", temp_dir.display());

        temp_env::with_vars(
            [("TERMINFO_DIRS", Some(terminfo_dirs)), ("TERMINFO", None)],
            || {
                assert_eq!(locate(TERM_NAME), Ok(terminfo_file));
            },
        );
    }

    #[test]
    fn found_hex_layout_terminfo_dirs() {
        let temp_dir = tempdir().unwrap();
        let temp_dir = temp_dir.path();
        let leaf_dir = temp_dir.join("6e");
        let terminfo_file = leaf_dir.join(TERM_NAME);
        create_dir(leaf_dir).unwrap();
        File::create(&terminfo_file).unwrap();
        let terminfo_dirs = format!("foo:{}:bar", temp_dir.display());

        temp_env::with_vars(
            [("TERMINFO_DIRS", Some(terminfo_dirs)), ("TERMINFO", None)],
            || {
                assert_eq!(locate(TERM_NAME), Ok(terminfo_file));
            },
        );
    }

    #[test]
    fn found_standard_layout_terminfo_variable() {
        let temp_dir = tempdir().unwrap();
        let temp_dir = temp_dir.path();
        let leaf_dir = temp_dir.join("n");
        let terminfo_file = leaf_dir.join(TERM_NAME);
        create_dir(leaf_dir).unwrap();
        File::create(&terminfo_file).unwrap();

        temp_env::with_vars(
            [("TERMINFO_DIRS", None), ("TERMINFO", Some(temp_dir))],
            || {
                assert_eq!(locate(TERM_NAME), Ok(terminfo_file));
            },
        );
    }

    #[test]
    fn dot_terminfo_standard_layout() {
        let temp_dir = tempdir().unwrap();
        let temp_dir = temp_dir.path();
        let dot_terminfo = temp_dir.join(".terminfo");
        let leaf_dir = dot_terminfo.join("n");
        let terminfo_file = leaf_dir.join(TERM_NAME);
        create_dir(dot_terminfo).unwrap();
        create_dir(leaf_dir).unwrap();
        File::create(&terminfo_file).unwrap();

        temp_env::with_vars(
            [
                ("TERMINFO_DIRS", None),
                ("TERMINFO", None),
                ("HOME", Some(temp_dir)),
            ],
            || {
                assert_eq!(locate(TERM_NAME), Ok(terminfo_file));
            },
        );
    }

    #[test]
    fn search_order() {
        let expected_dirs: Vec<PathBuf> = [
            "/my/terminfo",
            "/home/user/.terminfo",
            "/my/terminfo1",
            "/my/terminfo2",
            "/etc/terminfo",
            "/lib/terminfo",
            "/usr/share/terminfo",
            "/usr/lib/terminfo",
            "/boot/system/data/terminfo",
        ]
        .iter()
        .map(PathBuf::from)
        .collect();

        temp_env::with_vars(
            [
                ("TERMINFO_DIRS", Some("/my/terminfo1:/my/terminfo2")),
                ("TERMINFO", Some("/my/terminfo")),
                ("HOME", Some("/home/user")),
            ],
            || {
                assert_eq!(search_directories(), expected_dirs);
            },
        );
    }

    #[test]
    fn search_order_with_empty_element() {
        let expected_dirs: Vec<PathBuf> = [
            "/my/terminfo",
            "/home/user/.terminfo",
            "/my/terminfo1",
            "/etc/terminfo",
            "/lib/terminfo",
            "/usr/share/terminfo",
            "/usr/lib/terminfo",
            "/boot/system/data/terminfo",
            "/my/terminfo2",
        ]
        .iter()
        .map(PathBuf::from)
        .collect();

        temp_env::with_vars(
            [
                ("TERMINFO_DIRS", Some("/my/terminfo1::/my/terminfo2")),
                ("TERMINFO", Some("/my/terminfo")),
                ("HOME", Some("/home/user")),
            ],
            || {
                assert_eq!(search_directories(), expected_dirs);
            },
        );
    }
}
