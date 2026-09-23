/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tempfile::TempDir;
use std::fs;

#[test]
fn drop_removes_a_populated_tree() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().to_path_buf();
    fs::write(p.join("top.csv"), b"a").unwrap();
    fs::create_dir(p.join("sub")).unwrap();
    fs::write(p.join("sub").join("inner.csv"), b"b").unwrap();
    drop(dir);
    assert!(!p.exists());
}

#[test]
fn drop_removes_an_empty_directory() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().to_path_buf();
    drop(dir);
    assert!(!p.exists());
}

#[test]
fn drop_after_external_removal_does_not_panic() {
    let dir = TempDir::new().unwrap();
    fs::remove_dir_all(dir.path()).unwrap();
    drop(dir);
}
