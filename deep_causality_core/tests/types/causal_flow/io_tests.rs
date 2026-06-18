/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{CausalFlow, CausalityError, CausalityErrorEnum, read_text, write_text};
use deep_causality_haft::{IoAction, LogSize, io_fail, io_pure};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_path(tag: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut p = std::env::temp_dir();
    p.push(format!(
        "dc_flow_io_{}_{}_{}.tmp",
        std::process::id(),
        tag,
        n
    ));
    p
}

// --- source (read constructor) ---

#[test]
fn test_source_produces_value_from_action() {
    let flow = CausalFlow::source(io_pure::<i32, CausalityError>(99));
    assert_eq!(flow.finish(), Ok(99));
}

#[test]
fn test_source_routes_failure_to_error_channel() {
    let err = CausalityError::new(CausalityErrorEnum::IoError("boom".to_string()));
    let flow = CausalFlow::source(io_fail::<i32, CausalityError>(err));
    assert!(flow.is_err());
}

#[test]
fn test_read_text_from_constructs_flow_with_file_contents() {
    let path = temp_path("readfrom");
    write_text(&path, "from-file").run().unwrap();
    let out = CausalFlow::read_text_from(&path).finish();
    assert_eq!(out, Ok("from-file".to_string()));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_read_csv_from_constructs_flow_with_rows() {
    let path = temp_path("readcsvfrom");
    write_text(&path, "a,b\n1,2\n").run().unwrap();
    let out = CausalFlow::read_csv_from(&path).finish().unwrap();
    assert_eq!(
        out,
        vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["1".to_string(), "2".to_string()],
        ]
    );
    let _ = std::fs::remove_file(&path);
}

// --- commit / write_*_to (value-preserving steps) ---

#[test]
fn test_write_text_to_preserves_carried_value() {
    let path = temp_path("writepreserve");
    // The flow carries an i32; after the write it must STILL carry the same i32.
    let out = CausalFlow::value(42_i32)
        .write_text_to(&path, |v| v.to_string())
        .finish();
    assert_eq!(out, Ok(42));
    assert_eq!(read_text(&path).run().unwrap(), "42");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_write_csv_to_writes_rows_and_passes_value_through() {
    let path = temp_path("writecsvto");
    let header = vec!["v".to_string()];
    let out = CausalFlow::value(7_i32)
        .write_csv_to(&path, header, |v| vec![vec![v.to_string()]])
        .finish();
    assert_eq!(out, Ok(7));
    assert_eq!(std::fs::read(&path).unwrap(), b"v\n7\n");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_commit_appends_audit_log_entry() {
    let path = temp_path("audit");
    let flow = CausalFlow::value(1_i32).write_text_to(&path, |v| v.to_string());
    let process = flow.into_process();
    assert_eq!(
        process.logs.len(),
        1,
        "a committed write records one audit entry"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_commit_failure_short_circuits() {
    // Make the parent a file so the nested write path cannot be created.
    let mut path = temp_path("commitfail");
    std::fs::write(&path, "x").unwrap();
    path.push("nested.txt");

    let out = CausalFlow::value(5_i32)
        .write_text_to(&path, |v| v.to_string())
        .finish();
    assert!(matches!(
        out,
        Err(CausalityError(CausalityErrorEnum::IoError(_)))
    ));
}

#[test]
fn test_commit_on_errored_flow_does_not_run() {
    let path = temp_path("erroredflow");
    let start: CausalFlow<i32> =
        CausalFlow::fail(CausalityError::new(CausalityErrorEnum::Unspecified));
    let _ = start.write_text_to(&path, |v| v.to_string()).finish();
    assert!(!path.exists(), "a write on an errored flow must not run");
}
