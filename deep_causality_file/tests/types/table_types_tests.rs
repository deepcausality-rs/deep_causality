/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! NumericTable shape validation and accessors.

use deep_causality_file::{NumericTable, TableColumn};

#[test]
fn construction_validates_rectangularity() {
    let cols = vec![TableColumn::new("a", ""), TableColumn::new("b", "K")];
    assert!(NumericTable::new(cols.clone(), vec![vec![1.0_f64, 2.0]]).is_some());
    assert!(NumericTable::<f64>::new(cols.clone(), vec![vec![1.0]]).is_none());
    let empty = NumericTable::<f64>::new(cols, vec![]).expect("no rows is fine");
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
}

#[test]
fn column_lookup_by_name() {
    let t = NumericTable::new(
        vec![TableColumn::new("mach", "-"), TableColumn::new("alt", "km")],
        vec![vec![25.0_f64, 61.0]],
    )
    .expect("rectangular");
    assert_eq!(t.column_index("alt"), Some(1));
    assert_eq!(t.column_index("missing"), None);
    assert_eq!(t.columns()[1].unit(), "km");
}

#[test]
fn from_columns_equals_the_explicit_constructor_and_validates() {
    let rows = vec![vec![25.0_f64, 61.0], vec![8.5, 47.0]];
    let a = NumericTable::from_columns([("mach", "-"), ("alt", "km")], rows.clone())
        .expect("rectangular");
    let b = NumericTable::new(
        vec![TableColumn::new("mach", "-"), TableColumn::new("alt", "km")],
        rows,
    )
    .expect("rectangular");
    assert_eq!(a, b);
    assert!(NumericTable::from_columns([("a", ""), ("b", "")], vec![vec![1.0_f64]]).is_none());
}
