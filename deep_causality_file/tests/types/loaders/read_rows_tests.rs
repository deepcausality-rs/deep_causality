/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `read_rows`: file columns are matched to the schema by name and reordered into
//! schema order (extra columns tolerated), and a required column absent from the file is an
//! error naming that column.

use deep_causality_file::{FromTableRow, TableRow, read_rows};
use deep_causality_haft::IoAction;
use std::io::Write;
use tempfile::tempdir;

#[derive(Debug, Clone, PartialEq)]
struct FlightPoint {
    mach: f64,
    alt_km: f64,
}

impl TableRow for FlightPoint {
    type Scalar = f64;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[("mach", "-"), ("alt", "km")];
    fn cells(&self) -> Vec<f64> {
        vec![self.mach, self.alt_km]
    }
}

impl FromTableRow for FlightPoint {
    fn from_cells(cells: &[f64]) -> Option<Self> {
        Some(Self {
            mach: cells[0],
            alt_km: cells[1],
        })
    }
}

fn write_file(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(body.as_bytes()).unwrap();
    path
}

#[test]
fn read_rows_reorders_file_columns_to_schema_order() {
    // File columns are in the opposite order from SCHEMA and carry an extra numeric column `q`.
    let dir = tempdir().unwrap();
    let path = write_file(
        dir.path(),
        "matrix.csv",
        "alt,q,mach\n#units,km,kPa,-\n11.0,23.7,1.2\n0.0,0.0,0.0\n",
    );
    let rows: Vec<FlightPoint> = read_rows(&path).run().unwrap();
    // Cells arrive in schema order (mach, alt); the extra `q` column is ignored.
    assert_eq!(
        rows,
        vec![
            FlightPoint { mach: 1.2, alt_km: 11.0 },
            FlightPoint { mach: 0.0, alt_km: 0.0 },
        ]
    );
}

#[test]
fn read_rows_names_a_missing_required_column() {
    let dir = tempdir().unwrap();
    // No "alt" column: the schema requires it.
    let path = write_file(dir.path(), "bad.csv", "mach\n#units,-\n1.2\n");
    let err = read_rows::<FlightPoint>(&path).run().unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("alt"), "error names the missing column: {msg}");
}
