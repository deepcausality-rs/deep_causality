/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The entrywise residual reduction, and the Frobenius norm after it stopped folding `re² + im²`.
//!
//! # The oracle
//!
//! Two closed forms evaluated by hand, and nothing computed by the code under test:
//!
//! * `|3 + 4i| = 5` and `|0 + 12i| = 12`, from the `3-4-5` triple and a pure imaginary.
//! * The Frobenius norm of a matrix whose entries are `3` and `4` with the rest zero is
//!   `√(9 + 16) = 5`, the same triple read over a matrix.
//!
//! Both are scaled by `2⁶⁰⁰` for the overflow cases. `3·2⁶⁰⁰`, `4·2⁶⁰⁰` and `5·2⁶⁰⁰` are all exactly
//! representable in `f64` while `(3·2⁶⁰⁰)²` is not, so the scaled form's answer is exact and the
//! form it replaces returns infinity. The tests assert equality rather than a tolerance.
//!
//! # Corner cases, enumerated before the suite
//!
//! Empty input; a single entry; all zeros; the ordinary range; entries whose squares overflow;
//! a `NaN` entry, which every one of the five replaced sites skipped because each folded with
//! `if x > acc`; the conjugate pairing the Hermiticity defect needs; and the Frobenius norm of a
//! non-square matrix, which is defined over the entries regardless of shape.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{frobenius_norm, hermiticity_defect, max_modulus};
use deep_causality_tensor::CausalTensor;

/// `2⁶⁰⁰`, the scale at which a square overflows `f64` but the value does not.
fn wide() -> f64 {
    2f64.powi(600)
}

// =============================================================================
// max_modulus
// =============================================================================

#[test]
fn test_max_modulus_of_an_empty_sequence_is_zero() {
    let none: [Complex<f64>; 0] = [];
    assert_eq!(max_modulus(none.into_iter()), 0.0);
}

#[test]
fn test_max_modulus_picks_the_larger_of_two_hand_evaluated_moduli() {
    // |3 + 4i| = 5, |0 + 12i| = 12.
    let v = [Complex::new(3.0f64, 4.0), Complex::new(0.0, 12.0)];
    assert_eq!(max_modulus(v.into_iter()), 12.0);
}

#[test]
fn test_max_modulus_does_not_depend_on_order() {
    let a = [Complex::new(3.0f64, 4.0), Complex::new(0.0, 12.0)];
    let b = [Complex::new(0.0f64, 12.0), Complex::new(3.0, 4.0)];
    assert_eq!(max_modulus(a.into_iter()), max_modulus(b.into_iter()));
}

#[test]
fn test_max_modulus_of_a_single_entry_is_its_modulus() {
    assert_eq!(max_modulus([Complex::new(-3.0f64, 4.0)].into_iter()), 5.0);
}

#[test]
fn test_max_modulus_of_all_zeros_is_zero() {
    let v = [Complex::new(0.0f64, 0.0); 4];
    assert_eq!(max_modulus(v.into_iter()), 0.0);
}

#[test]
fn test_max_modulus_does_not_overflow_on_a_large_entry() {
    // |3·2⁶⁰⁰ + 4·2⁶⁰⁰ i| = 5·2⁶⁰⁰. The replaced form squared the components and returned infinity.
    let p = wide();
    let v = [Complex::new(3.0 * p, 4.0 * p)];
    let m = max_modulus(v.into_iter());
    assert!(
        m.is_finite(),
        "the modulus is representable and must be finite"
    );
    assert_eq!(m, 5.0 * p);
}

#[test]
fn test_max_modulus_skips_a_nan_entry() {
    // Every one of the five replaced sites folded with `if x > acc`, and a NaN never compares
    // greater, so a NaN was skipped rather than propagated. That behaviour is preserved.
    let v = [
        Complex::new(3.0f64, 4.0),
        Complex::new(f64::NAN, 0.0),
        Complex::new(0.0, 1.0),
    ];
    assert_eq!(max_modulus(v.into_iter()), 5.0);
}

// =============================================================================
// frobenius_norm
// =============================================================================

/// `[[3, 4], [0, 0]]` as a complex matrix: Frobenius norm `√(9 + 16) = 5`.
fn triple_matrix(scale: f64) -> CausalTensor<Complex<f64>> {
    CausalTensor::new(
        vec![
            Complex::new(3.0 * scale, 0.0),
            Complex::new(4.0 * scale, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
        ],
        vec![2, 2],
    )
    .unwrap()
}

#[test]
fn test_the_frobenius_norm_against_its_closed_form() {
    assert_eq!(frobenius_norm(&triple_matrix(1.0)), 5.0);
}

#[test]
fn test_the_frobenius_norm_counts_the_imaginary_parts() {
    // Entries 3i and 4i: the moduli are 3 and 4, so the norm is still 5.
    let m = CausalTensor::new(
        vec![
            Complex::new(0.0f64, 3.0),
            Complex::new(0.0, 4.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
        ],
        vec![2, 2],
    )
    .unwrap();
    assert_eq!(frobenius_norm(&m), 5.0);
}

#[test]
fn test_the_frobenius_norm_of_a_zero_matrix_is_zero() {
    let m = CausalTensor::new(vec![Complex::new(0.0f64, 0.0); 4], vec![2, 2]).unwrap();
    let n = frobenius_norm(&m);
    assert_eq!(n, 0.0);
    assert!(!n.is_nan());
}

#[test]
fn test_the_frobenius_norm_does_not_overflow_on_large_entries() {
    // This is the defect the delegation removes: the folded `re² + im²` reached infinity above
    // about 1.34e154, and `markov_pairs` fed that infinity to a commutator threshold.
    let n = frobenius_norm(&triple_matrix(wide()));
    assert!(
        n.is_finite(),
        "the norm is representable and must be finite"
    );
    assert_eq!(n, 5.0 * wide());
}

#[test]
fn test_the_frobenius_norm_reads_a_non_square_matrix() {
    // Defined over the entries, so the shape does not matter: 1 + 4 + 4 + 16 = 25.
    let m = CausalTensor::new(
        vec![
            Complex::new(1.0f64, 0.0),
            Complex::new(0.0, 2.0),
            Complex::new(2.0, 0.0),
            Complex::new(0.0, 4.0),
        ],
        vec![1, 4],
    )
    .unwrap();
    assert_eq!(frobenius_norm(&m), 5.0);
}

// =============================================================================
// hermiticity_defect — the fifth residual site, whose pairing is not a zip
// =============================================================================

#[test]
fn test_a_hermitian_matrix_has_no_hermiticity_defect() {
    // [[1, 2+3i], [2-3i, 4]] is Hermitian by construction.
    let m = CausalTensor::new(
        vec![
            Complex::new(1.0f64, 0.0),
            Complex::new(2.0, 3.0),
            Complex::new(2.0, -3.0),
            Complex::new(4.0, 0.0),
        ],
        vec![2, 2],
    )
    .unwrap();
    assert_eq!(hermiticity_defect(&m).unwrap(), 0.0);
}

#[test]
fn test_the_hermiticity_defect_is_the_largest_entrywise_discrepancy() {
    // [[1, 0], [0+7i, 1]]: M_10 − conj(M_01) = 7i − 0 = 7i, so the defect is 7.
    let m = CausalTensor::new(
        vec![
            Complex::new(1.0f64, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 7.0),
            Complex::new(1.0, 0.0),
        ],
        vec![2, 2],
    )
    .unwrap();
    assert_eq!(hermiticity_defect(&m).unwrap(), 7.0);
}

#[test]
fn test_the_hermiticity_defect_does_not_overflow() {
    // The same discrepancy at a scale whose square is not representable.
    let p = wide();
    let m = CausalTensor::new(
        vec![
            Complex::new(1.0f64, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(3.0 * p, 4.0 * p),
            Complex::new(1.0, 0.0),
        ],
        vec![2, 2],
    )
    .unwrap();
    let d = hermiticity_defect(&m).unwrap();
    assert!(d.is_finite());
    assert_eq!(d, 5.0 * p);
}
