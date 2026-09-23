/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tempfile::NamedTempFile;
use std::fs;
use std::io::Write;

#[test]
fn drop_removes_a_written_file() {
    let mut f = NamedTempFile::with_suffix(".csv").unwrap();
    f.write_all(b"a").unwrap();
    let p = f.path().to_path_buf();
    drop(f);
    assert!(!p.exists());
}

#[test]
fn drop_after_external_removal_does_not_panic() {
    let f = NamedTempFile::new().unwrap();
    fs::remove_file(f.path()).unwrap();
    drop(f);
}
