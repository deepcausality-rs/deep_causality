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
}
