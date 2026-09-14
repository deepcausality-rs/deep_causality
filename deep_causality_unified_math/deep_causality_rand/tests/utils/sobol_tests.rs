/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the Sobol low-discrepancy sequence.
//!
//! Reference points were produced by `scipy.stats.qmc.Sobol(d, scramble=False, bits=32)`;
//! the values are exact dyadic rationals, so equality is checked exactly.

use deep_causality_num::Float106;
use deep_causality_rand::{MAX_SOBOL_DIM, RngCore, RngError, SobolSequence, Xoshiro256};

/// A uniform `[0, 1)` from one machine word.
///
/// Built here rather than drawn through a distribution: this crate supplies entropy and Sobol
/// points, and a test of Sobol's discrepancy should not reach past that for its own baseline.
fn unit_from_word<R: RngCore + ?Sized>(rng: &mut R) -> f64 {
    (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64
}

#[test]
fn test_matches_scipy_reference_d4() {
    let s = SobolSequence::new(4).unwrap();
    let refs: &[(u64, [f64; 4])] = &[
        (0, [0.0, 0.0, 0.0, 0.0]),
        (1, [0.5, 0.5, 0.5, 0.5]),
        (2, [0.75, 0.25, 0.25, 0.25]),
        (3, [0.25, 0.75, 0.75, 0.75]),
        (5, [0.875, 0.875, 0.125, 0.375]),
        (8, [0.1875, 0.3125, 0.9375, 0.4375]),
        (13, [0.8125, 0.6875, 0.8125, 0.0625]),
    ];
    for (idx, expected) in refs {
        for (d, e) in expected.iter().enumerate() {
            assert_eq!(
                s.coordinate::<f64>(*idx, d),
                *e,
                "mismatch at index {idx}, dim {d}"
            );
        }
    }
}

#[test]
fn test_coordinates_in_unit_interval() {
    let s = SobolSequence::new(MAX_SOBOL_DIM).unwrap();
    for idx in 0..1024u64 {
        for d in 0..MAX_SOBOL_DIM {
            let x = s.coordinate::<f64>(idx, d);
            assert!((0.0..1.0).contains(&x), "x={x} out of [0,1) at {idx},{d}");
        }
    }
}

#[test]
fn test_deterministic_by_index_regardless_of_order() {
    let s = SobolSequence::new(3).unwrap();
    let a = s.coordinate::<f64>(5, 2);
    let _ = s.coordinate::<f64>(1, 0);
    let _ = s.coordinate::<f64>(99, 1);
    let b = s.coordinate::<f64>(5, 2);
    assert_eq!(a, b);
}

#[test]
fn test_point_matches_coordinate() {
    let s = SobolSequence::new(4).unwrap();
    let mut out = [0.0; 4];
    s.point::<f64>(7, &mut out);
    for (d, v) in out.iter().enumerate() {
        assert_eq!(*v, s.coordinate::<f64>(7, d));
    }
}

#[test]
fn test_dimension_cap_and_accessor() {
    assert_eq!(
        SobolSequence::new(0).err(),
        Some(RngError::UnsupportedDimension(
            "dimension 0 outside 1..=16".to_string()
        ))
    );
    assert!(matches!(
        SobolSequence::new(MAX_SOBOL_DIM + 1),
        Err(RngError::UnsupportedDimension(_))
    ));
    assert_eq!(
        SobolSequence::new(MAX_SOBOL_DIM).unwrap().dim(),
        MAX_SOBOL_DIM
    );
}

#[test]
fn test_shift_same_seed_reproducible() {
    let a = SobolSequence::new_shifted(4, 0xC0FFEE).unwrap();
    let b = SobolSequence::new_shifted(4, 0xC0FFEE).unwrap();
    for idx in 0..64u64 {
        for d in 0..4 {
            assert_eq!(a.coordinate::<f64>(idx, d), b.coordinate::<f64>(idx, d));
        }
    }
}

#[test]
fn test_shift_different_seeds_differ() {
    let a = SobolSequence::new_shifted(4, 1).unwrap();
    let b = SobolSequence::new_shifted(4, 2).unwrap();
    // At least one coordinate over a small batch must differ.
    let differ = (0..32u64)
        .any(|idx| (0..4).any(|d| a.coordinate::<f64>(idx, d) != b.coordinate::<f64>(idx, d)));
    assert!(differ, "different seeds produced identical sequences");
}

#[test]
fn test_shifted_coordinates_still_in_unit_interval() {
    let s = SobolSequence::new_shifted(MAX_SOBOL_DIM, 777).unwrap();
    for idx in 0..256u64 {
        for d in 0..MAX_SOBOL_DIM {
            assert!((0.0..1.0).contains(&s.coordinate::<f64>(idx, d)));
        }
    }
}

/// Warnock L2 star discrepancy in 2D (lower is better).
fn l2_star_discrepancy_2d(pts: &[(f64, f64)]) -> f64 {
    let n = pts.len() as f64;
    let s = 2i32;
    let mut sum1 = 0.0;
    for &(x, y) in pts {
        sum1 += (1.0 - x * x) * (1.0 - y * y);
    }
    let mut sum2 = 0.0;
    for &(xi, yi) in pts {
        for &(xk, yk) in pts {
            sum2 += (1.0 - xi.max(xk)) * (1.0 - yi.max(yk));
        }
    }
    let term = 2.0_f64.powi(1 - s) / n * sum1;
    sum2 / (n * n) - term + 3.0_f64.powi(-s)
}

#[test]
fn test_discrepancy_lower_than_pseudo_random() {
    const N: usize = 256;
    let s = SobolSequence::new(2).unwrap();
    let sobol: Vec<(f64, f64)> = (0..N as u64)
        .map(|i| (s.coordinate::<f64>(i, 0), s.coordinate::<f64>(i, 1)))
        .collect();

    // Deterministic pseudo-random baseline.
    let mut rng = Xoshiro256::from_seed(12345);
    let random: Vec<(f64, f64)> = (0..N)
        .map(|_| (unit_from_word(&mut rng), unit_from_word(&mut rng)))
        .collect();

    let d_sobol = l2_star_discrepancy_2d(&sobol);
    let d_random = l2_star_discrepancy_2d(&random);
    assert!(
        d_sobol < d_random,
        "Sobol discrepancy {d_sobol:e} not below pseudo-random {d_random:e}"
    );
}

#[test]
fn test_exact_stratification_in_quadrants() {
    // For N = 2^k points, Sobol places exactly N/4 in each half-open quadrant of the unit square.
    const N: u64 = 256;
    let s = SobolSequence::new(2).unwrap();
    let mut counts = [0u32; 4];
    for i in 0..N {
        let x = s.coordinate::<f64>(i, 0);
        let y = s.coordinate::<f64>(i, 1);
        let q = (if x < 0.5 { 0 } else { 1 }) + (if y < 0.5 { 0 } else { 2 });
        counts[q] += 1;
    }
    assert_eq!(counts, [64, 64, 64, 64]);
}

// ---------------------------------------------------------------------------------------------
// The scalar is the caller's; the resolution is the sequence's
// ---------------------------------------------------------------------------------------------

#[test]
fn a_wider_scalar_carries_the_same_coordinate() {
    // The coordinate is a 32-bit integer over `2^32`, which `f64` holds exactly. Asking for it at
    // `Float106` therefore cannot refine it: the two agree bit for bit, and a test asserting they
    // agree to some tolerance would be weaker than the truth.
    //
    // This is what "fixed width" means here. Precision as a parameter lets a caller choose the
    // scalar their arithmetic runs in; it does not invent bits the sequence never had.
    let s = SobolSequence::new_shifted(4, 0xC0FFEE).unwrap();
    for idx in [0u64, 1, 2, 3, 17, 255, 4096, 65_535, 1_000_000] {
        for d in 0..4 {
            let narrow: f64 = s.coordinate(idx, d);
            let wide: Float106 = s.coordinate(idx, d);
            assert_eq!(
                wide,
                Float106::from(narrow),
                "point {idx}, dimension {d}: the wide coordinate carries something the narrow one does not"
            );
        }
    }
}

#[test]
fn a_narrower_scalar_rounds_the_coordinate_away() {
    // The other side of the same fact, and the reason the scalar is the caller's choice rather
    // than the crate's: `f32` keeps 24 significand bits against the sequence's 32, so it cannot
    // separate points that differ in the low 8 bits.
    //
    // Consecutive Sobol indices in dimension 0 differ by a large stride, so a pair that collides
    // has to be looked for. Points 0 and 1 of dimension 1 differ by `2^-32` after the shift is
    // removed, which is exactly what `f32` cannot see.
    let s = SobolSequence::new(1).unwrap();
    let a: f64 = s.coordinate(1, 0);
    let b: f64 = a + 2f64.powi(-32);
    assert_ne!(a, b, "the two differ at f64");
    assert_eq!(
        a as f32, b as f32,
        "two coordinates one ulp of the sequence apart must collide at f32"
    );
}
