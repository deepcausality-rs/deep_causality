/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The categorical distribution, checked against its weights.

use deep_causality_rand::{Distribution, Xoshiro256};
use deep_causality_stats::Categorical;
use deep_causality_stats::utils_tests::sampling::{
    MaxRng, SUITE_SEED, ZeroRng, assert_near, expect_accepted, expect_refused,
};

const N: usize = 200_000;

/// Observed frequency of each index over `N` draws.
fn frequencies<T>(d: &Categorical<T>, n: usize, seed: u64) -> Vec<f64>
where
    Categorical<T>: Distribution<usize>,
{
    let mut g = Xoshiro256::from_seed(seed);
    let mut counts = vec![0u64; d.len()];
    for _ in 0..n {
        let i: usize = d.sample(&mut g);
        assert!(i < counts.len(), "index {i} is outside 0..{}", counts.len());
        counts[i] += 1;
    }
    counts.into_iter().map(|c| c as f64 / n as f64).collect()
}

// ---------------------------------------------------------------------------------------------
// The weights
// ---------------------------------------------------------------------------------------------

#[test]
fn the_frequencies_match_the_normalised_weights() {
    let d = expect_accepted(Categorical::new(vec![1.0f64, 3.0, 6.0]), "weights 1:3:6");
    let f = frequencies(&d, N, SUITE_SEED);
    assert_near(f[0], 0.1, 0.05, "index 0");
    assert_near(f[1], 0.3, 0.05, "index 1");
    assert_near(f[2], 0.6, 0.05, "index 2");
}

#[test]
fn unnormalised_weights_agree_with_their_normalised_form() {
    // The sampler normalises; the caller does not have to. A sampler that forgets is correct
    // whenever the caller happened to normalise, so both forms are compared against each other.
    let raw = expect_accepted(Categorical::new(vec![1.0f64, 3.0, 6.0]), "raw");
    let normalised = expect_accepted(Categorical::new(vec![0.1f64, 0.3, 0.6]), "normalised");

    let fr = frequencies(&raw, N, SUITE_SEED);
    let fnorm = frequencies(&normalised, N, SUITE_SEED);
    for i in 0..3 {
        assert_near(fr[i], fnorm[i], 0.05, "raw against normalised");
    }
}

#[test]
fn weights_that_sum_far_from_one_still_work() {
    // Totals well away from 1 in both directions, which a missing division gets wrong in opposite
    // ways: a large total starves the later indices, a small one overflows into the fallback.
    for weights in [vec![100.0f64, 300.0, 600.0], vec![0.001f64, 0.003, 0.006]] {
        let d = expect_accepted(Categorical::new(weights.clone()), "scaled weights");
        let f = frequencies(&d, 100_000, SUITE_SEED);
        assert_near(f[0], 0.1, 0.05, "index 0");
        assert_near(f[2], 0.6, 0.05, "index 2");
    }
}

// ---------------------------------------------------------------------------------------------
// Structure
// ---------------------------------------------------------------------------------------------

#[test]
fn a_single_category_always_wins() {
    let d = expect_accepted(Categorical::new(vec![7.0f64]), "one category");
    let f = frequencies(&d, 1_000, SUITE_SEED);
    assert_eq!(f[0], 1.0, "the only index must be drawn every time");
}

#[test]
fn a_zero_weight_is_never_drawn() {
    // A cumulative scan comparing with `<=` rather than `<` can select a zero-weight category,
    // which no frequency tolerance would notice at a low enough weight.
    let d = expect_accepted(
        Categorical::new(vec![1.0f64, 0.0, 1.0]),
        "a zero in the middle",
    );
    let f = frequencies(&d, 100_000, SUITE_SEED);
    assert_eq!(f[1], 0.0, "a zero-weight category was drawn");
    assert_near(f[0], 0.5, 0.05, "index 0");
    assert_near(f[2], 0.5, 0.05, "index 2");
}

#[test]
fn every_index_returned_is_in_range() {
    // An off-by-one returning `i + 1` would step past the last index. `frequencies` asserts this
    // on every draw; this test names it so the reason is visible.
    let d = expect_accepted(Categorical::new(vec![1.0f64; 8]), "eight equal weights");
    let f = frequencies(&d, 80_000, SUITE_SEED);
    assert_eq!(f.len(), 8);
    for (i, p) in f.iter().enumerate() {
        assert_near(*p, 0.125, 0.1, &format!("index {i}"));
    }
}

#[test]
fn the_last_category_is_reachable() {
    // Floating-point drift can leave the final comparison marginal, so the last index is the one
    // a scan is most likely to lose. With eight equal weights it should take an eighth.
    let d = expect_accepted(Categorical::new(vec![1.0f64; 8]), "eight equal weights");
    let f = frequencies(&d, 80_000, SUITE_SEED);
    assert!(f[7] > 0.05, "the last index took only {} of draws", f[7]);
}

// ---------------------------------------------------------------------------------------------
// Parameters outside the support
// ---------------------------------------------------------------------------------------------

#[test]
fn a_degenerate_weight_vector_is_refused() {
    expect_refused(Categorical::<f64>::new(vec![]), "no weights");
    expect_refused(Categorical::new(vec![1.0f64, -1.0]), "a negative weight");
    expect_refused(
        Categorical::new(vec![0.0f64, 0.0]),
        "weights summing to zero",
    );
    expect_refused(Categorical::new(vec![1.0f64, f64::NAN]), "a NaN weight");
    expect_refused(
        Categorical::new(vec![1.0f64, f64::INFINITY]),
        "an infinite weight",
    );
}

#[test]
fn the_weights_are_kept_verbatim() {
    let d = expect_accepted(Categorical::new(vec![1.0f64, 3.0, 6.0]), "weights");
    assert_eq!(
        d.weights(),
        &[1.0, 3.0, 6.0],
        "the constructor altered the weights"
    );
    assert_eq!(d.len(), 3);
    assert!(!d.is_empty());
}

// ---------------------------------------------------------------------------------------------
// The boundaries a random draw will not reach
// ---------------------------------------------------------------------------------------------

#[test]
fn a_leading_zero_weight_is_not_drawn_at_the_lower_boundary() {
    // The cumulative scan compares the remainder against each weight. With `<=` rather than `<`, a
    // zero-weight category wins whenever the remainder is exactly zero — which happens on a real
    // generator at probability `2^-53`, so `a_zero_weight_is_never_drawn` above passes against the
    // defective comparison however many draws it takes.
    //
    // A zero draw reaches it immediately: the remainder starts at zero, and the first weight is
    // zero, so `<` moves past it and `<=` selects it.
    let d = expect_accepted(Categorical::new(vec![0.0f64, 1.0]), "a leading zero weight");
    let i: usize = d.sample(&mut ZeroRng);
    assert_eq!(
        i, 1,
        "the zero-weight category was selected at the lower boundary"
    );
}

#[test]
fn the_draw_stays_in_range_at_the_upper_boundary() {
    // A draw as close to 1 as the scalar allows, at five equal weights. The last comparison still
    // fires here — `1 - 2^-53` times five, less four, is below one — so this checks the ordinary
    // upper boundary rather than the fallback. The fallback has its own test below, because
    // reaching it takes more than a large draw.
    let d = expect_accepted(Categorical::new(vec![1.0f64; 5]), "five equal weights");
    let i: usize = d.sample(&mut MaxRng);
    assert!(i < 5, "the draw returned index {i} for five categories");
}

#[test]
fn the_rounding_fallback_returns_the_last_category() {
    // The fallback exists for accumulated rounding, where the remainder outlives the weights. In
    // exact arithmetic it cannot be reached: `u * total` less the first `n - 1` weights is
    // `w_last - (1 - u) * total`, which is below `w_last`. It is reached when the weights sum to
    // slightly more than they should and the draw sits close enough to one.
    //
    // `0.3 + 0.7` is such a sum. Neither is representable, and their f64 sum rounds **up** to
    // exactly 1.0, so a draw of `1 - 2^-53` gives a remainder of `1 - 2^-53`; subtracting 0.3
    // leaves exactly 0.7, which is not *less than* 0.7, and the scan runs off the end.
    //
    // Two weights, no loop, and nothing about the case is contrived — a caller writing
    // `[0.3, 0.7]` is writing the most ordinary pair of probabilities there is. A mutation to
    // `weights.len() - 1` hands that caller index 2 or 3 for two categories, which is a panic in
    // whatever array they index with it.
    let d = expect_accepted(
        Categorical::new(vec![0.3f64, 0.7]),
        "two ordinary probabilities",
    );
    let i: usize = d.sample(&mut MaxRng);
    assert_eq!(
        i, 1,
        "the rounding fallback returned {i}, which is not the last of two categories"
    );
}
