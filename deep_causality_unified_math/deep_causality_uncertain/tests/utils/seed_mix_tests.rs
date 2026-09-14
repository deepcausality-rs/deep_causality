/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The draw address, pinned against an independent oracle.
//!
//! Every literal below was produced by a second implementation of the construction written in
//! Python — `fmix64`, the golden multiplier and the two rounds, transcribed from the same stated
//! sources rather than from this crate's code. Two implementations agreeing is the verification;
//! recording what the Rust returns would only pin today's behaviour to itself.

use deep_causality_uncertain::draw_seed;

/// Corner rows A–K: `(row, seed, index, ordinal, expected)`.
///
/// A–D take each argument to zero in turn, E–H take each to `u64::MAX` and then all three, and
/// I–K transpose each pair against the reference `(9, 7, 3)`.
const ROWS: [(&str, u64, u64, u64, u64); 11] = [
    ("A", 0, 0, 0, 0x79CF_C3DB_E5BB_FCAF),
    ("B", 0, 7, 3, 0x24F2_8A39_B0AC_369C),
    ("C", 9, 0, 3, 0xA25C_5F34_E892_EE64),
    ("D", 9, 7, 0, 0xD6C4_65DC_CBBA_5669),
    ("E", u64::MAX, 7, 3, 0xA575_366B_B56C_574C),
    ("F", 9, u64::MAX, 3, 0x8421_A5DF_2096_9DCA),
    ("G", 9, 7, u64::MAX, 0xE896_352B_93DF_3A97),
    ("H", u64::MAX, u64::MAX, u64::MAX, 0xCC78_3797_3B36_F35B),
    ("I", 7, 9, 3, 0x8DEA_62B7_2611_A89E),
    ("J", 3, 7, 9, 0x4CFE_0345_D6C4_6D5C),
    ("K", 9, 3, 7, 0x0E24_5C26_72B8_158D),
];

/// The row every transposition is measured against.
const REFERENCE: (u64, u64, u64, u64) = (9, 7, 3, 0x7B4F_A586_B1D4_1B7B);

#[test]
fn every_corner_row_matches_the_oracle() {
    for (row, seed, index, ordinal, expected) in ROWS {
        assert_eq!(
            draw_seed(seed, index, ordinal),
            expected,
            "row {row}: draw_seed({seed}, {index}, {ordinal})"
        );
    }
    let (s, i, o, expected) = REFERENCE;
    assert_eq!(draw_seed(s, i, o), expected, "reference row");
}

#[test]
fn transposing_any_two_arguments_changes_the_address() {
    // All three are u64, so a call site that transposes them still compiles. If any two entered
    // the construction symmetrically, the transposed call would sample a valid but wrong stream
    // and nothing would say so.
    let (s, i, o, reference) = REFERENCE;

    assert_ne!(draw_seed(i, s, o), reference, "seed and index interchange");
    assert_ne!(
        draw_seed(o, i, s),
        reference,
        "seed and ordinal interchange"
    );
    assert_ne!(
        draw_seed(s, o, i),
        reference,
        "index and ordinal interchange"
    );
}

#[test]
fn every_corner_row_is_distinct() {
    for (a, row_a) in ROWS.iter().enumerate() {
        for row_b in ROWS.iter().skip(a + 1) {
            assert_ne!(row_a.4, row_b.4, "rows {} and {} collide", row_a.0, row_b.0);
        }
    }
}

#[test]
fn the_all_zero_address_is_not_zero() {
    // `fmix64` fixes zero, so a construction without the additive step would return zero here.
    // Harmless one layer down, where the generator expands any seed, but a hash that maps the
    // origin to the origin is the kind of thing that is wrong in a second use.
    assert_ne!(draw_seed(0, 0, 0), 0);
}

#[test]
fn one_bit_of_any_argument_moves_about_half_the_output() {
    // A mixer that failed to avalanche would give neighbouring ordinals correlated streams, which
    // is invisible in any single draw and shows up as structure across a whole ensemble.
    let base = (123_456_789_u64, 42_u64, 7_u64);
    let reference = draw_seed(base.0, base.1, base.2);

    for position in 0..3 {
        let mut total = 0u32;
        let mut lowest = 64u32;

        for bit in 0..64 {
            let flip = 1u64 << bit;
            let moved = match position {
                0 => draw_seed(base.0 ^ flip, base.1, base.2),
                1 => draw_seed(base.0, base.1 ^ flip, base.2),
                _ => draw_seed(base.0, base.1, base.2 ^ flip),
            };
            let changed = (moved ^ reference).count_ones();
            total += changed;
            lowest = lowest.min(changed);
        }

        let mean = f64::from(total) / 64.0;
        assert!(
            (24.0..=40.0).contains(&mean),
            "argument {position}: mean bit change {mean}, expected near 32 of 64"
        );
        assert!(
            lowest >= 12,
            "argument {position}: one input bit moved only {lowest} output bits"
        );
    }
}

#[test]
fn consecutive_ordinals_do_not_share_an_address() {
    // The ordinals in a tree are 0, 1, 2, ... so adjacency is the common case rather than a corner.
    let addresses: Vec<u64> = (0..1024).map(|ordinal| draw_seed(7, 11, ordinal)).collect();

    let mut sorted = addresses.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        addresses.len(),
        "two ordinals share an address"
    );
}

#[test]
fn consecutive_indices_do_not_share_an_address() {
    let addresses: Vec<u64> = (0..1024).map(|index| draw_seed(7, index, 11)).collect();

    let mut sorted = addresses.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        addresses.len(),
        "two indices share an address"
    );
}
