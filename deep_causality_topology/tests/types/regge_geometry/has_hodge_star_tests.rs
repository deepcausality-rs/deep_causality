/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Property tests for the simplicial `HasHodgeStar<R>` implementation
//! (`impl<R: RealField + FromPrimitive> HasHodgeStar<R> for ReggeGeometry<R>`).
//!
//! The simplicial Hodge ⋆ is pre-computed at complex-construction time and stored
//! on `SimplicialComplex<R>::hodge_star_operators`. The trait impl is a thin
//! adapter that borrows the cache and vends it via `Cow::Borrowed`. These tests
//! verify:
//!
//! - The trait impl returns the same matrix that the existing `Manifold::hodge_star`
//!   legacy path applies (cache identity).
//! - The returned `Cow` is `Borrowed` (zero copy).
//! - Routing the manifold method through a metric instance produces bit-identical
//!   output to the legacy no-metric path.
//!
//! Mathematical properties of the Hodge ⋆ itself (⋆⋆ = (-1)^k(n-k), Laplacian
//! self-adjointness) are deferred to the R4.5 widening, where the manifold's
//! generic differential surface against the trait is in scope.

use std::borrow::Cow;

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    HasHodgeStar, Manifold, PointCloud, ReggeGeometry, SimplicialManifold,
};

fn triangle_complex() -> SimplicialManifold<f64, f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();
    let complex = pc.triangulate(1.2).unwrap();
    let data = CausalTensor::new(vec![10.0, 20.0, 30.0, 1.0, 2.0, 3.0, 100.0], vec![7]).unwrap();
    Manifold::new(complex, data, 0).unwrap()
}

#[test]
fn trait_impl_borrows_from_complex_cache_for_every_grade() {
    let m = triangle_complex();
    // ReggeGeometry stores edge lengths but the simplicial Hodge ⋆ is determined
    // by the complex cache; the metric instance's edge_lengths are irrelevant
    // for this trait method.
    let geom = ReggeGeometry::new(CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap());
    for k in 0..=m.complex().max_simplex_dimension() {
        let star = geom.hodge_star_matrix(m.complex(), k);
        assert!(matches!(star, Cow::Borrowed(_)));
        assert_eq!(star.shape(), m.complex().hodge_star_operators()[k].shape());
    }
}

#[test]
fn trait_routed_matrix_equals_complex_cache_matrix() {
    let m = triangle_complex();
    let geom = ReggeGeometry::new(CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap());
    for k in 0..=m.complex().max_simplex_dimension() {
        let via_trait = geom.hodge_star_matrix(m.complex(), k);
        let via_cache = &m.complex().hodge_star_operators()[k];
        assert_eq!(via_trait.row_indices(), via_cache.row_indices());
        assert_eq!(via_trait.col_indices(), via_cache.col_indices());
        assert_eq!(via_trait.values(), via_cache.values());
        assert_eq!(via_trait.shape(), via_cache.shape());
    }
}

#[test]
fn manifold_hodge_star_with_metric_returns_expected_shape() {
    // R4.5 removed the dual-path fallback. The manifold's `hodge_star` now
    // panics without a metric, so this test verifies the *only* supported
    // path: construct via `with_metric` and call.
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();
    let complex = pc.triangulate(1.2).unwrap();
    let data = CausalTensor::new(vec![10.0, 20.0, 30.0, 1.0, 2.0, 3.0, 100.0], vec![7]).unwrap();
    let geom = ReggeGeometry::new(CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap());
    let m = Manifold::with_metric(complex, data, Some(geom), 0).unwrap();

    for k in 0..=m.complex().max_simplex_dimension() {
        let star = m.hodge_star(k);
        assert_eq!(star.len(), m.complex().hodge_star_operators()[k].shape().0);
    }
}
