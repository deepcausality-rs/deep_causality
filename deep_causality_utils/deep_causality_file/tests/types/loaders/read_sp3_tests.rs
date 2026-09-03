/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use chrono::NaiveDate;
use deep_causality_file::{DataLoadingError, OrbitData, ReadOrbitData, read_orbit_data};
use deep_causality_haft::IoAction;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

/// Write `content` to a temporary `.sp3` file kept alive for the test's duration.
fn write_sp3(content: &str) -> NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(".sp3").tempfile().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f.flush().unwrap();
    f
}

fn run(path: &Path, sat: &str) -> Result<Vec<OrbitData<f64>>, DataLoadingError> {
    read_orbit_data::<f64>(path, sat).run()
}

const EPOCH: &str = "*  2016  7  1  0  0  0.00000000\n";

#[test]
fn test_parses_standard_position_km_to_m() {
    let f = write_sp3(&format!(
        "{EPOCH}P E14  12345.678901  -23456.789012  3456.789012\n"
    ));
    let orbits = run(f.path(), "E14").unwrap();
    assert_eq!(orbits.len(), 1);
    // SP3 is kilometres; the loader converts to metres (×1000).
    assert!((orbits[0].x_m() - 12_345_678.901).abs() < 1e-3);
    assert!((orbits[0].y_m() - (-23_456_789.012)).abs() < 1e-3);
    assert!((orbits[0].z_m() - 3_456_789.012).abs() < 1e-3);
    assert_eq!(
        orbits[0].timestamp(),
        NaiveDate::from_ymd_opt(2016, 7, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    );
}

#[test]
fn test_parses_compact_position_form() {
    // "PE14" (no space) is the compact record; coordinates start one field earlier.
    let f = write_sp3(&format!(
        "{EPOCH}PE14  12345.678901 -23456.789012 3456.789012\n"
    ));
    let orbits = run(f.path(), "E14").unwrap();
    assert_eq!(orbits.len(), 1);
    assert!((orbits[0].x_m() - 12_345_678.901).abs() < 1e-3);
}

#[test]
fn test_unknown_target_satellite_errors() {
    let f = write_sp3(EPOCH);
    let err = run(f.path(), "E99").unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("unknown identifier"), "got: {msg}");
    assert!(msg.contains("E99"), "got: {msg}");
    // The Unknown representation reports no underlying source.
    assert!(std::error::Error::source(&err).is_none());
}

#[test]
fn test_position_for_other_satellite_is_skipped() {
    let f = write_sp3(&format!("{EPOCH}P E18  1.0  2.0  3.0\n"));
    assert!(run(f.path(), "E14").unwrap().is_empty());
}

#[test]
fn test_position_before_epoch_is_skipped() {
    // No epoch seen yet → current time is unknown → the position line is ignored.
    let f = write_sp3("P E14  1.0  2.0  3.0\n");
    assert!(run(f.path(), "E14").unwrap().is_empty());
}

#[test]
fn test_short_epoch_line_is_skipped() {
    // Fewer than 7 epoch fields → skipped; the following position has no epoch and is ignored.
    let f = write_sp3("*  2016  7  1\nP E14  1.0  2.0  3.0\n");
    assert!(run(f.path(), "E14").unwrap().is_empty());
}

#[test]
fn test_short_line_is_skipped() {
    // A one-character line is below the 2-byte minimum and is skipped before any parsing.
    let f = write_sp3(&format!("x\n{EPOCH}P E14  1.0  2.0  3.0\n"));
    assert_eq!(run(f.path(), "E14").unwrap().len(), 1);
}

#[test]
fn test_bare_p_line_is_skipped() {
    // "P " has the marker but no satellite field → skipped.
    let f = write_sp3(&format!("{EPOCH}P \nP E14  1.0  2.0  3.0\n"));
    assert_eq!(run(f.path(), "E14").unwrap().len(), 1);
}

#[test]
fn test_insufficient_coordinates_is_skipped() {
    // Only two coordinate fields where three are required → skipped.
    let f = write_sp3(&format!("{EPOCH}P E14  100.0  200.0\n"));
    assert!(run(f.path(), "E14").unwrap().is_empty());
}

#[test]
fn test_invalid_epoch_year_is_parse_error() {
    let f = write_sp3("*  YEAR  7  1  0  0  0.0\n");
    let err = run(f.path(), "E14").unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("parse error in epoch year"), "got: {msg}");
}

#[test]
fn test_invalid_epoch_month_is_parse_error() {
    let f = write_sp3("*  2016  XX  1  0  0  0.0\n");
    let err = run(f.path(), "E14").unwrap_err();
    assert!(format!("{err}").contains("epoch month"));
}

#[test]
fn test_invalid_epoch_day_is_parse_error() {
    let f = write_sp3("*  2016  7  XX  0  0  0.0\n");
    let err = run(f.path(), "E14").unwrap_err();
    assert!(format!("{err}").contains("epoch day"));
}

#[test]
fn test_invalid_epoch_hour_is_parse_error() {
    let f = write_sp3("*  2016  7  1  XX  0  0.0\n");
    let err = run(f.path(), "E14").unwrap_err();
    assert!(format!("{err}").contains("epoch hour"));
}

#[test]
fn test_invalid_epoch_minute_is_parse_error() {
    let f = write_sp3("*  2016  7  1  0  XX  0.0\n");
    let err = run(f.path(), "E14").unwrap_err();
    assert!(format!("{err}").contains("epoch minute"));
}

#[test]
fn test_invalid_epoch_second_is_parse_error() {
    let f = write_sp3("*  2016  7  1  0  0  XX\n");
    let err = run(f.path(), "E14").unwrap_err();
    assert!(format!("{err}").contains("epoch second"));
}

#[test]
fn test_out_of_range_epoch_date_is_parse_error() {
    let f = write_sp3("*  2016  13  1  0  0  0.0\n");
    let err = run(f.path(), "E14").unwrap_err();
    assert!(format!("{err}").contains("epoch date"));
}

#[test]
fn test_out_of_range_epoch_time_is_parse_error() {
    let f = write_sp3("*  2016  7  1  25  0  0.0\n");
    let err = run(f.path(), "E14").unwrap_err();
    assert!(format!("{err}").contains("epoch time"));
}

#[test]
fn test_invalid_x_coordinate_is_parse_error() {
    let f = write_sp3(&format!("{EPOCH}P E14  bad  2.0  3.0\n"));
    let err = run(f.path(), "E14").unwrap_err();
    assert!(format!("{err}").contains("position x"));
}

#[test]
fn test_invalid_y_coordinate_is_parse_error() {
    let f = write_sp3(&format!("{EPOCH}P E14  1.0  bad  3.0\n"));
    let err = run(f.path(), "E14").unwrap_err();
    assert!(format!("{err}").contains("position y"));
}

#[test]
fn test_invalid_z_coordinate_is_parse_error() {
    let f = write_sp3(&format!("{EPOCH}P E14  1.0  2.0  bad\n"));
    let err = run(f.path(), "E14").unwrap_err();
    assert!(format!("{err}").contains("position z"));
}

#[test]
fn test_missing_file_is_io_error() {
    let err = run(Path::new("/no/such/path.sp3"), "E14").unwrap_err();
    assert!(format!("{err}").contains("I/O error"));
}

#[test]
fn test_read_orbit_data_returns_lazy_action() {
    let _action: ReadOrbitData<f64> = read_orbit_data::<f64>("unread.sp3", "E14");
}
