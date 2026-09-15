/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Materialising an ensemble into a carrier the caller names.
//!
//! Every assertion here is on **values**, never on a count. A count cannot separate a correct
//! diagonal from an off-by-one one, cannot separate "in order" from "reversed", and cannot separate
//! two carriers holding the same draws from two carriers holding different ones — all of those
//! have the same length. Where a count is asserted at all it is alongside the values, not instead
//! of them.

use deep_causality_haft::{Foldable, Semigroupal};
use deep_causality_linear::{DenseVector, DenseVectorWitness, ZipDenseVectorWitness};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};
use deep_causality_uncertain::{SampleSession, Uncertain, UncertainBool};

const SEED: u64 = 0x5EED_2026;

/// One signature, two carriers, the same draws in the same order.
///
/// This is the claim `materialize` exists to make: the ensemble's container is a parameter, and the
/// witness decides only *where* the draws land. If the two carriers disagreed on a single value the
/// carrier would be deciding something it has no business deciding.
#[test]
fn one_signature_fills_either_carrier_with_the_same_draws() {
    let quantity = Uncertain::<f64>::normal(10.0, 2.0);

    let mut for_vector = SampleSession::seeded(SEED);
    let vector: DenseVector<f64> = quantity
        .materialize::<DenseVectorWitness>(&mut for_vector, 32)
        .expect("materialise into a dense vector");

    let mut for_tensor = SampleSession::seeded(SEED);
    let tensor: CausalTensor<f64> = quantity
        .materialize::<CausalTensorWitness>(&mut for_tensor, 32)
        .expect("materialise into a rank-1 tensor");

    assert_eq!(
        vector.as_slice(),
        tensor.as_slice(),
        "the two carriers must hold the same draws in the same order"
    );
    assert_eq!(tensor.shape(), [32].as_slice(), "rank 1, extent 32");
}

/// The values are the session's own draws, not merely *some* draws of the right shape.
#[test]
fn the_ensemble_holds_exactly_the_sessions_draws() {
    let quantity = Uncertain::<f64>::normal(0.0, 1.0);

    let mut session = SampleSession::seeded(SEED);
    let ensemble: DenseVector<f64> = quantity
        .materialize::<DenseVectorWitness>(&mut session, 16)
        .expect("materialise");

    // The same session replayed, drawn one at a time through the ordinary surface.
    let mut replay = SampleSession::seeded(SEED);
    let expected: Vec<f64> = (0..16)
        .map(|_| quantity.sample_next(&mut replay).expect("draw"))
        .collect();

    assert_eq!(
        ensemble.as_slice(),
        expected.as_slice(),
        "materialise must be take-a-draw, sixteen times, and nothing else"
    );
}

/// An ensemble of zero draws is the carrier's empty container, not an error.
#[test]
fn an_empty_ensemble_is_the_empty_container() {
    let quantity = Uncertain::<f64>::normal(0.0, 1.0);
    let mut session = SampleSession::seeded(SEED);

    let vector: DenseVector<f64> = quantity
        .materialize::<DenseVectorWitness>(&mut session, 0)
        .expect("zero draws is a legitimate request");
    let tensor: CausalTensor<f64> = quantity
        .materialize::<CausalTensorWitness>(&mut session, 0)
        .expect("zero draws is a legitimate request");

    assert_eq!(vector.as_slice(), [0.0f64; 0].as_slice());
    assert_eq!(tensor.as_slice(), [0.0f64; 0].as_slice());
    assert_eq!(
        DenseVectorWitness::fold(vector, 7.0, |acc, x| acc + x),
        7.0,
        "folding an empty ensemble returns the accumulator unchanged"
    );
}

/// The ensemble arrives carrying its witness's structure, with nothing added by this crate.
#[test]
fn the_ensemble_carries_its_witnesss_structure() {
    let quantity = Uncertain::<f64>::point(2.0);
    let mut session = SampleSession::seeded(SEED);

    let ensemble: DenseVector<f64> = quantity
        .materialize::<DenseVectorWitness>(&mut session, 5)
        .expect("materialise");

    // `Foldable` comes from the witness; `deep_causality_uncertain` implements nothing for it.
    let total = DenseVectorWitness::fold(ensemble, 0.0, |acc, x| acc + x);
    assert_eq!(total, 10.0, "five certain twos sum to ten");
}

// -------------------------------------------------------------------------------------------
// Correlation: the diagonal, asserted pair by pair.
// -------------------------------------------------------------------------------------------

/// Two ensembles drawn at the **same indices** pair by index, and the i-th pair is the i-th draw
/// of each.
///
/// The assertion is on every pair rather than on the count, because a count of `n` is exactly what
/// an off-by-one diagonal also produces — and so does a cartesian product truncated to `n`.
#[test]
fn correlated_ensembles_pair_the_ith_draw_with_the_ith_draw() {
    let session = SampleSession::seeded(SEED);
    let left = Uncertain::<f64>::normal(100.0, 1.0);
    let right = Uncertain::<f64>::normal(-100.0, 1.0);

    // `materialize_at` holds the indices at `0..n`, so both are drawn at the same addresses.
    let left_draws: DenseVector<f64> = left
        .materialize_at::<DenseVectorWitness>(&session, 24)
        .expect("materialise left");
    let right_draws: DenseVector<f64> = right
        .materialize_at::<DenseVectorWitness>(&session, 24)
        .expect("materialise right");

    // The positional zip: `ZipDenseVectorWitness`'s `Semigroupal` pairs slot i with slot i.
    let paired: DenseVector<(f64, f64)> =
        ZipDenseVectorWitness::zip_with(left_draws.clone(), right_draws.clone(), |a, b| (a, b));

    assert_eq!(paired.len(), 24, "one pair per draw, not the product");

    for i in 0..24 {
        let (a, b) = paired.as_slice()[i];
        assert_eq!(
            a,
            left_draws.as_slice()[i],
            "pair {i} took the wrong draw from the left ensemble"
        );
        assert_eq!(
            b,
            right_draws.as_slice()[i],
            "pair {i} took the wrong draw from the right ensemble"
        );
    }
}

/// The correlation is real: two quantities drawn at the same index share that index's entropy per
/// leaf, so a quantity built from another agrees with it draw for draw.
///
/// This is what distinguishes "drawn at the same indices" from "drawn independently and zipped".
#[test]
fn the_diagonal_is_a_correlation_and_not_a_coincidence() {
    let session = SampleSession::seeded(SEED);
    let base = Uncertain::<f64>::normal(5.0, 2.0);
    let doubled = base.clone() + base.clone();

    let base_draws: DenseVector<f64> = base
        .materialize_at::<DenseVectorWitness>(&session, 24)
        .expect("materialise base");
    let doubled_draws: DenseVector<f64> = doubled
        .materialize_at::<DenseVectorWitness>(&session, 24)
        .expect("materialise doubled");

    for i in 0..24 {
        let b = base_draws.as_slice()[i];
        let d = doubled_draws.as_slice()[i];
        assert_eq!(
            d,
            b + b,
            "draw {i}: the doubled ensemble must be twice the *same* draw, not twice some draw"
        );
    }
}

/// Advancing the session is the other mode, and it is deliberately *not* correlated.
///
/// `materialize` advances by `n`, so a second call draws at fresh indices. Pinning that here stops
/// a reader from assuming either method correlates.
#[test]
fn materialize_advances_the_session_so_two_calls_differ() {
    let quantity = Uncertain::<f64>::normal(0.0, 1.0);
    let mut session = SampleSession::seeded(SEED);

    let first: DenseVector<f64> = quantity
        .materialize::<DenseVectorWitness>(&mut session, 16)
        .expect("materialise");
    let second: DenseVector<f64> = quantity
        .materialize::<DenseVectorWitness>(&mut session, 16)
        .expect("materialise");

    assert_ne!(
        first.as_slice(),
        second.as_slice(),
        "a second ensemble from an advanced session draws at fresh indices"
    );

    // And `materialize_at` is the one that does not advance: two calls agree exactly.
    let held = SampleSession::seeded(SEED);
    let a: DenseVector<f64> = quantity
        .materialize_at::<DenseVectorWitness>(&held, 16)
        .expect("materialise");
    let b: DenseVector<f64> = quantity
        .materialize_at::<DenseVectorWitness>(&held, 16)
        .expect("materialise");
    assert_eq!(a.as_slice(), b.as_slice());
}

// -------------------------------------------------------------------------------------------
// The Boolean carrier's ensemble.
// -------------------------------------------------------------------------------------------

/// The Boolean carrier materialises the same way, into the same carriers.
#[test]
fn the_boolean_carrier_materialises_too() {
    let verdict = UncertainBool::<f64>::point(true);
    let mut session = SampleSession::seeded(SEED);

    let ensemble: DenseVector<bool> = verdict
        .materialize::<DenseVectorWitness>(&mut session, 8)
        .expect("materialise a verdict ensemble");

    assert_eq!(ensemble.as_slice(), [true; 8].as_slice());
}

/// A verdict ensemble correlates with the real ensemble it was derived from, draw for draw.
#[test]
fn a_verdict_ensemble_agrees_with_the_draws_it_judges() {
    let session = SampleSession::seeded(SEED);
    let quantity = Uncertain::<f64>::normal(0.0, 1.0);
    let positive = quantity.greater_than(0.0);

    let draws: DenseVector<f64> = quantity
        .materialize_at::<DenseVectorWitness>(&session, 32)
        .expect("materialise draws");
    let verdicts: DenseVector<bool> = positive
        .materialize_at::<DenseVectorWitness>(&session, 32)
        .expect("materialise verdicts");

    for i in 0..32 {
        assert_eq!(
            verdicts.as_slice()[i],
            draws.as_slice()[i] > 0.0,
            "verdict {i} judges a different draw than the one at index {i}"
        );
    }

    // Both faces occur, so the loop above is not vacuously true.
    assert!(verdicts.as_slice().iter().any(|&v| v));
    assert!(verdicts.as_slice().iter().any(|&v| !v));
}

// -------------------------------------------------------------------------------------------
// The scalar is still a parameter here.
// -------------------------------------------------------------------------------------------

/// `materialize` is generic in the scalar as well as the carrier.
#[test]
fn an_ensemble_materialises_at_a_scalar_the_crate_never_names() {
    use deep_causality_num::BFloat16;

    let quantity = Uncertain::<BFloat16>::normal(BFloat16::from(4.0), BFloat16::from(0.0));
    let mut session = SampleSession::seeded(SEED);

    let ensemble: DenseVector<BFloat16> = quantity
        .materialize::<DenseVectorWitness>(&mut session, 6)
        .expect("materialise at BFloat16");

    assert_eq!(ensemble.as_slice(), [BFloat16::from(4.0); 6].as_slice());
}

/// `BFloat16` carries f32's range, which is what makes it a drop-in for an ensemble's storage.
///
/// The composition documentation claims an ensemble can be carried at `BFloat16` wherever the
/// quantity's *numerical range* fits f32, at half f32's memory and a quarter of f64's. That claim
/// rests entirely on the exponent, so the exponent is what this pins: same smallest normal, and a
/// largest finite within half a percent — short only because the significand is shorter, not
/// because the range is.
#[test]
fn bfloat16_carries_f32s_range_which_is_what_the_memory_claim_rests_on() {
    use deep_causality_num::{BFloat16, Float};

    // The smallest positive normal is identical: both are 2^-126.
    assert_eq!(
        <BFloat16 as Float>::min_positive_value().to_f32(),
        f32::MIN_POSITIVE,
        "bf16 and f32 must share the bottom of the normal range"
    );

    // The largest finite differs only in the significand's last bits: (2 - 2^-7) vs (2 - 2^-23).
    let bf16_max = <BFloat16 as Float>::max_value().to_f32();
    assert!(
        bf16_max / f32::MAX > 0.995,
        "bf16 MAX {bf16_max:e} must be within half a percent of f32 MAX {:e}",
        f32::MAX
    );

    // And it is half the width of f32, a quarter of f64 — the whole point of the trade.
    assert_eq!(core::mem::size_of::<BFloat16>(), 2);
    assert_eq!(core::mem::size_of::<f32>(), 4);
    assert_eq!(core::mem::size_of::<f64>(), 8);

    // A quantity at the top of f32's range materialises at bf16 without saturating.
    let large = Uncertain::<BFloat16>::point(BFloat16::from(3.0e38));
    let mut session = SampleSession::seeded(SEED);
    let ensemble: DenseVector<BFloat16> = large
        .materialize::<DenseVectorWitness>(&mut session, 4)
        .expect("materialise near the top of the range");
    for v in ensemble.as_slice() {
        assert!(v.is_finite(), "a 3e38 draw must stay finite at bf16");
    }
}
