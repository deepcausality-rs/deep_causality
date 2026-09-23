/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tempfile::NamedTempFile;
use std::fs;
use std::io::Write;

#[test]
fn written_bytes_are_read_back_by_path() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"a,b\n1,2\n").unwrap();
    f.flush().unwrap();
    assert_eq!(fs::read(f.path()).unwrap(), b"a,b\n1,2\n");
}

#[test]
fn consecutive_writes_append() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"ab").unwrap();
    f.write_all(b"cd").unwrap();
    f.flush().unwrap();
    assert_eq!(fs::read(f.path()).unwrap(), b"abcd");
}

#[test]
fn another_writer_can_overwrite_the_path() {
    let f = NamedTempFile::new().unwrap();
    fs::write(f.path(), b"x").unwrap();
    assert_eq!(fs::read(f.path()).unwrap(), b"x");
}
