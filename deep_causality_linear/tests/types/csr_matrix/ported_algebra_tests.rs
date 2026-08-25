/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The algebra laws `CsrMatrix` obeys, on worked examples.
//!
//! Ported from `deep_causality_sparse/tests/types/sparse_matrix/algebra_tests.rs`, which is the
//! record of how the retired implementation behaved. Each test names one law — an additive
//! identity, an additive inverse, commutativity, associativity, a multiplicative identity,
//! distributivity, annihilation — and checks it on values a reader can verify by hand, so a
//! reimplementation that computes a different answer is caught here rather than downstream.
//!
//! The sparse file's `admits_*` functions are compile-time witnesses for `AbelianGroup`, `Ring` and
//! `Module`. This crate keeps those as pins in `src/traits/tower_pins.rs`, where
//! `pin_container_ring::<CsrMatrix<f64>>()` and `pin_module::<CsrMatrix<f64>, f64>()` already stand,
//! so they are not restated as tests.

use deep_causality_linear::{CsrMatrix, MatrixBuild};
use deep_causality_num::{One, Zero};

fn m(triplets: &[(usize, usize, f64)], r: usize, c: usize) -> CsrMatrix<f64> {
    CsrMatrix::from_triplets(r, c, triplets).unwrap()
}

/// The `n × n` identity, through the inherent `one(n)` this crate now carries.
fn identity(n: usize) -> CsrMatrix<f64> {
    CsrMatrix::one(n)
}

#[test]
fn test_the_zero_matrix_has_the_requested_shape_and_stores_nothing() {
    let z: CsrMatrix<f64> = CsrMatrix::zeros(3, 3);
    assert_eq!(z.shape(), (3, 3));
    assert!(z.values().is_empty(), "every entry is structural");
}

#[test]
fn test_adding_the_zero_matrix_returns_the_original() {
    let a = m(&[(0, 0, 1.0), (1, 1, 2.0)], 2, 2);
    let z: CsrMatrix<f64> = CsrMatrix::zeros(2, 2);
    assert_eq!(a.clone() + z, a, "A + 0 = A");
}

#[test]
fn test_a_matrix_plus_its_negation_stores_nothing() {
    let a = m(&[(0, 0, 1.0), (1, 1, 2.0)], 2, 2);
    let sum = a.clone() + (-a.clone());
    assert!(
        sum.values().is_empty(),
        "the cancelled entries become structural zeros"
    );
    assert_eq!(
        sum.shape(),
        a.shape(),
        "the shape survives the cancellation"
    );
}

#[test]
fn test_addition_commutes() {
    let a = m(&[(0, 0, 1.0)], 2, 2);
    let b = m(&[(1, 1, 2.0)], 2, 2);
    assert_eq!(a.clone() + b.clone(), b + a, "A + B = B + A");
}

#[test]
fn test_addition_associates() {
    let a = m(&[(0, 0, 1.0)], 2, 2);
    let b = m(&[(1, 1, 2.0)], 2, 2);
    let c = m(&[(0, 1, 3.0)], 2, 2);
    let left = (a.clone() + b.clone()) + c.clone();
    let right = a + (b + c);
    assert_eq!(left, right, "(A + B) + C = A + (B + C)");
}

#[test]
fn test_scaling_multiplies_every_stored_entry() {
    let a = m(&[(0, 0, 1.0), (1, 1, 2.0)], 2, 2);
    let b = a.scale(3.0);
    assert_eq!(b.get_value_at(0, 0), 3.0);
    assert_eq!(b.get_value_at(1, 1), 6.0);
}

#[test]
fn test_the_identity_matrix_is_the_multiplicative_identity() {
    let i = identity(3);
    let a = m(&[(0, 0, 1.0), (1, 1, 2.0), (2, 2, 3.0)], 3, 3);
    assert_eq!(i * a.clone(), a, "I * A = A");
}

#[test]
#[should_panic(expected = "shape mismatch")]
fn test_the_add_operator_panics_on_a_shape_mismatch() {
    let a: CsrMatrix<f64> = CsrMatrix::zeros(2, 2);
    let b: CsrMatrix<f64> = CsrMatrix::zeros(3, 3);
    let _c = a + b;
}

#[test]
fn test_the_scalar_zero_is_the_empty_matrix() {
    let scalar_zero: CsrMatrix<f64> = Zero::zero();
    assert_eq!(scalar_zero.shape(), (0, 0));
    assert!(scalar_zero.is_zero());
    assert!(!m(&[(0, 0, 1.0)], 2, 2).is_zero());
}

#[test]
fn test_the_scalar_one_is_the_1x1_identity() {
    let scalar_one: CsrMatrix<f64> = One::one();
    assert_eq!(scalar_one.shape(), (1, 1));
    assert!(scalar_one.is_one());
    // A single 1 on a 2x2 leaves position (1, 1) at zero, so it is not an identity of any size.
    assert!(!m(&[(0, 0, 1.0)], 2, 2).is_one());
}

#[test]
fn test_scaling_returns_a_new_matrix_and_leaves_the_original() {
    let matrix = m(&[(0, 0, 1.0), (1, 1, 2.0)], 2, 2);
    let scaled = matrix.scale(2.0);
    assert_eq!(scaled.values().as_slice(), [2.0, 4.0].as_slice());
    assert_ne!(matrix.values(), scaled.values());
}

#[test]
fn test_the_multiplication_operator_carries_the_module_scaling() {
    let matrix = m(&[(0, 0, 2.0), (1, 1, 3.0)], 2, 2);
    let scaled = matrix * 2.0_f64;
    assert_eq!(scaled.get_value_at(0, 0), 4.0);
    assert_eq!(scaled.get_value_at(1, 1), 6.0);
}

#[test]
fn test_multiplication_distributes_over_addition() {
    // The law `Distributive` promises: A(B + C) = AB + AC.
    let a = m(&[(0, 0, 1.0), (0, 1, 2.0), (1, 1, 3.0)], 2, 2);
    let b = m(&[(0, 0, 4.0), (1, 0, 5.0)], 2, 2);
    let c = m(&[(0, 1, 6.0), (1, 1, 7.0)], 2, 2);

    let lhs = a.mat_mult(&(b.clone() + c.clone())).unwrap();
    let rhs = a.mat_mult(&b).unwrap() + a.mat_mult(&c).unwrap();

    for i in 0..2 {
        for j in 0..2 {
            assert_eq!(
                lhs.get_value_at(i, j),
                rhs.get_value_at(i, j),
                "mismatch at ({i}, {j})"
            );
        }
    }
}

#[test]
fn test_the_zero_matrix_annihilates() {
    // The law `Annihilating` promises: 0 * A = 0.
    let a = m(&[(0, 0, 1.0), (1, 1, 2.0)], 2, 2);
    let zero: CsrMatrix<f64> = CsrMatrix::zeros(2, 2);
    let product = zero.mat_mult(&a).unwrap();
    for i in 0..2 {
        for j in 0..2 {
            assert_eq!(product.get_value_at(i, j), 0.0);
        }
    }
}

#[test]
fn test_matrix_multiplication_does_not_commute() {
    // Why `Commutative<Multiplicative>` is deliberately absent, and why a `CommutativeRing` bound
    // must keep refusing this type.
    let a = m(&[(0, 0, 1.0), (0, 1, 1.0)], 2, 2);
    let b = m(&[(1, 0, 1.0)], 2, 2);
    let ab = a.mat_mult(&b).unwrap();
    let ba = b.mat_mult(&a).unwrap();
    assert_eq!(ab.get_value_at(0, 0), 1.0);
    assert_eq!(ba.get_value_at(0, 0), 0.0);
    assert_ne!(ab.get_value_at(0, 0), ba.get_value_at(0, 0));
}
