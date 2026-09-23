/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tempfile::NamedTempFile;
use std::collections::HashSet;
use std::io::ErrorKind;
use std::{env, fs, process, thread};

fn file_name(f: &NamedTempFile) -> String {
    f.path().file_name().unwrap().to_str().unwrap().to_string()
}

#[test]
fn new_creates_an_empty_absolute_file_under_temp_dir() {
    let f = NamedTempFile::new().unwrap();
    let p = f.path();
    let meta = fs::metadata(p).unwrap();
    assert!(meta.is_file());
    assert_eq!(meta.len(), 0);
    assert_eq!(p.parent(), Some(env::temp_dir().as_path()));
    assert!(p.is_absolute());
}

#[test]
fn thousand_files_have_distinct_paths() {
    let files: Vec<NamedTempFile> = (0..1000).map(|_| NamedTempFile::new().unwrap()).collect();
    let paths: HashSet<_> = files.iter().map(|f| f.path().to_path_buf()).collect();
    assert_eq!(paths.len(), 1000);
}

#[test]
fn concurrent_creation_yields_distinct_paths() {
    let files: Vec<NamedTempFile> = thread::scope(|s| {
        let handles: Vec<_> = (0..8)
            .map(|_| {
                s.spawn(|| {
                    (0..100)
                        .map(|_| NamedTempFile::new().unwrap())
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });
    let paths: HashSet<_> = files.iter().map(|f| f.path().to_path_buf()).collect();
    assert_eq!(paths.len(), 800);
}

#[test]
fn dotted_suffix_is_the_extension() {
    let f = NamedTempFile::with_suffix(".csv").unwrap();
    assert!(file_name(&f).ends_with(".csv"));
    assert_eq!(f.path().extension().unwrap(), "csv");
    assert!(f.path().is_file());
}

#[test]
fn undotted_suffix_is_appended_verbatim() {
    let f = NamedTempFile::with_suffix("_clk").unwrap();
    assert!(file_name(&f).ends_with("_clk"));
    assert!(f.path().is_file());
}

#[test]
fn empty_suffix_creates_a_file_under_temp_dir() {
    let f = NamedTempFile::with_suffix("").unwrap();
    assert!(f.path().is_file());
    assert_eq!(f.path().parent(), Some(env::temp_dir().as_path()));
}

#[test]
fn dot_suffixes_stay_inside_temp_dir() {
    for suffix in [".", ".."] {
        let f = NamedTempFile::with_suffix(suffix).unwrap();
        assert!(file_name(&f).ends_with(suffix), "{suffix}");
        assert_eq!(f.path().parent(), Some(env::temp_dir().as_path()));
        assert!(f.path().is_file());
    }
}

#[test]
fn slash_in_suffix_is_invalid_input_and_creates_nothing() {
    let marker = format!("dct-escape-marker-{}", process::id());
    let err = NamedTempFile::with_suffix(&format!("/../{marker}.csv")).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);
    let tmp = env::temp_dir();
    assert!(!tmp.join(format!("{marker}.csv")).exists());
    assert!(!tmp.parent().unwrap().join(format!("{marker}.csv")).exists());
}

#[test]
fn backslash_in_suffix_is_invalid_input_on_every_platform() {
    let err = NamedTempFile::with_suffix("a\\b.csv").unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidInput);
}

#[cfg(unix)]
#[test]
fn file_mode_grants_nothing_to_group_or_other() {
    // The umask can only clear bits of the requested 0o600, so under any umask no group or other
    // bit may be set. Under a umask that leaves those bits, 0o644 would set them.
    use std::os::unix::fs::PermissionsExt;
    let f = NamedTempFile::new().unwrap();
    let mode = fs::metadata(f.path()).unwrap().permissions().mode();
    assert_eq!(mode & 0o077, 0, "{mode:o}");
}
