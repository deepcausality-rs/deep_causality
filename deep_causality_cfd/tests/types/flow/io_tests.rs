/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the CFD CSV writers (`write_xy_csv`, `Report::write_series_csv`), which build deferred
//! `IoAction`s over the core `write_csv` file action.

use deep_causality_cfd::{CfdConfigBuilder, CfdFlow, IoAction, TaylorGreen, write_xy_csv};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_path(tag: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut p = std::env::temp_dir();
    p.push(format!(
        "dc_cfd_io_{}_{}_{}.tmp",
        std::process::id(),
        tag,
        n
    ));
    p
}

#[test]
fn test_write_xy_csv_constructs_without_io() {
    let path = temp_path("noeffect");
    let _action = write_xy_csv(&path, ["t", "v"], &[(0.0, 1.0)]);
    assert!(
        !path.exists(),
        "constructing the action must not write a file"
    );
}

#[test]
fn test_write_xy_csv_writes_two_columns() {
    let path = temp_path("xy");
    let series = vec![(0.0_f64, 1.5_f64), (0.5, -2.0)];
    write_xy_csv(&path, ["t", "v_probe"], &series)
        .run()
        .unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();
    let mut lines = contents.lines();
    assert_eq!(lines.next(), Some("t,v_probe"));
    assert_eq!(lines.next(), Some("0,1.5"));
    assert_eq!(lines.next(), Some("0.5,-2"));
    assert_eq!(lines.next(), None);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_write_series_csv_writes_named_columns_from_a_report() {
    // A verify run produces a Report carrying an "mms_error" series.
    let config = CfdConfigBuilder::verify::<f64, _>("io-series", TaylorGreen::new(0.1, 1.0))
        .sample_at([1.0, 0.5, 0.0], 0.0)
        .build()
        .unwrap();
    let report = CfdFlow::verify(&config).run().unwrap();
    let expected_rows = report.series("mms_error").unwrap().len();

    let path = temp_path("series");
    report
        .write_series_csv(&path, &["mms_error"])
        .run()
        .unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();
    let mut lines = contents.lines();
    assert_eq!(lines.next(), Some("mms_error"));
    assert_eq!(lines.count(), expected_rows);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_write_series_csv_ragged_missing_label_yields_header_only() {
    let config = CfdConfigBuilder::verify::<f64, _>("io-ragged", TaylorGreen::new(0.1, 1.0))
        .sample_at([1.0, 0.5, 0.0], 0.0)
        .build()
        .unwrap();
    let report = CfdFlow::verify(&config).run().unwrap();

    let path = temp_path("ragged");
    // A missing column has length 0, so the shortest-column row count is 0: header line only.
    report
        .write_series_csv(&path, &["mms_error", "does-not-exist"])
        .run()
        .unwrap();

    let contents = std::fs::read_to_string(&path).unwrap();
    assert_eq!(contents, "mms_error,does-not-exist\n");
    let _ = std::fs::remove_file(&path);
}
