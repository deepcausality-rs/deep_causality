/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_file::{ReadClockData, read_clock_data};
use deep_causality_haft::IoAction;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

/// Write `content` to a temporary `.clk` file kept alive for the test's duration.
fn write_clk(content: &str) -> NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(".clk").tempfile().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f.flush().unwrap();
    f
}

fn load(path: &Path, sat: &str) -> Vec<deep_causality_file::ClockData<f64>> {
    read_clock_data::<f64>(path, sat).run().unwrap()
}

#[test]
fn test_parses_valid_record() {
    let f = write_clk("AS E14 2016 07 01 00 00 00.000000  2  0.000123456789\n");
    let clocks = load(f.path(), "E14");
    assert_eq!(clocks.len(), 1);
    assert!((clocks[0].bias_s() - 0.000_123_456_789).abs() < 1e-15);
}

#[test]
fn test_skips_other_satellite() {
    let f = write_clk("AS E18 2016 07 01 00 00 00.000000  2  0.1\n");
    assert!(load(f.path(), "E14").is_empty());
}

#[test]
fn test_skips_non_as_lines() {
    // Header / comment lines that do not start with "AS" are ignored.
    let f = write_clk(
        "RINEX VERSION / TYPE\n\
         COMMENT some header\n\
         AS E14 2016 07 01 00 00 00.000000  2  0.5\n",
    );
    assert_eq!(load(f.path(), "E14").len(), 1);
}

#[test]
fn test_skips_short_record() {
    // Fewer than 10 whitespace fields → malformed → skipped.
    let f = write_clk("AS E14 2016 07 01 00\n");
    assert!(load(f.path(), "E14").is_empty());
}

#[test]
fn test_skips_invalid_bias_sentinel() {
    // IGS sentinel 999999.999999 marks invalid clock data and must be filtered.
    let f = write_clk("AS E14 2016 07 01 00 00 00.000000  2  999999.999999\n");
    assert!(load(f.path(), "E14").is_empty());
}

#[test]
fn test_skips_unparseable_bias() {
    // A non-numeric bias falls back to the sentinel default and is filtered out.
    let f = write_clk("AS E14 2016 07 01 00 00 00.000000  2  not_a_number\n");
    assert!(load(f.path(), "E14").is_empty());
}

#[test]
fn test_skips_invalid_date() {
    // Month 13 is out of range → NaiveDate fails → record skipped.
    let f = write_clk("AS E14 2016 13 40 00 00 00.000000  2  0.1\n");
    assert!(load(f.path(), "E14").is_empty());
}

#[test]
fn test_skips_unknown_satellite_code() {
    // The string matches the target, but "E99" is not a known SatId → skipped.
    let f = write_clk("AS E99 2016 07 01 00 00 00.000000  2  0.1\n");
    assert!(load(f.path(), "E99").is_empty());
}

#[test]
fn test_missing_file_is_io_error() {
    let action = read_clock_data::<f64>("/no/such/path.clk", "E14");
    let err = action.run().unwrap_err();
    assert!(format!("{err}").contains("I/O error"));
}

#[test]
fn test_read_clock_data_returns_lazy_action() {
    // Constructing the action performs no IO; the value is a ReadClockData description.
    let _action: ReadClockData<f64> = read_clock_data::<f64>("unread.clk", "E14");
}
