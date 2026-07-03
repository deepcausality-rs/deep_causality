/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The result-table writer: the write-read round trip preserves names, units, and `f64` bits.

use deep_causality_file::{NumericTable, TableColumn, read_table, write_table};
use deep_causality_haft::IoAction;

#[test]
fn write_read_round_trip_preserves_semantics_and_bits() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("result.csv");

    // Awkward values on purpose: shortest-round-trip formatting must recover exact bits.
    let rows = vec![
        vec![0.1_f64, 1e-308, -0.0],
        vec![2.0 / 3.0, 101_325.0, 6.062e18],
    ];
    let table = NumericTable::new(
        vec![
            TableColumn::new("alpha", "-"),
            TableColumn::new("p", "Pa"),
            TableColumn::new("n_e", "m^-3"),
        ],
        rows.clone(),
    )
    .expect("rectangular");

    write_table(&path, table).run().expect("writes");
    let back = read_table::<f64>(&path).run().expect("reads back");

    let names: Vec<&str> = back.columns().iter().map(|c| c.name()).collect();
    assert_eq!(names, ["alpha", "p", "n_e"]);
    let units: Vec<&str> = back.columns().iter().map(|c| c.unit()).collect();
    assert_eq!(units, ["-", "Pa", "m^-3"]);
    for (row, orig) in back.rows().iter().zip(&rows) {
        for (v, o) in row.iter().zip(orig) {
            assert_eq!(v.to_bits(), o.to_bits(), "bit-identical round trip");
        }
    }
}

#[test]
fn the_write_is_lazy_until_run() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("lazy.csv");
    let table = NumericTable::new(vec![TableColumn::new("x", "")], vec![vec![1.0_f64]])
        .expect("rectangular");
    let action = write_table(&path, table);
    assert!(!path.exists(), "describing a write performs no side effect");
    action.run().expect("writes");
    assert!(path.exists());
}

#[test]
fn an_unwritable_path_is_an_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("no_such_dir").join("result.csv");
    let table = NumericTable::new(vec![TableColumn::new("x", "")], vec![vec![1.0_f64]])
        .expect("rectangular");
    assert!(write_table(&path, table).run().is_err());
}
