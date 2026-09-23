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
    /// `\`, or equals `.` or `..`. Otherwise returns the `std::io::Error` of the failed
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
}
