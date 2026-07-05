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

/// A row whose `cells()` disagrees with its `SCHEMA` width — the writer must reject it.
#[derive(Debug, Clone)]
struct WrongWidthRow;

impl TableRow for WrongWidthRow {
    type Scalar = f64;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[("a", "-"), ("b", "-")];
    fn cells(&self) -> Vec<f64> {
        vec![1.0] // one cell for a two-column schema
    }
}

#[test]
fn write_rows_rejects_a_row_whose_cell_count_disagrees_with_the_schema() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("bad.csv");
    let err = write_rows(&path, vec![WrongWidthRow]).run().unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("1 cells") && msg.contains("2 columns"),
        "the error names the width mismatch: {msg}"
    );
    // Nothing is written when a row is rejected.
    assert!(!path.exists(), "no file is produced on a rejected row");
}

#[test]
fn write_rows_surfaces_a_filesystem_write_error() {
    // A path whose parent directory does not exist: the underlying `fs::write` fails and the IO
    // error propagates rather than being swallowed.
    let path = std::path::Path::new("/dcl_no_such_dir_xyz/map.csv");
    let rows = vec![MapRow {
        p_ratio: 0.9,
        mach_exit: 0.4,
        cf: 1.2,
    }];
    let err = write_rows(path, rows).run().unwrap_err();
    // It must be the filesystem write that failed, not a parse/schema error: the IO variant is the
    // only one that renders as an I/O error and carries the underlying `std::io::Error` as its
    // `source()`.
    assert!(
        format!("{err}").contains("I/O error"),
        "the failure is an I/O error: {err}"
    );
    assert!(
        std::error::Error::source(&err).is_some(),
        "the underlying io::Error is chained as the source"
    );
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
