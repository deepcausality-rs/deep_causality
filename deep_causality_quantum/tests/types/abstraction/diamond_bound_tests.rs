/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The two-sided bound on the closed-form pair: `id` against `R_z(θ)` has diamond distance
//! `2 sin(θ/2)` (the distance from the origin to the chord between `e^{±iθ/2}` is `cos(θ/2)`, and
//! `‖U − V‖_⋄ = 2√(1 − δ²)`), and Frobenius residual `2√2 sin(θ/2)` on the unnormalised Choi
//! operators. Both ends of the bound hold at every θ and the upper bound is loose by `2√2` exactly.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{Axis, Channel, DiamondBound, NumericCaps, QcMorphism, QubitOperator};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_both_bounds_hold_on_the_rotation_pair() {
    let caps = NumericCaps::default();
    let id =
        QcMorphism::from_channel(&Channel::unitary(&QubitOperator::identity()).unwrap()).unwrap();
    for k in 1..=8 {
        let theta = std::f64::consts::PI * k as f64 / 8.0;
        let rz = QcMorphism::from_channel(
            &Channel::unitary(&QubitOperator::rotation(Axis::Z, theta).unwrap()).unwrap(),
        )
        .unwrap();
        let (r, _) = id.frobenius_distance(&rz, &caps).unwrap();
        let diamond = 2.0 * (theta / 2.0).sin();
        assert!(
            (r - 2.0 * 2f64.sqrt() * (theta / 2.0).sin()).abs() < 1e-12,
            "θ = {theta}: r = {r}"
        );
        let b = DiamondBound::from_frobenius(r, 2, 2);
        assert!(b.lower <= diamond + 1e-12, "lower {} vs {diamond}", b.lower);
        assert!(b.upper >= diamond - 1e-12, "upper {} vs {diamond}", b.upper);
        assert!(
            (b.upper / diamond - 2.0 * 2f64.sqrt()).abs() < 1e-9,
            "the upper bound is loose by 2√2"
        );
        assert!((b.amplification - 2.0).abs() < 1e-15);
    }
}

#[test]
fn test_zero_residual_certifies_zero_distance_and_dimensions_enter() {
    let z = DiamondBound::from_frobenius(0.0f64, 4, 2);
    assert_eq!(z.lower, 0.0);
    assert_eq!(z.upper, 0.0);
    assert!((z.amplification - 8f64.sqrt()).abs() < 1e-15);
    let b = DiamondBound::from_frobenius(1.0f64, 4, 2);
    assert!((b.lower - 0.25).abs() < 1e-15);
    assert!((b.upper - 8f64.sqrt()).abs() < 1e-15);
}

/// A zero dimension names no system and is read as the trivial system, dimension one.
#[test]
fn test_a_zero_dimension_is_read_as_the_trivial_system() {
    assert_eq!(
        DiamondBound::from_frobenius(0.5f64, 0, 0),
        DiamondBound::from_frobenius(0.5f64, 1, 1)
    );
    assert_eq!(
        DiamondBound::from_frobenius_blocks(0.5f64, 2, 2, &[0], &[0, 3]),
        DiamondBound::from_frobenius_blocks(0.5f64, 2, 2, &[], &[3])
    );
    assert_eq!(
        DiamondBound::from_frobenius_blocks(0.5f64, 4, 2, &[], &[]),
        DiamondBound::from_frobenius(0.5f64, 4, 2)
    );
}

/// A classical channel as a morphism on the trivial quantum system: one scalar block `√p(y|x)` per
/// `(x, y)`, so its block Choi operators are the probabilities themselves.
fn classical(rows: &[&[f64]]) -> QcMorphism<f64> {
    let n_out = rows[0].len();
    let mut m = QcMorphism::<f64>::new(1, 1, vec![rows.len()], vec![n_out]).unwrap();
    for (x, row) in rows.iter().enumerate() {
        for (y, &p) in row.iter().enumerate() {
            let k = CausalTensor::from_slice(&[Complex::new(p.sqrt(), 0.0)], &[1, 1]);
            m.push(vec![x], vec![y], vec![k]).unwrap();
        }
    }
    m
}

/// The diamond distance of two classical channels: the largest total variation over the inputs,
/// `max_x Σ_y |p(y|x) − q(y|x)|`.
fn diamond(p: &[&[f64]], q: &[&[f64]]) -> f64 {
    p.iter()
        .zip(q)
        .map(|(a, b)| a.iter().zip(*b).map(|(u, v)| (u - v).abs()).sum::<f64>())
        .fold(0.0, f64::max)
}

/// Two classical input blocks whose residuals differ, `(1, 0)` against `(0, 1)` and `(1, 0)`
/// against `(½, ½)`: the direct-sum residual is `√(2 + ½)` and the diamond distance `2`, above
/// the single-block upper bound `√(d_in d_out) · r = r` and inside the block bound with the
/// factor `√n_out = √2`. Three identical input blocks of the first pair have residual `√6` against
/// the same distance `2`, below the single-block lower bound `r / d_in = √6` and above the block
/// bound `r / √(n_in n_out) = 1`.
#[test]
fn test_the_block_bound_brackets_the_diamond_distance_of_a_direct_sum() {
    let caps = NumericCaps::default();
    let p: &[&[f64]] = &[&[1.0, 0.0], &[1.0, 0.0]];
    let q: &[&[f64]] = &[&[0.0, 1.0], &[0.5, 0.5]];
    let (r, _) = classical(p)
        .frobenius_distance(&classical(q), &caps)
        .unwrap();
    assert!((r - 2.5f64.sqrt()).abs() < 1e-12, "{r}");
    let d = diamond(p, q);
    assert_eq!(d, 2.0);
    let single = DiamondBound::from_frobenius(r, 1, 1);
    assert!(
        single.upper < d,
        "the single-block upper bound {} fails",
        single.upper
    );
    let block = DiamondBound::from_frobenius_blocks(r, 1, 1, &[2], &[2]);
    assert!(block.lower <= d && d <= block.upper, "{block:?}");
    assert!((block.amplification - 2f64.sqrt()).abs() < 1e-15);
    assert!((block.lower - r / 2.0).abs() < 1e-15);

    let (one_zero, zero_one): (&[f64], &[f64]) = (&[1.0, 0.0], &[0.0, 1.0]);
    let p3: &[&[f64]] = &[one_zero; 3];
    let q3: &[&[f64]] = &[zero_one; 3];
    let (r3, _) = classical(p3)
        .frobenius_distance(&classical(q3), &caps)
        .unwrap();
    assert!((r3 - 6f64.sqrt()).abs() < 1e-12, "{r3}");
    let d3 = diamond(p3, q3);
    assert_eq!(d3, 2.0);
    let single = DiamondBound::from_frobenius(r3, 1, 1);
    assert!(
        single.lower > d3,
        "the single-block lower bound {} fails",
        single.lower
    );
    let block = DiamondBound::from_frobenius_blocks(r3, 1, 1, &[3], &[2]);
    assert!(block.lower <= d3 && d3 <= block.upper, "{block:?}");
    assert!((block.lower - 1.0).abs() < 1e-12);
    assert!((block.upper - 2.0 * 3f64.sqrt()).abs() < 1e-12);
}
