/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witness for the CSV codec round-trip (`core.io.csv_roundtrip`).
//!
//! Mirrors `lean/DeepCausalityFormal/Core/Csv.lean`. `WriteCsv::render` (`','`-join, `'\n'`-terminate)
//! and `ReadCsv` (`lines()` then `split(',')`) are inverse under the precondition that no field
//! contains `','` or `'\n'`. This witness exercises the real `write_csv`/`read_csv` `IoAction`s
//! through a temp file: `parse (render header rows) = header :: rows`.

use deep_causality_core::{read_csv, write_csv};
use deep_causality_haft::IoAction;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

// A unique temp path per call, no external crate needed. Distinct per process and per call so
// concurrent or repeated runs never race on the same file.
fn temp_path(tag: &str) -> std::path::PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut p = std::env::temp_dir();
    p.push(format!(
        "dc_core_csv_roundtrip_witness_{}_{}_{}.csv",
        std::process::id(),
        tag,
        n
    ));
    p
}

// ---- core.io.csv_roundtrip ---------------------------------------------------------------------

/// THEOREM_MAP: core.io.csv_roundtrip
#[test]
fn test_csv_roundtrip() {
    let path = temp_path("roundtrip");

    // Fields contain no ',' or '\n' — the codec's precondition holds.
    let header = vec!["a".to_string(), "b".to_string()];
    let rows = vec![
        vec!["1".to_string(), "2".to_string()],
        vec!["3".to_string(), "4".to_string()],
    ];

    write_csv(&path, header.clone(), rows.clone())
        .run()
        .unwrap();
    let parsed = read_csv(&path).run().unwrap();
    let _ = std::fs::remove_file(&path);

    // parse(render(header, rows)) = header :: rows.
    let mut expected = vec![header];
    expected.extend(rows);
    assert_eq!(parsed, expected);
}
