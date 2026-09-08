/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use chrono::NaiveDate;
use deep_causality_file::{ReadClockData, SatId, read_clock_data};
use deep_causality_haft::IoAction;
use std::fs;
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
fn test_collects_every_matching_record_in_source_order() {
    // Each other fixture holds at most one matching record, so nothing else observes that the
    // loop accumulates: a parser that returned after its first hit would pass them all.
    let f = write_clk(
        "AS E14 2016 07 01 00 00 00.000000  2  0.000123456789\n\
         AS E18 2016 07 01 00 05 00.000000  2  0.5\n\
         AS E14 2016 07 01 00 05 00.000000  2 -0.000223456789\n\
         AS E14 2016 07 01 00 10 00.000000  2  0.000323456789\n",
    );
    let clocks = load(f.path(), "E14");
    assert_eq!(clocks.len(), 3);

    let expected = [
        ((0, 0, 0), 0.000_123_456_789_f64),
        ((0, 5, 0), -0.000_223_456_789),
        ((0, 10, 0), 0.000_323_456_789),
    ];
    for (c, ((h, m, sec), bias)) in clocks.iter().zip(expected) {
        assert_eq!(c.sat_id(), SatId::E14);
        assert_eq!(
            c.timestamp(),
            NaiveDate::from_ymd_opt(2016, 7, 1)
                .unwrap()
                .and_hms_opt(h, m, sec)
                .unwrap()
        );
        assert!((c.bias_s() - bias).abs() < 1e-15);
    }
}

#[test]
fn test_a_nine_field_record_is_one_field_short() {
    // The `parts.len() < 10` guard is what keeps the `parts[9]` bias index in bounds. The other
    // skip tests sit far below it (6 fields), so only this one pins the threshold itself.
    let nine = write_clk("AS E14 2016 07 01 00 00 00.000000  2\n");
    assert!(load(nine.path(), "E14").is_empty());

    let ten = write_clk("AS E14 2016 07 01 00 00 00.000000  2  0.25\n");
    assert_eq!(load(ten.path(), "E14").len(), 1);
}

#[test]
fn test_a_well_formed_line_that_is_not_an_as_record_is_ignored() {
    // A line with the full ten fields and a matching satellite, differing only in its record
    // code: without it the "AS" prefix test is done by the field-count guard instead.
    let f = write_clk(
        "AR E14 2016 07 01 00 00 00.000000  2  0.000111\n\
         AS E14 2016 07 01 00 05 00.000000  2  0.000222\n",
    );
    let clocks = load(f.path(), "E14");
    assert_eq!(clocks.len(), 1);
    assert!((clocks[0].bias_s() - 0.000_222).abs() < 1e-15);
}

#[test]
fn test_a_non_numeric_date_or_time_field_skips_the_record() {
    // Every one of the six timestamp fields, not just the ones whose zero fallback happened to
    // be out of range. A record dated at year 0 or silently moved to midnight is indistinguishable
    // from a real reading downstream.
    for line in [
        "AS E14 xxxx 07 01 00 00 00.000000  2  0.1\n",
        "AS E14 2016 xx 01 00 00 00.000000  2  0.1\n",
        "AS E14 2016 07 xx 00 00 00.000000  2  0.1\n",
        "AS E14 2016 07 01 xx 00 00.000000  2  0.1\n",
        "AS E14 2016 07 01 00 xx 00.000000  2  0.1\n",
        "AS E14 2016 07 01 00 00 xx        2  0.1\n",
    ] {
        let f = write_clk(line);
        assert!(load(f.path(), "E14").is_empty(), "{line}");
    }
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
fn test_the_sentinel_gate_is_exclusive_at_its_threshold() {
    // The filter drops a bias strictly above 900000 s, so 900000 itself is kept and the next
    // representable value above it is not. Nothing else in this suite pins which comparison
    // the gate uses.
    let kept = write_clk("AS E14 2016 07 01 00 00 00.000000  2  900000.0\n");
    let clocks = load(kept.path(), "E14");
    assert_eq!(clocks.len(), 1);
    assert_eq!(clocks[0].bias_s(), 900_000.0);

    let dropped = write_clk("AS E14 2016 07 01 00 00 00.000000  2  900000.0000000001\n");
    assert!(load(dropped.path(), "E14").is_empty());
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
    // Constructing the action performs no IO: the description is built while the path is absent,
    // and the saved action still reads the file that is created afterwards.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("late.clk");
    let action: ReadClockData<f64> = read_clock_data::<f64>(&path, "E14");
    fs::write(
        &path,
        "AS E14 2016 07 01 00 00 00.000000  2  0.000123456789\n",
    )
    .unwrap();

    let clocks = action
        .run()
        .expect("reads the file created after description");
    assert_eq!(clocks.len(), 1);
    assert_eq!(clocks[0].sat_id(), SatId::E14);
    assert_eq!(
        clocks[0].timestamp(),
        NaiveDate::from_ymd_opt(2016, 7, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    );
    assert!((clocks[0].bias_s() - 0.000_123_456_789).abs() < 1e-15);
}
