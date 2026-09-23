/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tempfile::TempDir;
use std::collections::HashSet;
use std::{env, fs};

#[test]
fn new_creates_an_empty_absolute_directory_under_temp_dir() {
    let dir = TempDir::new().unwrap();
    let p = dir.path();
    assert!(p.is_dir());
    assert!(fs::read_dir(p).unwrap().next().is_none());
    assert_eq!(p.parent(), Some(env::temp_dir().as_path()));
    assert!(p.is_absolute());
}

#[test]
fn thousand_directories_have_distinct_paths() {
    let dirs: Vec<TempDir> = (0..1000).map(|_| TempDir::new().unwrap()).collect();
    let paths: HashSet<_> = dirs.iter().map(|d| d.path().to_path_buf()).collect();
    assert_eq!(paths.len(), 1000);
}

#[cfg(unix)]
#[test]
fn directory_mode_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new().unwrap();
    let mode = fs::metadata(dir.path()).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o700);
}
