/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Where `CsrMatrix` sits in the tower, and what it deliberately refuses.

use deep_causality_algebra::{
    Additive, Annihilating, Associative, Commutative, Distributive, Module, Multiplicative, Ring,
};
use deep_causality_linear::CsrMatrix;

fn admits_associative_add<T: Associative<Additive>>() {}
fn admits_commutative_add<T: Commutative<Additive>>() {}
fn admits_associative_mul<T: Associative<Multiplicative>>() {}
fn admits_distributive<T: Distributive>() {}
fn admits_annihilating<T: Annihilating>() {}
fn admits_ring<T: Ring>() {}
fn admits_module<M: Module<R>, R: Ring>() {}

#[test]
fn test_csr_matrix_carries_the_markers_that_reach_ring() {
    admits_associative_add::<CsrMatrix<f64>>();
    admits_commutative_add::<CsrMatrix<f64>>();
    admits_associative_mul::<CsrMatrix<f64>>();
    admits_distributive::<CsrMatrix<f64>>();
    admits_annihilating::<CsrMatrix<f64>>();
}

#[test]
fn test_csr_matrix_reaches_ring_and_module() {
    admits_ring::<CsrMatrix<f64>>();
    admits_ring::<CsrMatrix<i64>>();
    admits_module::<CsrMatrix<f64>, f64>();
    admits_module::<CsrMatrix<i64>, i64>();
}
