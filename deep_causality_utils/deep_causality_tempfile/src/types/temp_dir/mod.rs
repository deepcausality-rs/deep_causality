/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A scratch directory removed on drop.

mod temp_dir_drop;

use std::io;
use std::path::{Path, PathBuf};

/// A new, empty directory under `std::env::temp_dir()`, removed with its contents on drop.
///
/// On Unix the directory is created with mode `0o700`, before the umask applies. Creation fails
/// with `ErrorKind::AlreadyExists` rather than reuse an existing path.
///
/// ```
/// use deep_causality_tempfile::TempDir;
///
/// let dir = TempDir::new().unwrap();
/// std::fs::write(dir.path().join("a.csv"), b"x,y\n").unwrap();
/// let path = dir.path().to_path_buf();
/// drop(dir);
/// assert!(!path.exists());
/// ```
#[derive(Debug)]
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    /// Creates the directory.
    ///
    /// # Errors
    ///
    /// Returns the `std::io::Error` of the failed directory creation.
    pub fn new() -> io::Result<TempDir> {
        unimplemented!()
    }

    /// The absolute path of the directory.
    pub fn path(&self) -> &Path {
        let _ = &self.path;
        unimplemented!()
    }

    /// Creates the directory at `path`, failing with `ErrorKind::AlreadyExists` if any entry is
    /// there.
    pub(crate) fn create_at(path: PathBuf) -> io::Result<TempDir> {
        let _ = path;
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::TempDir;
    use std::io::ErrorKind;
    use std::path::PathBuf;
    use std::{env, fs, process};

    // A path unique to one test in this process. The guard removes whatever the test planted
    // there, including when the test panics.
    struct Probe(PathBuf);

    impl Drop for Probe {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
            let _ = fs::remove_file(&self.0);
        }
    }

    fn probe(name: &str) -> Probe {
        Probe(env::temp_dir().join(format!("dct-unit-{}-dir-{name}", process::id())))
    }

    #[test]
    fn create_at_fresh_path_creates_that_directory() {
        let p = probe("fresh");
        let dir = TempDir::create_at(p.0.clone()).unwrap();
        assert_eq!(dir.path(), p.0.as_path());
        assert!(p.0.is_dir());
    }

    #[test]
    fn create_at_existing_directory_is_already_exists() {
        let p = probe("existing_dir");
        fs::create_dir(&p.0).unwrap();
        fs::write(p.0.join("keep"), b"keep").unwrap();
        let err = TempDir::create_at(p.0.clone()).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::AlreadyExists);
        assert_eq!(fs::read(p.0.join("keep")).unwrap(), b"keep");
    }

    #[test]
    fn create_at_existing_file_is_already_exists() {
        let p = probe("existing_file");
        fs::write(&p.0, b"keep").unwrap();
        let err = TempDir::create_at(p.0.clone()).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&p.0).unwrap(), b"keep");
    }
}
