/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_file::{DataManager, read_gnss_single_satellite};
use deep_causality_haft::IoAction;
use std::io::Write;
use tempfile::NamedTempFile;

fn write_file(suffix: &str, content: &str) -> NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(suffix).tempfile().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f.flush().unwrap();
    f
}

fn clk_fixture() -> NamedTempFile {
    write_file(".clk", "AS E14 2016 07 01 00 00 00.000000  2  0.000123\n")
}

fn sp3_fixture() -> NamedTempFile {
    write_file(
        ".sp3",
        "*  2016  7  1  0  0  0.00000000\nP E14  12345.678901  -23456.789012  3456.789012\n",
    )
}

#[test]
fn test_composed_loader_runs_both_reads() {
    let clk = clk_fixture();
    let sp3 = sp3_fixture();
    let action = read_gnss_single_satellite::<f64>(clk.path(), sp3.path(), "E14");
    let (clocks, orbits) = action.run().unwrap();
    assert_eq!(clocks.len(), 1);
    assert_eq!(orbits.len(), 1);
}

#[test]
fn test_composed_loader_short_circuits_on_clock_error() {
    // The clock read runs first; its failure short-circuits before the orbit read.
    let sp3 = sp3_fixture();
    let action = read_gnss_single_satellite::<f64>("/no/such/file.clk", sp3.path(), "E14");
    let err = action.run().unwrap_err();
    assert!(format!("{err}").contains("I/O error"));
}

#[test]
fn test_data_manager_load_single_satellite() {
    let clk = clk_fixture();
    let sp3 = sp3_fixture();
    let mgr = DataManager::new();
    let (clocks, orbits) = mgr
        .load_gnss_single_satellite::<f64, _>(clk.path(), sp3.path(), "E14")
        .unwrap();
    assert_eq!(clocks.len(), 1);
    assert_eq!(orbits.len(), 1);
}

#[test]
fn test_data_manager_load_clock_only() {
    let clk = clk_fixture();
    let mgr = DataManager::new();
    let clocks = mgr
        .load_gnss_clock_data::<f64, _>(clk.path(), "E14")
        .unwrap();
    assert_eq!(clocks.len(), 1);
    assert!((clocks[0].bias_s() - 0.000_123).abs() < 1e-12);
}

#[test]
fn test_data_manager_load_orbit_only() {
    let sp3 = sp3_fixture();
    let mgr = DataManager;
    let orbits = mgr
        .load_gnss_orbit_data::<f64, _>(sp3.path(), "E14")
        .unwrap();
    assert_eq!(orbits.len(), 1);
}

#[test]
fn test_data_manager_is_copy_clone_debug_default() {
    let a = DataManager::new();
    let b = a; // Copy
    let c = Clone::clone(&a); // exercises the derived Clone on a Copy type
    let _ = (b, c);
    assert_eq!(format!("{a:?}"), "DataManager");
    let _default = <DataManager as Default>::default();
}
