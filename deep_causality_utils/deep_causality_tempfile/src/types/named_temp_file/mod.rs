/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A scratch file removed on drop.

mod named_temp_file_drop;
mod named_temp_file_write;

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

/// A new, empty regular file under `std::env::temp_dir()`, open for writing and removed on drop.
///
/// Writes go through `std::io::Write`; bytes written and flushed are readable by path while the
/// value is alive. On Unix the file is created with mode `0o600`, before the umask applies.
/// Creation never opens an existing path: it fails with `ErrorKind::AlreadyExists` instead.
///
/// ```
/// use std::io::Write;
/// use deep_causality_tempfile::NamedTempFile;
///
/// let mut f = NamedTempFile::with_suffix(".csv").unwrap();
/// f.write_all(b"a,b\n1,2\n").unwrap();
/// f.flush().unwrap();
/// assert_eq!(std::fs::read(f.path()).unwrap(), b"a,b\n1,2\n");
/// ```
#[derive(Debug)]
pub struct NamedTempFile {
    path: PathBuf,
    file: File,
}

impl NamedTempFile {
    /// Creates the file.
    ///
    /// # Errors
    ///
    /// Returns the `std::io::Error` of the failed file creation.
    pub fn new() -> io::Result<NamedTempFile> {
        unimplemented!()
    }

    /// Creates the file with a name ending in `suffix`, such as `".csv"`.
    ///
    /// An empty suffix is the same as [`NamedTempFile::new`].
    ///
    /// # Errors
    ///
    /// Returns `ErrorKind::InvalidInput`, and creates nothing, when `suffix` contains `/` or
    /// `\`. Otherwise returns the `std::io::Error` of the failed
    /// file creation.
    pub fn with_suffix(suffix: &str) -> io::Result<NamedTempFile> {
        let _ = suffix;
        unimplemented!()
    }

    /// The absolute path of the file.
    pub fn path(&self) -> &Path {
        let _ = (&self.path, &self.file);
        unimplemented!()
    }

    /// Creates the file at `path`, failing with `ErrorKind::AlreadyExists` if any entry is there,
    /// including a symlink.
    pub(crate) fn create_at(path: PathBuf) -> io::Result<NamedTempFile> {
        let _ = path;
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::NamedTempFile;
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
        Probe(env::temp_dir().join(format!("dct-unit-{}-file-{name}", process::id())))
    }

    #[test]
    fn create_at_fresh_path_creates_that_file() {
        let p = probe("fresh");
        let f = NamedTempFile::create_at(p.0.clone()).unwrap();
        assert_eq!(f.path(), p.0.as_path());
        assert!(p.0.is_file());
    }

    #[test]
    fn create_at_existing_file_is_already_exists_and_keeps_content() {
        let p = probe("existing");
        fs::write(&p.0, b"keep").unwrap();
        let err = NamedTempFile::create_at(p.0.clone()).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&p.0).unwrap(), b"keep");
    }

    #[cfg(unix)]
    #[test]
    fn create_at_planted_symlink_is_already_exists_and_not_followed() {
        let link = probe("link");
        let target = probe("link_target");
        std::os::unix::fs::symlink(&target.0, &link.0).unwrap();
        let err = NamedTempFile::create_at(link.0.clone()).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::AlreadyExists);
        assert!(!target.0.exists());
    }
}
