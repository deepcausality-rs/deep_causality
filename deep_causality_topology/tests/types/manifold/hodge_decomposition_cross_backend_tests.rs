/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cross-backend tests for `Manifold::hodge_decompose` (Block H3, task 4.4).
//!
//! The same prescribed scalar field f(x, y) = 2x + 3y is decomposed twice:
//!
//! 1. Simplicially: a `SimplicialComplex` built by `PointCloud::triangulate`
//!    from a right triangle (0,0), (1,0), (0,1) at radius 1.5. Three vertices,
//!    three edges, one 2-simplex.
//! 2. Cubically: a `LatticeComplex<2, f64>` of 2x2 cells = 3x3 vertices, with
//!    `CubicalReggeGeometry<2, f64, Euclidean>` at unit edge.
//!
//! ## Why a right triangle rather than the full unit square
//!
//! `spec.md` task 4.4 ideally calls for "unit square (two triangles) versus
//! unit square (one 2-cube)". Building the two-triangle simplicial fixture
//! requires a manifold-respecting triangulation algorithm:
//! `PointCloud::triangulate` is Vietoris-Rips — it builds every clique of
//! every grade — so on the four coplanar corners of the unit square at
//! radius 1.5 it produces a complex with 6 edges (4 sides + 2 diagonals) and
//! 4 overlapping triangles (every 3-clique), not the 5 edges + 2 triangles
//! of a Delaunay triangulation. The 4-triangle Vietoris-Rips complex fails
//! `Manifold::with_metric`'s manifold-property check because four triangles
//! sharing pairwise interiors do not form a 2-manifold.
//!
//! The previous limitation (degenerate 3-simplex collapsing lumped-mass M_0)
//! was already addressed by the ambient-dimension cap on `triangulate`. The
//! remaining gap — Delaunay (or other manifold-respecting) triangulation —
//! is a separate, larger upstream change in `PointCloud`. Until that lands,
//! the cross-backend fixture stays at the right-triangle vs unit-square
//! comparison, with the same algebraic invariants checked on both sides.

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
// Vertices: v0=(0,0), v1=(1,0), v2=(0,1). Three edges, one 2-simplex.
// ---------------------------------------------------------------------------

fn simplicial_triangle_manifold() -> Manifold<SimplicialComplex<f64>, f64> {
    let points = CausalTensor::new(vec![0.0_f64, 0.0, 1.0, 0.0, 0.0, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0_f64; 3], vec![3]).unwrap();
    let cloud = PointCloud::new(points, metadata, 0).unwrap();
    let complex = cloud
        .triangulate(1.5)
        .expect("triangulate single triangle at radius 1.5");

    let coords: [(f64, f64); 3] = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)];
    let skeletons = complex.skeletons();
    let n_edges = skeletons[1].simplices().len();
    let mut edge_lengths_vec = Vec::with_capacity(n_edges);
    for simplex in skeletons[1].simplices() {
        let v = simplex.vertices();
        let (xa, ya) = coords[v[0]];
        let (xb, yb) = coords[v[1]];
        let length = ((xb - xa).powi(2) + (yb - ya).powi(2)).sqrt();
        edge_lengths_vec.push(length);
    }
    let edge_lengths = CausalTensor::new(edge_lengths_vec, vec![n_edges]).unwrap();
    let metric = ReggeGeometry::new(edge_lengths);

    let total: usize = skeletons.iter().map(|s| s.simplices().len()).sum();
    let n0 = skeletons[0].simplices().len();
    let mut data_vec = vec![0.0_f64; total];
    for (i, slot) in data_vec.iter_mut().enumerate().take(n0) {
        let (x, y) = coords[i];
        *slot = 2.0 * x + 3.0 * y;
    }
    let data = CausalTensor::new(data_vec, vec![total]).unwrap();
    Manifold::with_metric(complex, data, Some(metric), 0).expect("build simplicial manifold")
}

// ---------------------------------------------------------------------------
// Cubical fixture: a 2x2 lattice (the unit square at single-cell refinement).
// ---------------------------------------------------------------------------

fn cubical_unit_square_manifold() -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(2);
    let n0 = lattice.num_cells(0);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let mut data_vec = vec![0.0_f64; total];
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
        s_rel < 1e-6,
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
        c_rel < 1e-6,
        "cubical orthogonality violated: ‖α‖²+‖β‖²+‖h‖²={}, ‖ω‖²={}, rel={}",
        c_sum,
        c_omega_norm_sq,
        c_rel
    );
}

#[test]
fn simplicial_and_cubical_decompositions_agree_on_vanishing_component_structure() {
    let s_manifold = simplicial_triangle_manifold();
    let s_omega = s_manifold.exterior_derivative(0);
    let s_result = s_manifold
        .hodge_decompose(&s_omega, 1)
        .expect("simplicial decompose ω = df");
    let s_omega_n = norm_sq(s_omega.as_slice());
    let s_vanishing = (norm_sq(s_result.co_exact().as_slice())
        + norm_sq(s_result.harmonic().as_slice()))
        / s_omega_n;

    let c_manifold = cubical_unit_square_manifold();
    let c_omega = c_manifold.exterior_derivative(0);
    let c_result = c_manifold
        .hodge_decompose(&c_omega, 1)
        .expect("cubical decompose ω = df");
    let c_omega_n = norm_sq(c_omega.as_slice());
    let c_vanishing = (norm_sq(c_result.co_exact().as_slice())
        + norm_sq(c_result.harmonic().as_slice()))
        / c_omega_n;

    // For ω = df on a non-degenerate domain, both backends must report the
    // co-exact and harmonic components as effectively zero relative to ω².
    assert!(
        s_vanishing < 1e-6,
        "simplicial: (β² + h²) / ω² = {} should be < 1e-6",
        s_vanishing
    );
    assert!(
        c_vanishing < 1e-6,
        "cubical: (β² + h²) / ω² = {} should be < 1e-6",
        c_vanishing
    );

    // Cross-backend agreement on the vanishing ratio to spec.md 4.4 tolerance.
    let agreement_diff = (s_vanishing - c_vanishing).abs();
    assert!(
        agreement_diff < 1e-6,
        "cross-backend vanishing-ratio disagreement {} exceeds spec.md 4.4 tolerance 1e-6 \
         (simplicial: {}, cubical: {})",
        agreement_diff,
        s_vanishing,
        c_vanishing
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
    let vanishing_ratio = (c_beta_norm_sq + c_h_norm_sq) / c_omega_norm_sq;
    assert!(
        vanishing_ratio < 1e-6,
        "cubical: (β² + h²) / ω² = {} should be < 1e-6",
        vanishing_ratio
    );
}
