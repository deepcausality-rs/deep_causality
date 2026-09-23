/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `RandomField` at every scalar the lattice can run in.
//!
//! The trait is implemented for every scalar the lattice runs in, so a gauge field has the
//! precision its scalar parameter names. An implementation for `f64` alone would give every field
//! that one precision whatever the parameter said, and these tests are what rules it out.

use deep_causality_num::{Float106, ToPrimitive};
use deep_causality_num_complex::Complex;
use deep_causality_stats::Xoshiro256;
use deep_causality_topology::{LinkVariable, RandomField, U1};

const SEED: u64 = 0x5EED_2026;

/// Draws at one scalar, lowered for comparison.
fn draws<T: RandomField + ToPrimitive>(n: usize) -> Vec<f64> {
    let mut g = Xoshiro256::from_seed(SEED);
    (0..n)
        .map(|_| {
            T::generate_uniform(&mut g)
                .to_f64()
                .expect("every supported scalar lowers to f64")
        })
        .collect()
}

#[test]
fn the_draw_is_centred_at_every_scalar() {
    // `[-0.5, 0.5)` is the trait's contract, and the mean of a uniform over it is zero.
    for (name, xs) in [
        ("f32", draws::<f32>(100_000)),
        ("f64", draws::<f64>(100_000)),
        ("Float106", draws::<Float106>(100_000)),
    ] {
        assert!(
            xs.iter().all(|&x| (-0.5..0.5).contains(&x)),
            "{name}: a draw left [-0.5, 0.5)"
        );
        let mean = xs.iter().sum::<f64>() / xs.len() as f64;
        assert!(mean.abs() < 0.005, "{name}: mean {mean} is not centred");

        // A uniform on an interval of width one has variance 1/12.
        let var = xs.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / xs.len() as f64;
        assert!(
            (var - 1.0 / 12.0).abs() < 0.002,
            "{name}: variance {var} is not that of a unit-width uniform"
        );
    }
}

#[test]
fn the_double_double_draw_carries_more_than_a_double() {
    // The load-bearing check. A `Float106` draw that secretly came from one 53-bit word would pass
    // every range and moment assertion above — it is an `f64` wearing a wider type. The only thing
    // that separates them is whether any draw has bits below the `f64` rounding of itself.
    let mut g = Xoshiro256::from_seed(SEED);
    let wide_enough = (0..1_000).any(|_| {
        let v: Float106 = RandomField::generate_uniform(&mut g);
        Float106::from(v.to_f64()) != v
    });
    assert!(
        wide_enough,
        "every Float106 draw was exactly an f64: the second limb never received entropy"
    );
}

#[test]
fn two_scalars_draw_different_streams() {
    // Not a law, an observation worth pinning: the scalars consume the generator differently —
    // `Float106` takes two words per draw where `f64` takes one — so the streams must not coincide.
    let narrow = draws::<f64>(16);
    let wide = draws::<Float106>(16);
    assert_ne!(
        narrow[1], wide[1],
        "the double-double draw consumed the same words as the double"
    );
}

#[test]
fn a_link_variable_is_random_at_every_scalar() {
    // The trait exists for this: a gauge link whose matrix entries are drawn. Before this change
    // only the `f64` row compiled.
    let mut g = Xoshiro256::from_seed(SEED);
    let at_f32: LinkVariable<U1, Complex<f32>, f32> =
        LinkVariable::try_random(&mut g).expect("a random U(1) link at f32");
    let at_f64: LinkVariable<U1, Complex<f64>, f64> =
        LinkVariable::try_random(&mut g).expect("a random U(1) link at f64");
    let at_wide: LinkVariable<U1, Complex<Float106>, Float106> =
        LinkVariable::try_random(&mut g).expect("a random U(1) link at Float106");

    // A U(1) link is a single complex number on the unit circle, so its Frobenius norm is one
    // whatever the scalar. That is what says the draw fed a well-formed group element rather than
    // noise: an unprojected random entry would land anywhere in the unit square.
    for (name, norm_sq) in [
        ("f32", at_f32.frobenius_norm_sq().to_f64().unwrap()),
        ("f64", at_f64.frobenius_norm_sq().to_f64().unwrap()),
        ("Float106", at_wide.frobenius_norm_sq().to_f64()),
    ] {
        assert!(
            (norm_sq - 1.0).abs() < 1e-6,
            "{name}: the random link has squared norm {norm_sq}, not one"
        );
    }
}
