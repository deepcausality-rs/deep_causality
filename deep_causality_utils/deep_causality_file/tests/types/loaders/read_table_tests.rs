/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The typed table reader: laziness, the two-row header, exact lifts, and error-not-guess
//! semantics for malformed input.

use deep_causality_file::read_table;
use deep_causality_haft::IoAction;
use std::fs;

fn write_temp(content: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("table.csv");
    fs::write(&path, content).expect("write");
    (dir, path)
}

#[test]
fn the_read_is_lazy_until_run() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("late.csv");
    // Describe the read while the file does not exist yet: constructing the action is pure.
    let action = read_table::<f64>(&path);
    fs::write(&path, "mach,alt_km\n#units,-,km\n25.0,61.0\n").expect("write");
    let table = action
        .run()
        .expect("reads the file created after description");
    assert_eq!(table.len(), 1);
}

#[test]
fn a_missing_file_is_an_error_at_run() {
    let dir = tempfile::tempdir().expect("tempdir");
    let action = read_table::<f64>(dir.path().join("absent.csv"));
    assert!(action.run().is_err());
}

#[test]
fn names_units_and_exact_values_round_trip() {
    let (_d, path) = write_temp(
        "mach,alt_km,q_kpa\n#units,-,km,kPa\n\n# a comment line\n25.0,61.0,0.1\n8.5,47.0,1e-3\n",
    );
    let table = read_table::<f64>(&path).run().expect("parses");
    let names: Vec<&str> = table.columns().iter().map(|c| c.name()).collect();
    assert_eq!(names, ["mach", "alt_km", "q_kpa"]);
    let units: Vec<&str> = table.columns().iter().map(|c| c.unit()).collect();
    assert_eq!(units, ["-", "km", "kPa"]);
    assert_eq!(table.len(), 2);
    // Exact f64: the decimal literals recover their exact parsed bits.
    assert_eq!(table.rows()[0][2].to_bits(), 0.1f64.to_bits());
    assert_eq!(table.rows()[1][2].to_bits(), 1e-3f64.to_bits());
    assert_eq!(table.column_index("alt_km"), Some(1));
}

#[test]
fn the_units_row_is_optional() {
    let (_d, path) = write_temp("a,b\n1,2\n");
    let table = read_table::<f64>(&path).run().expect("parses");
    assert_eq!(table.columns()[0].unit(), "");
    assert_eq!(table.rows()[0], vec![1.0, 2.0]);
}

#[test]
fn a_ragged_row_is_an_error_naming_path_and_row() {
    let (_d, path) = write_temp("a,b\n1,2\n3\n");
    let err = read_table::<f64>(&path).run().expect_err("ragged row");
    let msg = err.to_string();
    assert!(msg.contains("table.csv"), "{msg}");
    assert!(msg.contains("row 3"), "{msg}");
    assert!(msg.contains("ragged"), "{msg}");
}

#[test]
fn a_non_numeric_cell_is_an_error_naming_the_column() {
    let (_d, path) = write_temp("a,b\n1,x\n");
    let err = read_table::<f64>(&path).run().expect_err("bad cell");
    let msg = err.to_string();
    assert!(msg.contains("column 'b'"), "{msg}");
    assert!(msg.contains("'x'"), "{msg}");
}

#[test]
fn an_empty_file_is_a_missing_header_error() {
    let (_d, path) = write_temp("");
    let err = read_table::<f64>(&path).run().expect_err("no header");
    assert!(err.to_string().contains("missing header"));
}

#[test]
fn a_malformed_units_row_is_an_error() {
    let (_d, path) = write_temp("a,b\n#units,K\n1,2\n");
    let err = read_table::<f64>(&path).run().expect_err("short units");
    assert!(err.to_string().contains("#units"), "{err}");
}

#[test]
fn a_units_lookalike_comment_stays_a_comment() {
    // `#units-note` merely starts with `#units`; its first cell is not the exact marker, so it
    // is an ordinary comment and must not be mistaken for the units row.
    let (_d, path) = write_temp("a,b\n#units,K,K\n#units-note, ignore me\n1,2\n");
    let table = read_table::<f64>(&path).run().expect("parses");
    assert_eq!(table.columns()[0].unit(), "K");
    assert_eq!(table.len(), 1);
    assert_eq!(table.rows()[0], vec![1.0, 2.0]);
}

#[test]
fn a_duplicate_column_name_is_an_error() {
    let (_d, path) = write_temp("a,a\n1,2\n");
    let err = read_table::<f64>(&path).run().expect_err("duplicate name");
    let msg = err.to_string();
    assert!(msg.contains("duplicate column name"), "{msg}");
    assert!(msg.contains("'a'"), "{msg}");
}

#[test]
fn a_single_column_table_parses() {
    let (_d, path) = write_temp("q\n#units,Pa\n101325\n");
    let table = read_table::<f64>(&path).run().expect("parses");
    assert_eq!(table.columns().len(), 1);
    assert_eq!(table.columns()[0].unit(), "Pa");
    assert_eq!(table.rows()[0][0], 101_325.0);
}
