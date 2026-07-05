/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The sensor-trace loader: gap honesty (absent, never a sentinel), laziness, typed lifts,
//! and malformed-trace errors.

use deep_causality_file::read_sensor_trace;
use deep_causality_haft::IoAction;
use std::fs;

fn write_temp(content: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("trace.csv");
    fs::write(&path, content).expect("write");
    (dir, path)
}

#[test]
fn an_intermittent_channel_keeps_its_gaps() {
    let (_d, path) =
        write_temp("t,p_tap,q_gauge\n#units,s,kPa,W/cm2\n0.0,101.3,\n0.1,,3.2\n0.2,101.1,3.1\n");
    let trace = read_sensor_trace::<f64>(&path).run().expect("parses");
    assert_eq!(trace.timestamps(), &[0.0, 0.1, 0.2]);
    let p = trace.channel("p_tap").expect("channel");
    assert_eq!(p.unit(), "kPa");
    assert_eq!(p.samples(), &[Some(101.3), None, Some(101.1)]);
    let q = trace.channel("q_gauge").expect("channel");
    assert_eq!(q.samples(), &[None, Some(3.2), Some(3.1)]);
}

#[test]
fn the_load_is_lazy_until_run() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("late.csv");
    let action = read_sensor_trace::<f64>(&path);
    fs::write(&path, "t,a\n0.0,1.0\n").expect("write");
    let trace = action.run().expect("reads after description");
    assert_eq!(trace.channels().len(), 1);
}

#[test]
fn a_missing_timestamp_is_an_error() {
    let (_d, path) = write_temp("t,a\n,1.0\n");
    let err = read_sensor_trace::<f64>(&path).run().expect_err("no time");
    assert!(err.to_string().contains("timestamp"), "{err}");
}

#[test]
fn a_non_numeric_sample_is_an_error_not_a_gap() {
    let (_d, path) = write_temp("t,a\n0.0,broken\n");
    let err = read_sensor_trace::<f64>(&path).run().expect_err("bad cell");
    let msg = err.to_string();
    assert!(msg.contains("channel 'a'"), "{msg}");
    assert!(msg.contains("'broken'"), "{msg}");
}

#[test]
fn a_duplicate_channel_name_is_an_error() {
    let (_d, path) = write_temp("t,a,a\n0.0,1.0,2.0\n");
    let err = read_sensor_trace::<f64>(&path)
        .run()
        .expect_err("duplicate channel");
    let msg = err.to_string();
    assert!(msg.contains("duplicate column name"), "{msg}");
    assert!(msg.contains("'a'"), "{msg}");
}

#[test]
fn a_units_lookalike_comment_stays_a_comment() {
    // `#units-note` starts with `#units` but its first cell is not the exact marker, so it is
    // a comment, not the units row.
    let (_d, path) = write_temp("t,a\n#units,s,V\n#units-note, skip\n0.0,1.0\n");
    let trace = read_sensor_trace::<f64>(&path).run().expect("parses");
    assert_eq!(trace.timestamps(), &[0.0]);
    let a = trace.channel("a").expect("channel");
    assert_eq!(a.unit(), "V");
    assert_eq!(a.samples(), &[Some(1.0)]);
}

#[test]
fn a_trace_needs_at_least_one_channel() {
    let (_d, path) = write_temp("t\n0.0\n");
    let err = read_sensor_trace::<f64>(&path)
        .run()
        .expect_err("no channel");
    assert!(err.to_string().contains("at least one channel"), "{err}");
}
