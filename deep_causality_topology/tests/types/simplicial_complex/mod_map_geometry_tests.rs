/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests targeting the lazy-Hodge / geometric-data branches of
//! `SimplicialComplex`:
//!
//! - `SimplicialComplex::map` cloning a populated Hodge ⋆ cache and the
//!   `geometric_data` coordinates (src/types/simplicial_complex/mod.rs).
//! - `SimplicialComplex::new` pre-populating the Hodge ⋆ cache from a non-empty
//!   operator vector (src/types/simplicial_complex/constructors/constructors_impl.rs).
//! - `SimplicialTopology::max_simplex_dimension` (trait method, distinct from the
//!   inherent getter), exercised through an explicit fully-qualified call.

use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    PointCloud, Simplex, SimplicialComplex, SimplicialTopology, Skeleton,
};

fn triangulated_complex() -> SimplicialComplex<f64> {
    // A non-degenerate triangle in 2D; `triangulate` builds via `with_geometry`
    // so coordinates are retained for the lazy Hodge ⋆ build.
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();
    pc.triangulate(1.1).unwrap()
}

#[test]
fn test_map_clones_populated_hodge_cache_and_geometry() {
    let complex = triangulated_complex();

    // Force the lazy Hodge ⋆ build so the OnceLock cache is populated AND
    // geometric_data is present. This drives the `if let Some(hodge_ops)` and
    // `geometric_data.map(...)` branches inside `SimplicialComplex::map`.
    let hodge_before = complex
        .hodge_star_operators()
        .expect("non-degenerate triangle should build Hodge ⋆")
        .len();
    assert!(hodge_before > 0);

    // Map f64 -> f32. The populated cache and the coordinate slab must both be
    // remapped through the closure.
    let mapped: SimplicialComplex<f32> = complex.map(|x| x as f32);

    // The mapped complex still exposes its Hodge ⋆ surface (cache carried over
    // and remapped), and the count matches the original.
    let hodge_after = mapped
        .hodge_star_operators()
        .expect("mapped complex retains Hodge ⋆ cache")
        .len();
    assert_eq!(hodge_before, hodge_after);
}

#[test]
fn test_new_prepopulates_hodge_cache_from_nonempty_vector() {
    // Single edge: two vertices, one 1-simplex.
    let vertices = vec![Simplex::new(vec![0]), Simplex::new(vec![1])];
    let edges = vec![Simplex::new(vec![0, 1])];
    let skeletons = vec![Skeleton::new(0, vertices), Skeleton::new(1, edges)];
    let d1: CsrMatrix<i8> = CsrMatrix::from_triplets(2, 1, &[(1, 0, 1i8), (0, 0, -1)]).unwrap();

    // Non-empty Hodge ⋆ vector: a 2x2 diagonal for grade 0 and a 1x1 for grade 1.
    let star0: CsrMatrix<f64> =
        CsrMatrix::from_triplets(2, 2, &[(0, 0, 0.5f64), (1, 1, 0.5)]).unwrap();
    let star1: CsrMatrix<f64> = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1.0f64)]).unwrap();

    let complex: SimplicialComplex<f64> =
        SimplicialComplex::new(skeletons, vec![d1], Vec::new(), vec![star0, star1]);

    // Pre-supplied operators populate the cache directly (constructors_impl
    // `cell.set(...)` branch); the accessor returns them unchanged.
    let ops = complex
        .hodge_star_operators()
        .expect("pre-supplied Hodge ⋆ must be returned verbatim");
    assert_eq!(ops.len(), 2);
    assert_eq!(ops[0].shape(), (2, 2));
    assert_eq!(ops[1].shape(), (1, 1));
}

#[test]
fn test_simplicial_topology_trait_max_dimension_via_fully_qualified_call() {
    let complex = triangulated_complex();
    // The inherent `max_simplex_dimension` getter shadows the trait method at
    // the call site, so we invoke the trait method explicitly to exercise the
    // `SimplicialTopology` impl body.
    let dim = <SimplicialComplex<f64> as SimplicialTopology>::max_simplex_dimension(&complex);
    assert_eq!(dim, 2);
}

#[test]
fn test_simplicial_topology_trait_max_dimension_empty() {
    // Empty complex exercises the `unwrap_or(0)` fallback of the trait method.
    let complex: SimplicialComplex<f64> = SimplicialComplex::default();
    let dim = <SimplicialComplex<f64> as SimplicialTopology>::max_simplex_dimension(&complex);
    assert_eq!(dim, 0);
}
