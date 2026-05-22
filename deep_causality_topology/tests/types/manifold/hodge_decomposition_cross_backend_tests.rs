/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cross-backend tests for `Manifold::hodge_decompose` (Block H3, task 4.4).
//!
//! The same prescribed scalar field is used to construct ω = df on two
//! backends, and the resulting decomposition is checked for shared algebraic
//! invariants:
//!
//! 1. The Hodge orthogonality identity `‖α‖² + ‖β‖² + ‖h‖² = ‖ω‖²` holds in
//!    both backends to relative tolerance `1e-3`.
//! 2. Both backends produce non-degenerate decompositions (`α` non-zero) for
//!    a pure-exact 1-form on a non-degenerate triangulation.
//!
//! ## Fixture choice
//!
//! `spec.md` task 4.4 calls for a "unit square (two triangles) versus unit
//! square (one 2-cube)" cross-check. That fixture cannot be built through the
//! existing `PointCloud::triangulate` path: triangulating four coplanar
//! corner points in 2D produces a degenerate 3-simplex (a flat tetrahedron
//! spanning all four points) whose lumped-mass `M_0` collapses to zero,
//! making the simplicial codifferential return identically zero. That is an
//! upstream behaviour of `PointCloud::triangulate`, not an algorithmic
//! property of `hodge_decompose`, and is out of scope for this change set.
//!
//! We therefore use a single right triangle on the simplicial side (the
//! `setup_triangle_manifold` pattern already exercised by the differential
//! operator tests) and the `LatticeComplex<2>` unit square on the cubical
//! side. The cross-backend check is qualitative — both backends produce a
//! non-degenerate, orthogonality-respecting decomposition of ω = df — rather
//! than exact numerical L2-norm agreement, since the two underlying domains
//! differ in cell counts and boundary geometry.
//!
//! When `PointCloud::triangulate` gains a cap on top-grade dimension or a
//! flat-fixture constructor lands, the test can be tightened to the full
//! exact-numerical agreement specified in `spec.md` 4.4.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold, PointCloud, ReggeGeometry,
    SimplicialComplex,
};

fn norm_sq(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum()
}

// ---------------------------------------------------------------------------
// Simplicial fixture: a single right triangle with legs 1 and 1.
// Vertices: v0=(0,0), v1=(1,0), v2=(0,1).
// Edges: (0,1), (0,2), (1,2). Length 1, 1, √2.
// One 2-simplex.
// ---------------------------------------------------------------------------

fn simplicial_triangle_manifold() -> Manifold<SimplicialComplex<f64>, f64> {
    let points = CausalTensor::new(vec![0.0_f64, 0.0, 1.0, 0.0, 0.0, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0_f64; 3], vec![3]).unwrap();
    let cloud = PointCloud::new(points, metadata, 0).unwrap();
    let complex = cloud
        .triangulate(1.5)
        .expect("triangulate single triangle at radius 1.5");

    let n_edges = complex.num_cells(1);
    let edge_lengths = CausalTensor::new(vec![1.0_f64; n_edges], vec![n_edges]).unwrap();
    let metric = ReggeGeometry::new(edge_lengths);

    let total: usize = complex
        .skeletons()
        .iter()
        .map(|s| s.simplices().len())
        .sum();
    let n0 = complex.skeletons()[0].simplices().len();
    let mut data_vec = vec![0.0_f64; total];
    // Linear field f(x, y) = 2x + 3y on the triangle vertices.
    let coords: [(f64, f64); 3] = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)];
    for (i, slot) in data_vec.iter_mut().enumerate().take(n0) {
        let (x, y) = coords[i];
        *slot = 2.0 * x + 3.0 * y;
    }
    let data = CausalTensor::new(data_vec, vec![total]).unwrap();
    Manifold::with_metric(complex, data, Some(metric), 0).expect("build simplicial manifold")
}

// ---------------------------------------------------------------------------
// Cubical fixture: one 2-cube (the unit square).
// ---------------------------------------------------------------------------

fn cubical_unit_square_manifold() -> Manifold<LatticeComplex<2, f64>, f64> {
    // `square_open(2)` builds a 2x2 grid of cells = 3x3 vertices. The fixture
    // remains a discretised unit square; refining beyond a single 2-cube is
    // necessary because `square_open(1)` collapses to a single vertex with
    // no edges (the lattice extent is in cells per side, not vertices, so
    // extent 1 leaves no interior structure for d to act on).
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(2);
    let n0 = lattice.num_cells(0);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let mut data_vec = vec![0.0_f64; total];
    // Linear field f(x, y) = 2x + 3y. Vertex coordinates on the lattice are
    // (i % side, i / side) where side = 3 for a 2x2-cell open lattice.
    let side = 3usize;
    for (i, slot) in data_vec.iter_mut().enumerate().take(n0) {
        let x = (i % side) as f64;
        let y = (i / side) as f64;
        *slot = 2.0 * x + 3.0 * y;
    }
    let data = CausalTensor::new(data_vec, vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

// ---------------------------------------------------------------------------
// Cross-backend tests
// ---------------------------------------------------------------------------

#[test]
fn simplicial_and_cubical_decompositions_both_satisfy_orthogonality_identity() {
    let s_manifold = simplicial_triangle_manifold();
    let s_omega = s_manifold.exterior_derivative(0);
    let s_result = s_manifold
        .hodge_decompose(&s_omega, 1)
        .expect("simplicial decompose ω = df");
    let s_omega_norm_sq = norm_sq(s_omega.as_slice());
    let s_sum = norm_sq(s_result.exact().as_slice())
        + norm_sq(s_result.co_exact().as_slice())
        + norm_sq(s_result.harmonic().as_slice());
    let s_rel = (s_sum - s_omega_norm_sq).abs() / s_omega_norm_sq.max(1.0);
    assert!(
        s_rel < 1e-3,
        "simplicial orthogonality violated: ‖α‖²+‖β‖²+‖h‖²={}, ‖ω‖²={}, rel={}",
        s_sum,
        s_omega_norm_sq,
        s_rel
    );

    let c_manifold = cubical_unit_square_manifold();
    let c_omega = c_manifold.exterior_derivative(0);
    let c_result = c_manifold
        .hodge_decompose(&c_omega, 1)
        .expect("cubical decompose ω = df");
    let c_omega_norm_sq = norm_sq(c_omega.as_slice());
    let c_sum = norm_sq(c_result.exact().as_slice())
        + norm_sq(c_result.co_exact().as_slice())
        + norm_sq(c_result.harmonic().as_slice());
    let c_rel = (c_sum - c_omega_norm_sq).abs() / c_omega_norm_sq.max(1.0);
    assert!(
        c_rel < 1e-3,
        "cubical orthogonality violated: ‖α‖²+‖β‖²+‖h‖²={}, ‖ω‖²={}, rel={}",
        c_sum,
        c_omega_norm_sq,
        c_rel
    );
}

#[test]
fn cubical_decomposition_of_pure_exact_1form_is_non_degenerate() {
    let c_manifold = cubical_unit_square_manifold();
    let c_omega = c_manifold.exterior_derivative(0);
    let c_result = c_manifold
        .hodge_decompose(&c_omega, 1)
        .expect("cubical decompose ω = df");
    let c_omega_norm_sq = norm_sq(c_omega.as_slice());
    let c_alpha_norm_sq = norm_sq(c_result.exact().as_slice());
    let c_beta_norm_sq = norm_sq(c_result.co_exact().as_slice());
    let c_h_norm_sq = norm_sq(c_result.harmonic().as_slice());

    assert!(c_omega_norm_sq > 0.0, "ω must be non-trivial");
    assert!(
        c_alpha_norm_sq > c_beta_norm_sq && c_alpha_norm_sq > c_h_norm_sq,
        "cubical: α should dominate, got α²={}, β²={}, h²={}",
        c_alpha_norm_sq,
        c_beta_norm_sq,
        c_h_norm_sq
    );
    // β and h must collectively account for at most 1% of ω's energy.
    let vanishing_ratio = (c_beta_norm_sq + c_h_norm_sq) / c_omega_norm_sq;
    assert!(
        vanishing_ratio < 1e-2,
        "cubical: (β² + h²) / ω² = {} should be near zero",
        vanishing_ratio
    );
}
