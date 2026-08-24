/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Where `DenseMatrix` sits in the tower. These are compile-time admissions: each witness function
//! compiles only if the bound admits the type, so calling it is the assertion. They pass before the
//! implementation exists, because they check the type system rather than a body.

use deep_causality_algebra::{
    Additive, Annihilating, Associative, Commutative, Distributive, Module, Multiplicative, Ring,
};
use deep_causality_linear::DenseMatrix;

fn admits_associative_add<T: Associative<Additive>>() {}
fn admits_commutative_add<T: Commutative<Additive>>() {}
fn admits_associative_mul<T: Associative<Multiplicative>>() {}
fn admits_distributive<T: Distributive>() {}
fn admits_annihilating<T: Annihilating>() {}
fn admits_ring<T: Ring>() {}
fn admits_module<M: Module<R>, R: Ring>() {}

#[test]
fn test_dense_matrix_carries_the_markers_that_reach_ring() {
    admits_associative_add::<DenseMatrix<f64>>();
    admits_commutative_add::<DenseMatrix<f64>>();
    admits_associative_mul::<DenseMatrix<f64>>();
    admits_distributive::<DenseMatrix<f64>>();
    admits_annihilating::<DenseMatrix<f64>>();
}

#[test]
fn test_dense_matrix_carries_them_over_the_integers_too() {
    admits_associative_add::<DenseMatrix<i64>>();
    admits_distributive::<DenseMatrix<i64>>();
    admits_annihilating::<DenseMatrix<i64>>();
}

#[test]
fn test_dense_matrix_reaches_ring_and_module() {
    admits_ring::<DenseMatrix<f64>>();
    admits_ring::<DenseMatrix<i64>>();
    admits_module::<DenseMatrix<f64>, f64>();
    admits_module::<DenseMatrix<i64>, i64>();
}
