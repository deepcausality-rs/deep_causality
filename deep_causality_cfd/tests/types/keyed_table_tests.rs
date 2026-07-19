/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The schema-agnostic keyed lookup table (`weather-table-consumption`): value-bracketed linear
//! interpolation over sorted-unique keyed rows, with duplicate-key rejection and a clamp marker.
//! Column vectors here stand in for the weather dispersion columns (onset, exit, dwell, …) the M5
//! example binds; the keys mirror the committed table's run order (0, +20, −25, −40, −5, +5).

use deep_causality_cfd::KeyedTable;

/// Rows in the committed weather table's run order (not temperature order), with three stand-in
/// numeric columns per key.
fn run_order_rows() -> Vec<(f64, Vec<f64>)> {
    vec![
        (0.0, vec![10.0, 20.0, 5.0]),
        (20.0, vec![12.0, 24.0, 6.0]),
        (-25.0, vec![6.0, 14.0, 3.0]),
        (-40.0, vec![4.0, 11.0, 2.0]),
        (-5.0, vec![9.0, 18.0, 4.5]),
        (5.0, vec![11.0, 22.0, 5.5]),
    ]
}

#[test]
fn rows_are_sorted_ascending_by_key_after_load() {
    let table = KeyedTable::new(run_order_rows()).expect("valid table");
    let keys: Vec<f64> = table.rows().iter().map(|(k, _)| *k).collect();
    assert_eq!(keys, vec![-40.0, -25.0, -5.0, 0.0, 5.0, 20.0]);
    // The columns travel with their key: the −40 row keeps its own values after the sort.
    assert_eq!(table.rows()[0].1, vec![4.0, 11.0, 2.0]);
}

#[test]
fn a_mid_range_key_interpolates_its_true_value_neighbors() {
    let table = KeyedTable::new(run_order_rows()).expect("valid table");
    // dT = −15 must bracket the −25 and −5 rows (indices 1 and 2 after sorting), never file-order
    // neighbors.
    let r = table.interpolate(-15.0);
    assert!(!r.clamped());
    assert_eq!(r.lower(), 1);
    assert_eq!(r.upper(), 2);
    // Halfway between −25 and −5: every column is the midpoint of its bracketing values.
    assert_eq!(r.values(), &[7.5, 16.0, 3.75]);
    // And every interpolated column lies between its bracketing values.
    for (j, v) in r.values().iter().enumerate() {
        let lo = table.rows()[1].1[j];
        let hi = table.rows()[2].1[j];
        assert!(*v >= lo.min(hi) && *v <= lo.max(hi));
    }
}

#[test]
fn an_exact_interior_key_returns_that_row_unclamped() {
    let table = KeyedTable::new(run_order_rows()).expect("valid table");
    let r = table.interpolate(-5.0);
    assert!(!r.clamped());
    assert_eq!(r.values(), &[9.0, 18.0, 4.5]);
}

#[test]
fn a_key_below_the_range_clamps_to_the_first_row_with_the_marker() {
    let table = KeyedTable::new(run_order_rows()).expect("valid table");
    // dT = −60 is below the −40 K row: clamp to it, marker set.
    let r = table.interpolate(-60.0);
    assert!(r.clamped());
    assert_eq!(r.lower(), 0);
    assert_eq!(r.upper(), 0);
    assert_eq!(r.values(), &[4.0, 11.0, 2.0]);
}

#[test]
fn a_key_above_the_range_clamps_to_the_last_row_with_the_marker() {
    let table = KeyedTable::new(run_order_rows()).expect("valid table");
    let r = table.interpolate(30.0);
    assert!(r.clamped());
    assert_eq!(r.lower(), 5);
    assert_eq!(r.upper(), 5);
    assert_eq!(r.values(), &[12.0, 24.0, 6.0]);
}

#[test]
fn an_exact_end_key_is_not_clamped() {
    let table = KeyedTable::new(run_order_rows()).expect("valid table");
    let low = table.interpolate(-40.0);
    assert!(
        !low.clamped(),
        "the exact lowest key is a boundary, not a clamp"
    );
    assert_eq!(low.values(), &[4.0, 11.0, 2.0]);
    let high = table.interpolate(20.0);
    assert!(!high.clamped());
    assert_eq!(high.values(), &[12.0, 24.0, 6.0]);
}

#[test]
fn duplicate_keys_are_rejected() {
    let rows = vec![(1.0, vec![1.0]), (2.0, vec![2.0]), (1.0, vec![3.0])];
    let err = KeyedTable::new(rows).expect_err("duplicate key rejected");
    assert!(
        format!("{err:?}").contains("duplicate"),
        "names the duplicate: {err:?}"
    );
}

#[test]
fn an_empty_table_is_rejected() {
    assert!(KeyedTable::<f64>::new(Vec::new()).is_err());
}

#[test]
fn ragged_rows_are_rejected() {
    let rows = vec![(0.0, vec![1.0, 2.0]), (1.0, vec![3.0])];
    assert!(
        KeyedTable::new(rows).is_err(),
        "inconsistent column counts rejected"
    );
}

#[test]
fn len_and_is_empty_report_the_row_count() {
    let table = KeyedTable::new(run_order_rows()).expect("valid table");
    assert_eq!(table.len(), 6);
    assert!(!table.is_empty());
}
