/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `TableScalar` text codec: `parse_cell(write_cell(x)) == x` at every supported
//! precision, the `Float106` pair encoding, the plain-decimal fallback, and malformed input.

use deep_causality_file::TableScalar;
use deep_causality_num::Float106;

fn encode<R: TableScalar>(x: R) -> String {
    let mut s = String::new();
    x.write_cell(&mut s);
    s
}

#[test]
fn f64_round_trips_bit_for_bit() {
    for &x in &[
        0.0_f64,
        1.0,
        -0.9,
        300_000.0,
        1.234_567_890_123_45e-17,
        f64::MAX,
    ] {
        let cell = encode(x);
        let back = f64::parse_cell(&cell).expect("parses");
        assert_eq!(back.to_bits(), x.to_bits(), "cell '{cell}' for {x}");
    }
}

#[test]
fn f32_round_trips_bit_for_bit() {
    for &x in &[0.0_f32, 1.0, -0.9, 12_345.678, 1.5e-20, f32::MAX] {
        let cell = encode(x);
        let back = f32::parse_cell(&cell).expect("parses");
        assert_eq!(back.to_bits(), x.to_bits(), "cell '{cell}' for {x}");
    }
}

#[test]
fn float106_pair_cell_round_trips_both_components() {
    // A value whose lo component is nonzero: only the pair encoding preserves it.
    let x = Float106::new(1.0, 0.0) / Float106::new(3.0, 0.0);
    let cell = encode(x);
    assert!(
        cell.contains('|'),
        "pair cell carries the separator: '{cell}'"
    );
    let back = Float106::parse_cell(&cell).expect("parses");
    assert_eq!(back.hi().to_bits(), x.hi().to_bits(), "hi component");
    assert_eq!(back.lo().to_bits(), x.lo().to_bits(), "lo component");
}

#[test]
fn float106_accepts_a_plain_decimal_literal() {
    // A hand-authored spec table carries plain decimals; they lift through exact f64.
    let back = Float106::parse_cell("0.9").expect("plain decimal parses");
    assert_eq!(back.hi().to_bits(), 0.9_f64.to_bits());
    assert_eq!(back.lo(), 0.0);
}

#[test]
fn malformed_cells_are_none() {
    assert!(f64::parse_cell("not-a-number").is_none());
    assert!(f32::parse_cell("").is_none());
    assert!(Float106::parse_cell("1.0|garbage").is_none());
    assert!(Float106::parse_cell("garbage").is_none());
}
