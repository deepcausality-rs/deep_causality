/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Phase-2 suite for `bin_equal_width` and `bin_equal_frequency`.
//!
//! Written before the implementation. Every expectation is either hand arithmetic over the
//! documented edge convention, written out in the comment above the assertion, an invariant the
//! answer must satisfy whatever the arithmetic, or a linear-scan oracle over bin edges that are
//! themselves written down as literals. No expectation is the implementation's formula retyped,
//! and none was recorded from a run.
//!
//! # The convention under test
//!
//! From `binning.rs`: "Both use the half-open convention `[lower, upper)` for every bin except the
//! last, which is closed so the maximum falls inside the range rather than one bin past its end."
//!
//! Read at a boundary that means: a value **equal to** an interior edge belongs to the bin **above**
//! it, and the maximum belongs to bin `bins - 1`.
//!
//! # Readings taken where the docs leave a gap
//!
//! 1. **More bins than observations is refused, in both functions.** `StatsErrorEnum::InvalidBinCount`
//!    is documented as "A bin count below two, **or above what the data can support**". The binning
//!    pair are the only functions in the crate that take a bin count, so that clause is about them;
//!    `n` observations cannot support more than `n` non-empty bins. `bins == n` is therefore the
//!    largest accepted count, and `bins == n + 1` is refused. A consequence is that a single
//!    observation is always refused, since every accepted count is at least two.
//! 2. **Empty data is `EmptyInput`, not `InvalidBinCount`.** Both readings are literally available
//!    for `(&[], 4)`. The enum's own doc splits its variants into "an input the mathematics does not
//!    admit (`EmptyInput`, …)" and "a shape the caller got wrong (…, `InvalidBinCount`)", and no
//!    observations is a statement about the data, so the empty check is taken to run first.
//! 3. **A constant column in `bin_equal_frequency` puts every observation in bin 0.** `bin_equal_width`
//!    documents that case ("every observation falls in bin zero"); `bin_equal_frequency` documents
//!    only that identical values cannot be split across a boundary, which forces one shared bin but
//!    does not name it. The equal-width convention is read across.
//! 4. **`bin_equal_frequency` is monotone**: bins are intervals of the sorted order, so a larger
//!    observation never lands in a lower bin. That is what makes the exactly-divisible case's index
//!    vector forced rather than merely counted.
//!
//! # Corner-case enumeration (`openspec/changes/unified-math-next/tdd/corner-cases.md`)
//!
//! | Row | Covered by |
//! |-----|------------|
//! | A empty input | `test_equal_width_empty_data_is_empty_input`, `test_equal_frequency_empty_data_is_empty_input` |
//! | B single element | `test_equal_width_single_observation_supports_no_bins`, `test_equal_frequency_single_observation_supports_no_bins`; the smallest accepted input (two observations, two bins) is in `test_equal_width_reaches_the_types_extremes` |
//! | C two quantities coincide | constant columns (`min == max`) in `test_equal_width_constant_column_is_bin_zero` and `test_equal_frequency_constant_column_is_bin_zero`; ties in `test_equal_frequency_ties_share_one_bin` and `test_equal_frequency_interior_tie_shares_one_bin` |
//! | D index degenerates | the maximum, where the unclamped quotient is exactly `bins` rather than `bins - 1`: `test_equal_width_bin_count_equal_to_observation_count_is_accepted` states the case where that quotient is exactly `4.0`; also the single interior edge of a two-bin split in `test_equal_width_two_bins_is_accepted` |
//! | E each threshold, both sides | every interior edge exactly (`test_equal_width_boundary_value_lands_in_the_upper_bin`) and a nudge either side of it (`test_equal_width_either_side_of_every_boundary`); the bin-count floor at one and two; the bin-count ceiling at `n` and `n + 1` |
//! | F zero | zero as the minimum of a range, and an all-zero constant column, in the constant-column and boundary tests |
//! | G negative | `test_equal_width_negative_range_boundaries`, the negative constant column, and the negative extreme in the reach tests |
//! | H exact domain boundary | the minimum, the maximum and each interior edge are the domain boundaries here. Probability 0 or 1 is n/a: binning takes arbitrary reals, not probabilities |
//! | I non-finite | `test_equal_width_non_finite_is_refused`, `test_equal_frequency_non_finite_is_refused` (NaN, +inf, −inf) |
//! | J overflow / underflow reach | `test_equal_width_reaches_the_types_extremes`, `test_equal_frequency_reaches_the_types_extremes`: a span to the type's largest finite value and a span down to its smallest positive normal, per precision. A span from −MAX to +MAX is deliberately **not** asserted: the range `max − min` overflows to infinity there and the docs do not say whether that is refused or computed in a wider intermediate, so the suite does not invent an answer |
//! | K f32, f64, Float106 | every test below calls its generic case three times |
//!
//! The per-precision quantities travel in [`Precision`]: the nudge that sits just inside a bin
//! boundary, and the two ends of the type's reach. Cases whose assertions are exact — an error
//! variant, or a bin index, which is a small whole number that all three precisions represent
//! exactly — take no such parameter, and say so.

use core::fmt::Debug;
// `RealField` carries `Real` (`floor`) and `ToPrimitive` (`to_usize`) as supertraits, so a bound on
// it brings both method sets to a generic parameter without naming them again here.
use deep_causality_algebra::RealField;
use deep_causality_num::lift;
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_stats::utils_tests::lift_array;
use deep_causality_stats::utils_tests::precision::{F32, F64, F106};
use deep_causality_stats::{StatsErrorEnum, bin_equal_frequency, bin_equal_width};

// ---------------------------------------------------------------------------------------------
// Per-precision constants
// ---------------------------------------------------------------------------------------------

/// A typed view of three rows of the shared precision table.
///
/// It holds no values of its own: each field is projected from `utils_tests::precision`. The
/// struct exists only so a generic test body can read them at `T` rather than at `f64`.
///
/// Note `huge` here is the type's LARGEST FINITE value — the shared table's `max_finite`, not its
/// `huge`, which means something else: finite, but with a square that is not.
struct Precision<T> {
    /// A step that must sit strictly inside a bin at the fixtures' magnitude (about 10) and well
    /// above the type's own rounding there. The spacing at 10 is roughly `1e-6` for `f32`,
    /// `2e-15` for `f64` and `1e-31` for `Float106`, so each nudge below is several orders of
    /// magnitude larger than its type's step and several orders smaller than the bin width of 2.
    nudge: T,
    /// The largest finite value of the type: the overflow reach for row J.
    huge: T,
    /// The smallest positive normal of the type: the underflow reach for row J.
    tiny: T,
}

fn precision_f32() -> Precision<f32> {
    Precision {
        nudge: lift(F32.nudge),
        huge: lift(F32.max_finite),
        tiny: lift(F32.min_positive),
    }
}

fn precision_f64() -> Precision<f64> {
    Precision {
        nudge: lift(F64.nudge),
        huge: lift(F64.max_finite),
        tiny: lift(F64.min_positive),
    }
}

fn precision_f106() -> Precision<Float106> {
    Precision {
        nudge: lift(F106.nudge),
        huge: lift(F106.max_finite),
        tiny: lift(F106.min_positive),
    }
}

// ---------------------------------------------------------------------------------------------
// Shared assertions
// ---------------------------------------------------------------------------------------------

/// Checks every returned entry is a whole number in `[0, bins)` and returns the per-bin counts.
///
/// The `floor` here is an integrality *predicate* — "this entry is a whole number" — not an index
/// computation; the indices themselves come from the function under test.
fn counts<T: RealField + FromPrimitive + Debug>(out: &[T], bins: usize) -> Vec<usize> {
    let zero = lift::<T>(0.0);
    let upper = lift::<T>(bins as f64);
    let mut tally = vec![0usize; bins];
    for &b in out {
        assert!(b >= zero, "a bin index is never negative, got {b:?}");
        assert!(b < upper, "a bin index is below {bins}, got {b:?}");
        assert_eq!(b, b.floor(), "a bin index is a whole number, got {b:?}");
        let k = b
            .to_usize()
            .expect("a whole number in [0, bins) converts to usize");
        tally[k] += 1;
    }
    tally
}

/// Sorted input must give non-decreasing indices.
///
/// Invariant, not a formula: bins are intervals of the ordering, so a larger observation cannot
/// land in a lower bin than a smaller one.
fn assert_non_decreasing<T: RealField + Debug>(out: &[T]) {
    for pair in out.windows(2) {
        assert!(
            pair[0] <= pair[1],
            "indices of sorted data are non-decreasing, got {:?} then {:?}",
            pair[0],
            pair[1]
        );
    }
}

/// A deliberately different algorithm for "which bin does `x` fall in".
///
/// The implementation under test divides by a width. This scans a list of edges that the caller
/// writes down as literals and returns the first bin whose upper edge `x` is strictly below,
/// falling through to the last bin, which is closed. Search rather than arithmetic, and the edges
/// are given rather than derived, so it shares no expression with the code under test.
fn scan_edges(x: f64, edges: &[f64]) -> f64 {
    let last = edges.len() - 2;
    for k in 0..last {
        if x < edges[k + 1] {
            return k as f64;
        }
    }
    last as f64
}

// ---------------------------------------------------------------------------------------------
// bin_equal_width — the edge convention
// ---------------------------------------------------------------------------------------------

/// Every interior edge value, plus both ends of the range.
///
/// Range `[0, 10]` into 5 bins. Width by hand: `(10 − 0) / 5 = 2`.
/// Edges: `0, 2, 4, 6, 8, 10`. Bins under the documented convention:
///   `[0,2) → 0`, `[2,4) → 1`, `[4,6) → 2`, `[6,8) → 3`, `[8,10] → 4`.
/// So each interior edge belongs to the bin above it, and 10 — the maximum — belongs to bin 4
/// rather than to a sixth bin.
fn case_width_boundary_lands_in_upper_bin<T: RealField + FromPrimitive + Debug>() {
    let data = lift_array::<T>(&[0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
    let got = bin_equal_width(&data, 5).expect("a finite range and five bins is a valid request");
    // Read off the interval table above, one entry per input value in input order.
    let want = lift_array::<T>(&[0.0, 1.0, 2.0, 3.0, 4.0, 4.0]);
    assert_eq!(got, want, "each edge belongs to the bin above it");
    assert_eq!(got.len(), data.len(), "one index per observation");
}

#[test]
fn test_equal_width_boundary_value_lands_in_the_upper_bin() {
    case_width_boundary_lands_in_upper_bin::<f32>();
    case_width_boundary_lands_in_upper_bin::<f64>();
    case_width_boundary_lands_in_upper_bin::<Float106>();
}

/// A nudge either side of every interior edge — row E, both sides of each threshold.
///
/// Same range and edges as above: `0, 2, 4, 6, 8, 10`, so `0.0` and `10.0` stay in the fixture to
/// hold the range fixed. Just below an edge is the lower bin, at or above it the upper one:
///   `2 − d → [0,2) → 0`, `2 + d → [2,4) → 1`,
///   `4 − d → [2,4) → 1`, `4 + d → [4,6) → 2`,
///   `6 − d → [4,6) → 2`, `6 + d → [6,8) → 3`,
///   `8 − d → [6,8) → 3`, `8 + d → [8,10] → 4`.
fn case_width_either_side_of_boundaries<T: RealField + FromPrimitive + Debug>(p: Precision<T>) {
    let d = p.nudge;
    let two = lift::<T>(2.0);
    let four = lift::<T>(4.0);
    let six = lift::<T>(6.0);
    let eight = lift::<T>(8.0);
    let data = vec![
        lift::<T>(0.0),
        lift::<T>(10.0),
        two - d,
        two + d,
        four - d,
        four + d,
        six - d,
        six + d,
        eight - d,
        eight + d,
    ];
    let got = bin_equal_width(&data, 5).expect("a finite range and five bins is a valid request");
    // Hand-read from the list above; the first two entries are the minimum and the maximum.
    let want = lift_array::<T>(&[0.0, 4.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0]);
    assert_eq!(got, want, "an edge is the first value of its own bin");
}

#[test]
fn test_equal_width_either_side_of_every_boundary() {
    case_width_either_side_of_boundaries::<f32>(precision_f32());
    case_width_either_side_of_boundaries::<f64>(precision_f64());
    case_width_either_side_of_boundaries::<Float106>(precision_f106());
}

/// A range that straddles zero, with a non-integer width — rows G and H.
///
/// Range `[−3, 7]` into 4 bins. Width by hand: `(7 − (−3)) / 4 = 10 / 4 = 2.5`.
/// Edges: `−3, −0.5, 2, 4.5, 7`. Bins:
///   `[−3,−0.5) → 0`, `[−0.5,2) → 1`, `[2,4.5) → 2`, `[4.5,7] → 3`.
/// The fixture holds the minimum, all three interior edges, one interior point either side of the
/// middle edge, and the maximum.
fn case_width_negative_range<T: RealField + FromPrimitive + Debug>() {
    let data = lift_array::<T>(&[-3.0, -0.5, 0.0, 2.0, 3.0, 4.5, 7.0]);
    let got = bin_equal_width(&data, 4).expect("a finite range and four bins is a valid request");
    // −3 is the minimum → 0; −0.5 is an edge → 1; 0 lies in [−0.5,2) → 1; 2 is an edge → 2;
    // 3 lies in [2,4.5) → 2; 4.5 is an edge → 3; 7 is the maximum → 3, the last bin.
    let want = lift_array::<T>(&[0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0]);
    assert_eq!(got, want);
}

#[test]
fn test_equal_width_negative_range_boundaries() {
    case_width_negative_range::<f32>();
    case_width_negative_range::<f64>();
    case_width_negative_range::<Float106>();
}

/// The minimum is in bin 0 and the maximum is in bin `bins − 1`, asserted directly.
///
/// Range `[1, 9]` into 4 bins. Width by hand: `(9 − 1) / 4 = 2`. Edges: `1, 3, 5, 7, 9`. Bins:
///   `[1,3) → 0`, `[3,5) → 1`, `[5,7) → 2`, `[7,9] → 3`.
fn case_width_min_first_max_last<T: RealField + FromPrimitive + Debug>() {
    let data = lift_array::<T>(&[1.0, 2.5, 3.75, 5.0, 6.25, 9.0]);
    let bins = 4;
    let got =
        bin_equal_width(&data, bins).expect("a finite range and four bins is a valid request");
    // 1 is the minimum → 0; 2.5 ∈ [1,3) → 0; 3.75 ∈ [3,5) → 1; 5 is an edge → 2;
    // 6.25 ∈ [5,7) → 2; 9 is the maximum → 3.
    let want = lift_array::<T>(&[0.0, 0.0, 1.0, 2.0, 2.0, 3.0]);
    assert_eq!(got, want);
    // The two claims the docs make in words, asserted on their own.
    assert_eq!(got[0], lift::<T>(0.0), "the minimum is in bin 0");
    assert_eq!(
        got[5],
        // bins − 1 = 4 − 1 = 3, written as the literal it is.
        lift::<T>(3.0),
        "the maximum is in the last bin, not one past the end"
    );
}

#[test]
fn test_equal_width_minimum_is_first_bin_and_maximum_is_last_bin() {
    case_width_min_first_max_last::<f32>();
    case_width_min_first_max_last::<f64>();
    case_width_min_first_max_last::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// bin_equal_width — degenerate range
// ---------------------------------------------------------------------------------------------

/// A constant column: zero range, no width to divide.
///
/// The doc is explicit — "It is not an error … and every observation falls in bin zero" — so the
/// expectation is a vector of zeros, one per observation, at a positive constant, at zero itself
/// (row F) and at a negative constant (row G).
fn case_width_constant_column<T: RealField + FromPrimitive + Debug>() {
    let bins = 3;
    for constant in [4.0, 0.0, -2.5] {
        let data = lift_array::<T>(&[constant, constant, constant, constant]);
        let got = bin_equal_width(&data, bins).expect("a constant column is not an error");
        let want = lift_array::<T>(&[0.0, 0.0, 0.0, 0.0]);
        assert_eq!(
            got, want,
            "every observation of a constant column is in bin 0"
        );
        assert_eq!(got.len(), data.len(), "one index per observation");
    }
}

#[test]
fn test_equal_width_constant_column_is_bin_zero() {
    case_width_constant_column::<f32>();
    case_width_constant_column::<f64>();
    case_width_constant_column::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// bin_equal_width — shape invariants
// ---------------------------------------------------------------------------------------------

/// Over a generated family: every index is a whole number in `[0, bins)`, the length is preserved,
/// sorted input gives non-decreasing indices, and the two ends sit in the two end bins.
///
/// Nothing here is a computed expectation — these are the properties any binning of `0, 1, …, 19`
/// into 6 bins must have, whatever the width works out to.
fn case_width_index_range_and_order<T: RealField + FromPrimitive + Debug>() {
    let values: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let data = lift_array::<T>(&values);
    let bins = 6;
    let got = bin_equal_width(&data, bins).expect("twenty observations support six bins");
    assert_eq!(got.len(), 20, "one index per observation");
    let tally = counts(&got, bins); // also asserts every index is whole and inside [0, 6)
    assert_eq!(
        tally.iter().sum::<usize>(),
        20,
        "every observation is placed"
    );
    assert_non_decreasing(&got);
    assert_eq!(got[0], lift::<T>(0.0), "the minimum is in bin 0");
    // bins − 1 = 6 − 1 = 5.
    assert_eq!(got[19], lift::<T>(5.0), "the maximum is in the last bin");
}

#[test]
fn test_equal_width_indices_are_in_range_and_ordered() {
    case_width_index_range_and_order::<f32>();
    case_width_index_range_and_order::<f64>();
    case_width_index_range_and_order::<Float106>();
}

/// Against an independent oracle: a linear scan over edges written down as literals.
///
/// The fixture's minimum is 0 and its maximum is 10, split into 5 bins, so the edges are
/// `0, 2, 4, 6, 8, 10` — written out below rather than computed. `scan_edges` then walks them and
/// returns the first bin whose upper edge the value is below, falling through to the closed last
/// bin. Every fixture value is at least 0.1 away from an edge unless it *is* one, so lifting the
/// literals into `f32` cannot move a value across a boundary.
fn case_width_matches_edge_scan<T: RealField + FromPrimitive + Debug>() {
    let values = [0.0, 0.5, 1.9, 2.0, 3.7, 4.0, 5.5, 6.0, 7.25, 8.0, 9.9, 10.0];
    let edges = [0.0, 2.0, 4.0, 6.0, 8.0, 10.0];
    let data = lift_array::<T>(&values);
    let got = bin_equal_width(&data, 5).expect("twelve observations support five bins");
    let want: Vec<f64> = values.iter().map(|&x| scan_edges(x, &edges)).collect();
    assert_eq!(
        got,
        lift_array::<T>(&want),
        "arithmetic binning agrees with an edge scan"
    );
}

#[test]
fn test_equal_width_agrees_with_an_independent_edge_scan() {
    case_width_matches_edge_scan::<f32>();
    case_width_matches_edge_scan::<f64>();
    case_width_matches_edge_scan::<Float106>();
}

/// Order is preserved: the index of an observation depends on the observation, not on where it sits
/// in the slice.
///
/// Range `[0, 10]` into 5 bins, so the edges are again `0, 2, 4, 6, 8, 10` and the bins
/// `[0,2) → 0`, `[2,4) → 1`, `[4,6) → 2`, `[6,8) → 3`, `[8,10] → 4`.
fn case_width_preserves_order<T: RealField + FromPrimitive + Debug>() {
    let shuffled = lift_array::<T>(&[10.0, 0.0, 6.0, 2.0, 4.0]);
    let got = bin_equal_width(&shuffled, 5).expect("five observations support five bins");
    // 10 is the maximum → 4; 0 is the minimum → 0; 6 is an edge → 3; 2 is an edge → 1;
    // 4 is an edge → 2. In the input's order.
    let want = lift_array::<T>(&[4.0, 0.0, 3.0, 1.0, 2.0]);
    assert_eq!(got, want, "the answer follows the input's order");

    // The same multiset in ascending order gets the same index attached to the same value.
    let sorted = lift_array::<T>(&[0.0, 2.0, 4.0, 6.0, 10.0]);
    let got_sorted = bin_equal_width(&sorted, 5).expect("five observations support five bins");
    let want_sorted = lift_array::<T>(&[0.0, 1.0, 2.0, 3.0, 4.0]);
    assert_eq!(got_sorted, want_sorted, "a permutation permutes the answer");
}

#[test]
fn test_equal_width_preserves_input_order() {
    case_width_preserves_order::<f32>();
    case_width_preserves_order::<f64>();
    case_width_preserves_order::<Float106>();
}

/// The output length equals the input length, at several sizes and bin counts.
fn case_width_length_is_preserved<T: RealField + FromPrimitive + Debug>() {
    for n in [2usize, 5, 6, 7] {
        for bins in 2..=n {
            let values: Vec<f64> = (0..n).map(|i| i as f64 * 1.5).collect();
            let data = lift_array::<T>(&values);
            let got = bin_equal_width(&data, bins).expect("bins ≤ n is an accepted request");
            assert_eq!(
                got.len(),
                n,
                "one index per observation, at n={n} bins={bins}"
            );
            let tally = counts(&got, bins);
            assert_eq!(
                tally.iter().sum::<usize>(),
                n,
                "every observation is placed"
            );
        }
    }
}

#[test]
fn test_equal_width_output_length_equals_input_length() {
    case_width_length_is_preserved::<f32>();
    case_width_length_is_preserved::<f64>();
    case_width_length_is_preserved::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// bin_equal_width — refused inputs
// ---------------------------------------------------------------------------------------------

/// Fewer than two bins is refused. Row E on the bin-count floor: 0 and 1 are refused, 2 is not.
///
/// "Fewer than two bins is refused: one bin is not a discretisation, and zero is not a partition."
fn case_width_bin_count_floor<T: RealField + FromPrimitive + Debug>() {
    let data = lift_array::<T>(&[0.0, 1.0, 2.0, 3.0]);
    for bins in [0usize, 1] {
        let err = bin_equal_width(&data, bins).expect_err("fewer than two bins is refused");
        assert!(
            matches!(err.0, StatsErrorEnum::InvalidBinCount(_)),
            "bins={bins} is an InvalidBinCount, got {err:?}"
        );
    }
}

#[test]
fn test_equal_width_fewer_than_two_bins_is_invalid_bin_count() {
    case_width_bin_count_floor::<f32>();
    case_width_bin_count_floor::<f64>();
    case_width_bin_count_floor::<Float106>();
}

/// Two bins — the other side of the floor — is accepted, and the single interior edge behaves.
///
/// Range `[0, 3]` into 2 bins. Width by hand: `(3 − 0) / 2 = 1.5`. Edges: `0, 1.5, 3`. Bins:
///   `[0,1.5) → 0`, `[1.5,3] → 1`.
fn case_width_two_bins_accepted<T: RealField + FromPrimitive + Debug>() {
    let data = lift_array::<T>(&[0.0, 1.0, 2.0, 3.0]);
    let got = bin_equal_width(&data, 2).expect("two bins is the smallest discretisation");
    // 0 → 0; 1 ∈ [0,1.5) → 0; 2 ∈ [1.5,3) → 1; 3 is the maximum → 1.
    let want = lift_array::<T>(&[0.0, 0.0, 1.0, 1.0]);
    assert_eq!(got, want);
}

#[test]
fn test_equal_width_two_bins_is_accepted() {
    case_width_two_bins_accepted::<f32>();
    case_width_two_bins_accepted::<f64>();
    case_width_two_bins_accepted::<Float106>();
}

/// More bins than observations is refused; exactly as many as observations is not. Row E on the
/// bin-count ceiling, and reading 1 of the header.
///
/// Range `[0, 3]` into 4 bins. Width by hand: `(3 − 0) / 4 = 0.75`. Edges: `0, 0.75, 1.5, 2.25, 3`.
/// Bins: `[0,0.75) → 0`, `[0.75,1.5) → 1`, `[1.5,2.25) → 2`, `[2.25,3] → 3`.
///
/// This fixture is also row D: at the maximum the unclamped quotient is exactly `3 / 0.75 = 4`,
/// which is `bins`, one past the last index. The closed last bin is what pulls it back to 3.
fn case_width_bin_count_ceiling<T: RealField + FromPrimitive + Debug>() {
    let data = lift_array::<T>(&[0.0, 1.0, 2.0, 3.0]);
    let got = bin_equal_width(&data, 4).expect("four observations support four bins");
    // 0 → 0; 1 ∈ [0.75,1.5) → 1; 2 ∈ [1.5,2.25) → 2; 3 is the maximum → 3.
    let want = lift_array::<T>(&[0.0, 1.0, 2.0, 3.0]);
    assert_eq!(got, want);

    let err = bin_equal_width(&data, 5).expect_err("four observations cannot support five bins");
    assert!(
        matches!(err.0, StatsErrorEnum::InvalidBinCount(_)),
        "a bin count above what the data can support is an InvalidBinCount, got {err:?}"
    );
}

#[test]
fn test_equal_width_bin_count_equal_to_observation_count_is_accepted() {
    case_width_bin_count_ceiling::<f32>();
    case_width_bin_count_ceiling::<f64>();
    case_width_bin_count_ceiling::<Float106>();
}

/// A single observation supports no discretisation: every accepted bin count is at least two, and
/// two is already more bins than the data can support. Row B.
fn case_width_single_observation<T: RealField + FromPrimitive + Debug>() {
    let data = lift_array::<T>(&[7.0]);
    let err = bin_equal_width(&data, 2).expect_err("one observation cannot support two bins");
    assert!(
        matches!(err.0, StatsErrorEnum::InvalidBinCount(_)),
        "one observation against two bins is an InvalidBinCount, got {err:?}"
    );
}

#[test]
fn test_equal_width_single_observation_supports_no_bins() {
    case_width_single_observation::<f32>();
    case_width_single_observation::<f64>();
    case_width_single_observation::<Float106>();
}

/// No observations at all. Row A, and reading 2 of the header.
fn case_width_empty<T: RealField + FromPrimitive + Debug>() {
    let data: Vec<T> = lift_array::<T>(&[]);
    let err = bin_equal_width(&data, 4).expect_err("no observations is refused");
    assert!(
        matches!(err.0, StatsErrorEnum::EmptyInput(_)),
        "an empty column is an EmptyInput, got {err:?}"
    );
}

#[test]
fn test_equal_width_empty_data_is_empty_input() {
    case_width_empty::<f32>();
    case_width_empty::<f64>();
    case_width_empty::<Float106>();
}

/// A non-finite observation "has no place on the range" and is refused. Row I.
fn case_width_non_finite<T: RealField + FromPrimitive + Debug>() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let data = lift_array::<T>(&[0.0, bad, 10.0]);
        let err = bin_equal_width(&data, 2).expect_err("a non-finite observation is refused");
        assert!(
            matches!(err.0, StatsErrorEnum::NonFiniteInput(_)),
            "a non-finite observation is a NonFiniteInput, got {err:?}"
        );
    }
}

#[test]
fn test_equal_width_non_finite_is_refused() {
    case_width_non_finite::<f32>();
    case_width_non_finite::<f64>();
    case_width_non_finite::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// bin_equal_width — the type's own extremes (row J)
// ---------------------------------------------------------------------------------------------

/// A span reaching the type's largest finite value, one reaching down to its smallest positive
/// normal, and one running from the negative extreme to zero.
///
/// Nothing here is a computed expectation. Two observations into two bins puts the minimum in bin 0
/// and the maximum in the closed last bin, index 1, whatever the width is; the point of the case is
/// that a width at the type's reach does not overflow to infinity, underflow to zero, or produce a
/// non-finite index. Each precision brings its own extremes, so `f32` is tested at `f32`'s reach
/// rather than at `f64`'s.
fn case_width_extremes<T: RealField + FromPrimitive + Debug>(p: Precision<T>) {
    let zero = lift::<T>(0.0);
    let spans = [[zero, p.huge], [zero, p.tiny], [-p.huge, zero]];
    for span in spans {
        let got = bin_equal_width(&span, 2).expect("two observations support two bins");
        assert_eq!(got.len(), 2, "one index per observation");
        assert_eq!(got[0], lift::<T>(0.0), "the minimum is in bin 0");
        // bins − 1 = 2 − 1 = 1.
        assert_eq!(got[1], lift::<T>(1.0), "the maximum is in the last bin");
    }
}

#[test]
fn test_equal_width_reaches_the_types_extremes() {
    case_width_extremes::<f32>(precision_f32());
    case_width_extremes::<f64>(precision_f64());
    case_width_extremes::<Float106>(precision_f106());
}

// ---------------------------------------------------------------------------------------------
// bin_equal_frequency — counts
// ---------------------------------------------------------------------------------------------

/// When the bin count divides the observation count, every bin holds exactly `n / k`.
///
/// Twelve distinct values into 3 bins: `12 / 3 = 4` by hand, so the counts are `4, 4, 4`. The bins
/// are intervals of the sorted order (reading 4), so on ascending input the index vector is forced:
/// the first four values are bin 0, the next four bin 1, the last four bin 2.
fn case_frequency_exact_division<T: RealField + FromPrimitive + Debug>() {
    let data = lift_array::<T>(&[
        10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0, 110.0, 120.0,
    ]);
    let bins = 3;
    let got = bin_equal_frequency(&data, bins).expect("twelve observations support three bins");
    assert_eq!(got.len(), 12, "one index per observation");
    let want = lift_array::<T>(&[0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 2.0]);
    assert_eq!(got, want, "four to a bin, in ascending order");
    assert_eq!(
        counts(&got, bins),
        vec![4, 4, 4],
        "12 / 3 = 4 observations in each bin"
    );
    assert_eq!(got[0], lift::<T>(0.0), "the minimum is in bin 0");
    // bins − 1 = 3 − 1 = 2.
    assert_eq!(got[11], lift::<T>(2.0), "the maximum is in the last bin");
}

#[test]
fn test_equal_frequency_divides_evenly_when_the_count_divides() {
    case_frequency_exact_division::<f32>();
    case_frequency_exact_division::<f64>();
    case_frequency_exact_division::<Float106>();
}

/// The same twelve values shuffled: each value keeps the index it had when sorted, in the input's
/// own order.
///
/// The per-value indices come from the case above — 10 to 40 in bin 0, 50 to 80 in bin 1, 90 to 120
/// in bin 2 — read off for this permutation by hand.
fn case_frequency_preserves_order<T: RealField + FromPrimitive + Debug>() {
    let data = lift_array::<T>(&[
        70.0, 10.0, 120.0, 50.0, 30.0, 90.0, 40.0, 110.0, 20.0, 80.0, 100.0, 60.0,
    ]);
    let bins = 3;
    let got = bin_equal_frequency(&data, bins).expect("twelve observations support three bins");
    // 70→1, 10→0, 120→2, 50→1, 30→0, 90→2, 40→0, 110→2, 20→0, 80→1, 100→2, 60→1.
    let want = lift_array::<T>(&[1.0, 0.0, 2.0, 1.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 2.0, 1.0]);
    assert_eq!(got, want, "the answer follows the input's order");
    assert_eq!(
        counts(&got, bins),
        vec![4, 4, 4],
        "a permutation does not change the counts"
    );
}

#[test]
fn test_equal_frequency_preserves_input_order() {
    case_frequency_preserves_order::<f32>();
    case_frequency_preserves_order::<f64>();
    case_frequency_preserves_order::<Float106>();
}

/// When the bin count does not divide the observation count, "the remainder is spread rather than
/// concentrated in one bin".
///
/// Ten distinct values into 3 bins: `10 = 3 + 3 + 4` by hand, so every count is 3 or 4 and no bin
/// takes the whole remainder. The invariant, not a chosen assignment: the largest count and the
/// smallest differ by at most one.
fn case_frequency_spreads_the_remainder<T: RealField + FromPrimitive + Debug>() {
    let data = lift_array::<T>(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);
    let bins = 3;
    let got = bin_equal_frequency(&data, bins).expect("ten observations support three bins");
    let tally = counts(&got, bins);
    assert_eq!(
        tally.iter().sum::<usize>(),
        10,
        "every observation is placed"
    );
    let smallest = *tally.iter().min().expect("three bins");
    let largest = *tally.iter().max().expect("three bins");
    assert!(
        largest - smallest <= 1,
        "the remainder is spread, not concentrated; counts were {tally:?}"
    );
    // 10 = 3 + 3 + 4, so every bin holds three or four and none is empty.
    for count in &tally {
        assert!(
            *count == 3 || *count == 4,
            "each of three bins holds three or four of ten observations, counts were {tally:?}"
        );
    }
}

#[test]
fn test_equal_frequency_spreads_the_remainder() {
    case_frequency_spreads_the_remainder::<f32>();
    case_frequency_spreads_the_remainder::<f64>();
    case_frequency_spreads_the_remainder::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// bin_equal_frequency — ties
// ---------------------------------------------------------------------------------------------

/// Tied values cannot be split across a boundary, so the bins come out uneven. Row C.
///
/// Six copies of the minimum and four larger distinct values, ten observations into 5 bins. An even
/// split would be two per bin; the tie makes that impossible. What is asserted is what the doc
/// forces: the six tied values share one index, and — since they are the smallest and bins are
/// intervals of the sorted order — that index is 0. The bin holding them therefore has six
/// observations where an even split would give two.
///
/// Where the four remaining values land is deliberately not pinned: with four observations left for
/// four bins the doc does not say how the spread lands, and inventing an answer here would test the
/// implementation's choice rather than its contract.
fn case_frequency_ties<T: RealField + FromPrimitive + Debug>() {
    let data = lift_array::<T>(&[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
    let bins = 5;
    let got = bin_equal_frequency(&data, bins).expect("ten observations support five bins");
    assert_eq!(got.len(), 10, "one index per observation");
    let tally = counts(&got, bins);
    assert_eq!(
        tally.iter().sum::<usize>(),
        10,
        "every observation is placed"
    );

    let tied = &got[0..6];
    for (i, index) in tied.iter().enumerate() {
        assert_eq!(
            *index, got[0],
            "identical values share a bin; copy {i} went elsewhere"
        );
    }
    assert_eq!(
        got[0],
        lift::<T>(0.0),
        "the tie sits at the minimum, so its shared bin is bin 0"
    );
    assert_eq!(
        tally[0], 6,
        "the six tied observations are all in bin 0, so it is over-full by necessity"
    );
    assert_non_decreasing(&got);
}

#[test]
fn test_equal_frequency_ties_share_one_bin() {
    case_frequency_ties::<f32>();
    case_frequency_ties::<f64>();
    case_frequency_ties::<Float106>();
}

/// A tie in the middle rather than at an end. Row C again, with the tie away from bin 0.
///
/// Six observations into 3 bins, four of them identical. An even split would be two per bin; the
/// four copies must share one bin instead. The minimum is alone below the tie, so it is bin 0, and
/// the maximum sits at or above the tie's bin by monotonicity.
fn case_frequency_interior_tie<T: RealField + FromPrimitive + Debug>() {
    let data = lift_array::<T>(&[1.0, 2.0, 2.0, 2.0, 2.0, 3.0]);
    let bins = 3;
    let got = bin_equal_frequency(&data, bins).expect("six observations support three bins");
    assert_eq!(got.len(), 6, "one index per observation");
    let tally = counts(&got, bins);
    assert_eq!(
        tally.iter().sum::<usize>(),
        6,
        "every observation is placed"
    );
    for (i, index) in got[1..5].iter().enumerate() {
        assert_eq!(
            *index, got[1],
            "the four copies of 2 share a bin; copy {i} went elsewhere"
        );
    }
    assert_eq!(got[0], lift::<T>(0.0), "the minimum is in bin 0");
    assert!(
        got[5] >= got[1],
        "the maximum is not below the tie's bin: {:?} against {:?}",
        got[5],
        got[1]
    );
    assert_eq!(
        tally.iter().max().copied(),
        Some(4),
        "the tied bin holds all four copies, so the bins are uneven by necessity"
    );
    assert_non_decreasing(&got);
}

#[test]
fn test_equal_frequency_interior_tie_shares_one_bin() {
    case_frequency_interior_tie::<f32>();
    case_frequency_interior_tie::<f64>();
    case_frequency_interior_tie::<Float106>();
}

/// A constant column is the tie taken to its limit: every observation is identical, so nothing can
/// be split and every observation shares one bin. Reading 3 of the header names that bin 0.
fn case_frequency_constant_column<T: RealField + FromPrimitive + Debug>() {
    for constant in [4.0, 0.0, -2.5] {
        let data = lift_array::<T>(&[constant, constant, constant, constant]);
        let got = bin_equal_frequency(&data, 2).expect("a constant column is not an error");
        let want = lift_array::<T>(&[0.0, 0.0, 0.0, 0.0]);
        assert_eq!(got, want, "a constant column is one bin, and it is bin 0");
    }
}

#[test]
fn test_equal_frequency_constant_column_is_bin_zero() {
    case_frequency_constant_column::<f32>();
    case_frequency_constant_column::<f64>();
    case_frequency_constant_column::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// bin_equal_frequency — shape invariants and refused inputs
// ---------------------------------------------------------------------------------------------

/// Over a generated family: whole indices inside `[0, bins)`, length preserved, sorted input
/// non-decreasing, ends in the end bins, and the counts within one of each other.
///
/// Twenty distinct values into 6 bins: `20 = 6 × 3 + 2` by hand, so the counts are four 3s and two
/// 4s in some order — every count is 3 or 4.
fn case_frequency_index_range_and_order<T: RealField + FromPrimitive + Debug>() {
    let values: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let data = lift_array::<T>(&values);
    let bins = 6;
    let got = bin_equal_frequency(&data, bins).expect("twenty observations support six bins");
    assert_eq!(got.len(), 20, "one index per observation");
    let tally = counts(&got, bins);
    assert_eq!(
        tally.iter().sum::<usize>(),
        20,
        "every observation is placed"
    );
    for count in &tally {
        assert!(
            *count == 3 || *count == 4,
            "20 = 6 × 3 + 2, so each bin holds three or four; counts were {tally:?}"
        );
    }
    assert_non_decreasing(&got);
    assert_eq!(got[0], lift::<T>(0.0), "the minimum is in bin 0");
    // bins − 1 = 6 − 1 = 5.
    assert_eq!(got[19], lift::<T>(5.0), "the maximum is in the last bin");
}

#[test]
fn test_equal_frequency_indices_are_in_range_and_ordered() {
    case_frequency_index_range_and_order::<f32>();
    case_frequency_index_range_and_order::<f64>();
    case_frequency_index_range_and_order::<Float106>();
}

/// Fewer than two bins is refused; two is accepted. Row E on the bin-count floor.
///
/// With four distinct values and 2 bins, `4 / 2 = 2` by hand, so the ascending input splits
/// `0, 0, 1, 1`.
fn case_frequency_bin_count_floor<T: RealField + FromPrimitive + Debug>() {
    let data = lift_array::<T>(&[1.0, 2.0, 3.0, 4.0]);
    for bins in [0usize, 1] {
        let err = bin_equal_frequency(&data, bins).expect_err("fewer than two bins is refused");
        assert!(
            matches!(err.0, StatsErrorEnum::InvalidBinCount(_)),
            "bins={bins} is an InvalidBinCount, got {err:?}"
        );
    }
    let got = bin_equal_frequency(&data, 2).expect("two bins is the smallest discretisation");
    let want = lift_array::<T>(&[0.0, 0.0, 1.0, 1.0]);
    assert_eq!(got, want, "two to a bin, in ascending order");
}

#[test]
fn test_equal_frequency_fewer_than_two_bins_is_invalid_bin_count() {
    case_frequency_bin_count_floor::<f32>();
    case_frequency_bin_count_floor::<f64>();
    case_frequency_bin_count_floor::<Float106>();
}

/// More bins than observations is refused; exactly as many is not. Row E on the bin-count ceiling.
///
/// Four distinct values into 4 bins: `4 / 4 = 1` by hand, so each bin holds exactly one and the
/// ascending input gives `0, 1, 2, 3`. Five bins have no fourth observation to fill them.
fn case_frequency_bin_count_ceiling<T: RealField + FromPrimitive + Debug>() {
    let data = lift_array::<T>(&[1.0, 2.0, 3.0, 4.0]);
    let got = bin_equal_frequency(&data, 4).expect("four observations support four bins");
    let want = lift_array::<T>(&[0.0, 1.0, 2.0, 3.0]);
    assert_eq!(got, want, "one to a bin, in ascending order");
    assert_eq!(counts(&got, 4), vec![1, 1, 1, 1], "4 / 4 = 1 in each bin");

    let err =
        bin_equal_frequency(&data, 5).expect_err("four observations cannot support five bins");
    assert!(
        matches!(err.0, StatsErrorEnum::InvalidBinCount(_)),
        "a bin count above what the data can support is an InvalidBinCount, got {err:?}"
    );
}

#[test]
fn test_equal_frequency_bin_count_equal_to_observation_count_is_accepted() {
    case_frequency_bin_count_ceiling::<f32>();
    case_frequency_bin_count_ceiling::<f64>();
    case_frequency_bin_count_ceiling::<Float106>();
}

/// A single observation supports no discretisation. Row B.
fn case_frequency_single_observation<T: RealField + FromPrimitive + Debug>() {
    let data = lift_array::<T>(&[7.0]);
    let err = bin_equal_frequency(&data, 2).expect_err("one observation cannot support two bins");
    assert!(
        matches!(err.0, StatsErrorEnum::InvalidBinCount(_)),
        "one observation against two bins is an InvalidBinCount, got {err:?}"
    );
}

#[test]
fn test_equal_frequency_single_observation_supports_no_bins() {
    case_frequency_single_observation::<f32>();
    case_frequency_single_observation::<f64>();
    case_frequency_single_observation::<Float106>();
}

/// No observations at all. Row A, and reading 2 of the header.
fn case_frequency_empty<T: RealField + FromPrimitive + Debug>() {
    let data: Vec<T> = lift_array::<T>(&[]);
    let err = bin_equal_frequency(&data, 4).expect_err("no observations is refused");
    assert!(
        matches!(err.0, StatsErrorEnum::EmptyInput(_)),
        "an empty column is an EmptyInput, got {err:?}"
    );
}

#[test]
fn test_equal_frequency_empty_data_is_empty_input() {
    case_frequency_empty::<f32>();
    case_frequency_empty::<f64>();
    case_frequency_empty::<Float106>();
}

/// A non-finite observation has no place in the order the bins are cut from. Row I.
fn case_frequency_non_finite<T: RealField + FromPrimitive + Debug>() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let data = lift_array::<T>(&[0.0, bad, 10.0]);
        let err = bin_equal_frequency(&data, 2).expect_err("a non-finite observation is refused");
        assert!(
            matches!(err.0, StatsErrorEnum::NonFiniteInput(_)),
            "a non-finite observation is a NonFiniteInput, got {err:?}"
        );
    }
}

#[test]
fn test_equal_frequency_non_finite_is_refused() {
    case_frequency_non_finite::<f32>();
    case_frequency_non_finite::<f64>();
    case_frequency_non_finite::<Float106>();
}

/// The type's own extremes. Row J, as for equal width.
///
/// Two observations into two bins: one each, so the smaller is bin 0 and the larger bin 1, whatever
/// the magnitudes. The case is that a rank cut at the type's reach still returns finite indices.
fn case_frequency_extremes<T: RealField + FromPrimitive + Debug>(p: Precision<T>) {
    let zero = lift::<T>(0.0);
    let spans = [[zero, p.huge], [zero, p.tiny], [-p.huge, zero]];
    for span in spans {
        let got = bin_equal_frequency(&span, 2).expect("two observations support two bins");
        assert_eq!(got.len(), 2, "one index per observation");
        assert_eq!(
            got[0],
            lift::<T>(0.0),
            "the smaller observation is in bin 0"
        );
        // bins − 1 = 2 − 1 = 1.
        assert_eq!(
            got[1],
            lift::<T>(1.0),
            "the larger observation is in the last bin"
        );
    }
}

#[test]
fn test_equal_frequency_reaches_the_types_extremes() {
    case_frequency_extremes::<f32>(precision_f32());
    case_frequency_extremes::<f64>(precision_f64());
    case_frequency_extremes::<Float106>(precision_f106());
}
