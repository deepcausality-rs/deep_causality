/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `write_rows`: the schema names and units come from the row type, and a written
//! typed table round-trips through `read_rows`.

use deep_causality_file::{FromTableRow, TableRow, read_rows, write_rows};
use deep_causality_haft::IoAction;
use tempfile::tempdir;

#[derive(Debug, Clone, PartialEq)]
struct MapRow {
    p_ratio: f64,
    mach_exit: f64,
    cf: f64,
}

impl TableRow for MapRow {
    type Scalar = f64;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[
        ("p_back_over_p0", "-"),
        ("mach_exit", "-"),
        ("thrust_coefficient", "-"),
    ];
    fn cells(&self) -> Vec<f64> {
        vec![self.p_ratio, self.mach_exit, self.cf]
    }
}

impl FromTableRow for MapRow {
    fn from_cells(cells: &[f64]) -> Option<Self> {
        Some(Self {
            p_ratio: cells[0],
            mach_exit: cells[1],
            cf: cells[2],
        })
    }
}

#[test]
fn write_rows_emits_schema_header_and_units() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("map.csv");
    let rows = vec![MapRow {
        p_ratio: 0.9,
        mach_exit: 0.4,
        cf: 1.2,
    }];
    write_rows(&path, rows).run().unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    let mut lines = text.lines();
    assert_eq!(
        lines.next().unwrap(),
        "p_back_over_p0,mach_exit,thrust_coefficient"
    );
    assert_eq!(lines.next().unwrap(), "#units,-,-,-");
}

#[test]
fn write_then_read_rows_round_trips() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("map.csv");
    let rows = vec![
        MapRow {
            p_ratio: 0.9,
            mach_exit: 0.4,
            cf: 1.2,
        },
        MapRow {
            p_ratio: 0.6,
            mach_exit: 2.12,
            cf: 1.5,
        },
    ];
    write_rows(&path, rows.clone()).run().unwrap();
    let back: Vec<MapRow> = read_rows(&path).run().unwrap();
    assert_eq!(back, rows);
}
