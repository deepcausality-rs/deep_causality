/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The streaming mean.
//!
//! The property that matters is not that it computes *a* mean but that it computes the *same* one
//! as [`mean`] over the same observations in the same order — otherwise a caller that switches to
//! it for the memory silently changes its answer.

use deep_causality_num::{BFloat16, Float106, lift};
use deep_causality_stats::utils_tests::lift_array;
use deep_causality_stats::{MeanAccumulator, StatsErrorEnum, mean};

/// Bit-for-bit agreement with the slice form, on a sample whose sum rounds.
///
/// `0.1` and `0.2` are not dyadic, so the running sum rounds at almost every step; if the
/// accumulator summed in a different arrangement the two answers would part. It does not — both
/// sum the same balanced tree, because which partial sum lands in which slot is decided by the
/// count, which both callers share — so the assertion is exact equality rather than a tolerance.
#[test]
fn test_accumulator_agrees_with_the_slice_mean_bit_for_bit() {
    for sample in [
        vec![0.1_f64, 0.2, 0.3, 0.4, 0.5],
        vec![1e16, 1.0, -1e16],
        vec![-3.0, 1.0, 5.0],
        vec![7.0; 9],
    ] {
        let mut acc = MeanAccumulator::<f64>::new();
        for &x in &sample {
            acc.push(x);
        }
        assert_eq!(
            acc.mean().unwrap(),
            mean(&sample).unwrap(),
            "streaming and slice means must agree exactly on {sample:?}"
        );
    }
}

/// Agreement where a differing arrangement would actually show: a thousand observations at
/// `BFloat16`, whose count is not a power of two.
///
/// Two observations agree under any arrangement, and a power-of-two count agrees under both a
/// midpoint split and a carry tree. 1000 is neither, and at two decimal digits a single differing
/// association is visible in the result — so this is the case that pins the two callers to one
/// arrangement rather than merely to one answer.
#[test]
fn test_accumulator_agrees_over_a_thousand_observations_at_the_narrowest_scalar() {
    let sample: Vec<BFloat16> = (1..=1000).map(|i| lift::<BFloat16>(i as f64)).collect();
    let mut acc = MeanAccumulator::<BFloat16>::new();
    for &x in &sample {
        acc.push(x);
    }
    assert_eq!(
        acc.mean().unwrap(),
        mean(&sample).unwrap(),
        "streaming and slice means must agree exactly over a thousand observations"
    );
}

/// The same agreement at the widest and narrowest shipped scalars.
#[test]
fn test_accumulator_agrees_at_every_precision() {
    let raw = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];

    let f32s = lift_array::<f32>(&raw);
    let mut a32 = MeanAccumulator::<f32>::new();
    for &x in &f32s {
        a32.push(x);
    }
    assert_eq!(a32.mean().unwrap(), mean(&f32s).unwrap());

    let wide = lift_array::<Float106>(&raw);
    let mut a106 = MeanAccumulator::<Float106>::new();
    for &x in &wide {
        a106.push(x);
    }
    assert_eq!(a106.mean().unwrap(), mean(&wide).unwrap());
}

/// An empty accumulator refuses, exactly as the slice form does — a caller wanting zero there says
/// so itself rather than having the statistic invent it.
#[test]
fn test_accumulator_refuses_before_anything_is_pushed() {
    let acc = MeanAccumulator::<f64>::new();
    assert_eq!(acc.count(), 0);
    match acc.mean() {
        Err(e) => assert!(matches!(e.0, StatsErrorEnum::EmptyInput(_))),
        Ok(v) => panic!("the mean of no observations is undefined, got {v}"),
    }
}

/// The count tracks the pushes, and a single observation is its own mean.
#[test]
fn test_accumulator_counts_and_handles_one_observation() {
    let mut acc = MeanAccumulator::<f64>::new();
    acc.push(lift::<f64>(42.0));
    assert_eq!(acc.count(), 1);
    assert_eq!(acc.mean().unwrap(), 42.0);
    acc.push(lift::<f64>(44.0));
    assert_eq!(acc.count(), 2);
    assert_eq!(acc.mean().unwrap(), 43.0);
}
