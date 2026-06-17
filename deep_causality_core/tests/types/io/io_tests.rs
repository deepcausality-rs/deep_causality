/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{CausalityErrorEnum, read_csv, read_text, write_csv, write_text};
use deep_causality_haft::IoAction;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

// A unique temp path per call, no external crate needed. Distinct per process and per call so
// concurrent tests never collide.
fn temp_path(tag: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut p = std::env::temp_dir();
    p.push(format!("dc_io_{}_{}_{}.tmp", std::process::id(), tag, n));
    p
}

#[test]
fn test_construction_performs_no_io() {
    let path = temp_path("noeffect");
    // Build, but do not run.
    let _action = write_text(&path, "hello");
    assert!(
        !path.exists(),
        "constructing an action must not touch the filesystem"
    );
}

#[test]
fn test_write_text_then_read_text_round_trips() {
    let path = temp_path("roundtrip");
    write_text(&path, "alpha\nbeta").run().unwrap();
    let read = read_text(&path).run().unwrap();
    assert_eq!(read, "alpha\nbeta");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_write_csv_emits_caller_formatted_bytes() {
    let path = temp_path("csvbytes");
    let header = vec!["a".to_string(), "b".to_string()];
    let rows = vec![
        vec!["1".to_string(), "2".to_string()],
        vec!["3".to_string(), "4".to_string()],
    ];
    write_csv(&path, header, rows).run().unwrap();

    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(bytes, b"a,b\n1,2\n3,4\n");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_read_csv_parses_rows() {
    let path = temp_path("csvparse");
    write_text(&path, "a,b\n1,2\n3,4\n").run().unwrap();

    let rows = read_csv(&path).run().unwrap();
    assert_eq!(
        rows,
        vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["1".to_string(), "2".to_string()],
            vec!["3".to_string(), "4".to_string()],
        ]
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_read_csv_round_trips_write_csv() {
    let path = temp_path("csvround");
    let header = vec!["x".to_string(), "y".to_string()];
    let rows = vec![vec!["10".to_string(), "20".to_string()]];
    write_csv(&path, header, rows).run().unwrap();

    let parsed = read_csv(&path).run().unwrap();
    assert_eq!(
        parsed,
        vec![
            vec!["x".to_string(), "y".to_string()],
            vec!["10".to_string(), "20".to_string()],
        ]
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_read_text_missing_file_is_io_error() {
    let path = temp_path("missing");
    let err = read_text(&path).run().unwrap_err();
    assert!(matches!(err.0, CausalityErrorEnum::IoError(_)));
}

#[test]
fn test_write_text_unwritable_path_is_io_error() {
    // A child path under a non-directory parent cannot be created.
    let mut path = temp_path("notdir");
    std::fs::write(&path, "x").unwrap();
    path.push("child.txt"); // `path` is a file, so this nested write must fail.
    let err = write_text(&path, "data").run().unwrap_err();
    assert!(matches!(err.0, CausalityErrorEnum::IoError(_)));
}

#[test]
fn test_map_over_read_transforms_output() {
    let path = temp_path("map");
    write_text(&path, "7").run().unwrap();
    let n = read_text(&path)
        .map(|s| s.trim().parse::<i32>().unwrap())
        .run()
        .unwrap();
    assert_eq!(n, 7);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_and_then_composes_read_then_write() {
    let src = temp_path("src");
    let dst = temp_path("dst");
    write_text(&src, "payload").run().unwrap();

    let dst_for_action = dst.clone();
    read_text(&src)
        .and_then(move |contents| write_text(dst_for_action, contents))
        .run()
        .unwrap();

    assert_eq!(read_text(&dst).run().unwrap(), "payload");
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&dst);
}
