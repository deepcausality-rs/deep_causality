/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Where `DenseVector` sits in the tower.
//!
//! It carries the additive markers and no multiplicative ones, because it has no `Mul`. Claiming
//! `Associative<Multiplicative>` would be a claim about an operation that does not exist.

use deep_causality_algebra::{
    AbelianGroup, Additive, Annihilating, Associative, Commutative, Module, Ring,
};
use deep_causality_linear::DenseVector;

fn admits_associative_add<T: Associative<Additive>>() {}
fn admits_commutative_add<T: Commutative<Additive>>() {}
fn admits_annihilating<T: Annihilating>() {}
fn admits_abelian_group<T: AbelianGroup>() {}
fn admits_module<M: Module<R>, R: Ring>() {}

#[test]
fn test_vector_carries_the_additive_markers() {
    admits_associative_add::<DenseVector<f64>>();
    admits_commutative_add::<DenseVector<f64>>();
    admits_annihilating::<DenseVector<f64>>();
}

#[test]
fn test_vector_is_an_abelian_group() {
    admits_abelian_group::<DenseVector<f64>>();
    admits_abelian_group::<DenseVector<i64>>();
}

#[test]
fn test_vector_is_a_module_over_its_scalar_ring() {
    // Module<R: Ring> is the tower's name for a vector space. Stating it at the general level is
    // what admits Z: DenseVector<i64> scaled by i64 is a module and is not a vector space.
    admits_module::<DenseVector<f64>, f64>();
    admits_module::<DenseVector<i64>, i64>();
}
