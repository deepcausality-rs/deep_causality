/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the sparse-operator helpers shared by the differential
//! operators: how a stored incidence value maps into the field the cochain
//! carries.

use deep_causality_linear::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, Simplex, SimplicialComplex, Skeleton};

/// The line complex `0 —— 1`: two vertices and one edge, with the coboundary
/// `δ₀` supplied by the caller.
fn line_manifold(coboundary: CsrMatrix<i8>) -> Manifold<SimplicialComplex<f64>, f64> {
    let skeleton_0 = Skeleton::new(0, vec![Simplex::new(vec![0]), Simplex::new(vec![1])]);
    let skeleton_1 = Skeleton::new(1, vec![Simplex::new(vec![0, 1])]);
    let d1 = CsrMatrix::from_triplets(2, 1, &[(0, 0, -1i8), (1, 0, 1)]).unwrap();
    let complex = SimplicialComplex::new(
        vec![skeleton_0, skeleton_1],
        vec![d1],
        vec![coboundary],
        Vec::new(),
    );
    let data = CausalTensor::new(vec![0.0, 0.0, 0.0], vec![3]).unwrap();
    Manifold::new(complex, data, 0).unwrap()
}

#[test]
fn exterior_derivative_is_the_incidence_weighted_difference() {
    // δ₀ = ∂₁ᵀ on the line complex: the edge picks up `f(1) − f(0)`.
    let coboundary = CsrMatrix::from_triplets(1, 2, &[(0, 0, -1i8), (0, 1, 1)]).unwrap();
    let manifold = line_manifold(coboundary);

    let df = manifold.exterior_derivative_of(&[3.0, 5.0], 0);
    assert_eq!(df.as_slice(), &[2.0]);
}

#[test]
fn a_stored_zero_incidence_contributes_nothing() {
    // A sparse operator may store an explicit zero at a position. Zero is the
    // additive identity, so that entry contributes no term and the edge sees
    // only `−f(0)`.
    let coboundary = CsrMatrix::from_triplets(1, 2, &[(0, 0, -1i8), (0, 1, 1)])
        .unwrap()
        .map_values(|v| if v > 0 { 0i8 } else { v });
    let manifold = line_manifold(coboundary);

    let df = manifold.exterior_derivative_of(&[3.0, 5.0], 0);
    assert_eq!(df.as_slice(), &[-3.0]);
}
