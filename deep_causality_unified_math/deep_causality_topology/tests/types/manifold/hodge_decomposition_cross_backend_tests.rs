/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cross-backend tests for `Manifold::hodge_decompose`.
//!
//! The same prescribed scalar field `f(x, y) = 2x + 3y` is decomposed twice:
//!
//! 1. Simplicially: a `SimplicialComplex<f64>` built by
//!    `PointCloud::triangulate_delaunay` on the four corners of the unit
//!    square. Four vertices, five edges (4 sides + 1 diagonal), two
//!    triangles. The output is manifold-respecting by construction.
//! 2. Cubically: a `LatticeComplex<2, f64>` of 2×2 cells (3×3 vertices),
//!    with `CubicalReggeGeometry<2, f64, Euclidean>` at unit edge.
//!
//! ## Scenarios checked
//!
//! - **Strict per-component fractional agreement** (the primary assertion
//!   added by `add-pointcloud-delaunay-triangulation` D2): for each
//!   component `c ∈ {exact, co_exact, harmonic}`, the fraction
//!   `‖c‖² / ‖ω‖²` agrees across backends at `1e-6`. This uses normalized
//!   fractions rather than raw L2 norms because the two discretizations
//!   have different edge counts (simplicial: 5; cubical: 12), so raw
//!   `‖α‖` differs by O(1) for any non-trivial ω — see spec.md MODIFIED
//!   Requirement "Two-backend cross-check on the unit square" for the
//!   reasoning behind the strict-fraction framing.
//! - **Per-component vanishing on each backend individually**:
//!   `‖β‖² / ‖ω‖²` and `‖h‖² / ‖ω‖²` are each < 1e-6 on both sides.
//! - **Relaxed scenarios** (defence-in-depth from `add-hodge-decomposition`
//!   H3): orthogonality identity per backend; summed-vanishing-ratio
//!   cross-backend agreement.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold, PointCloud, ReggeGeometry,
    SimplicialComplex,
};

fn norm_sq(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum()
}

// ---------------------------------------------------------------------------
// Simplicial fixture: the canonical two-triangle Delaunay unit square.
// ---------------------------------------------------------------------------

fn simplicial_unit_square_manifold() -> Manifold<SimplicialComplex<f64>, f64> {
    let coords: [(f64, f64); 4] = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];

    let flat: Vec<f64> = coords.iter().flat_map(|&(x, y)| [x, y]).collect();
    let points = CausalTensor::new(flat, vec![4, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0_f64; 4], vec![4]).unwrap();
    let cloud = PointCloud::new(points, metadata, 0).unwrap();
    let complex = cloud
        .triangulate_delaunay()
        .expect("Delaunay triangulation of the unit square");

    // Sanity: the spec mandates 4 vertices, 5 edges, 2 triangles for this fixture.
    assert_eq!(complex.skeletons()[0].simplices().len(), 4);
    assert_eq!(complex.skeletons()[1].simplices().len(), 5);
    assert_eq!(complex.skeletons()[2].simplices().len(), 2);

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
// Cubical fixture: a 2×2 lattice (the unit square at single-cell refinement).
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
// Helper: full-decomposition computation reused across scenarios.
// ---------------------------------------------------------------------------

struct DecompositionStats {
    omega_norm_sq: f64,
    alpha_norm_sq: f64,
    beta_norm_sq: f64,
    h_norm_sq: f64,
}

fn simplicial_stats() -> DecompositionStats {
    let m = simplicial_unit_square_manifold();
    let omega = m.exterior_derivative(0);
    let result = m
        .hodge_decompose(&omega, 1)
        .expect("simplicial decompose ω = df");
    DecompositionStats {
        omega_norm_sq: norm_sq(omega.as_slice()),
        alpha_norm_sq: norm_sq(result.exact().as_slice()),
        beta_norm_sq: norm_sq(result.co_exact().as_slice()),
        h_norm_sq: norm_sq(result.harmonic().as_slice()),
    }
}

fn cubical_stats() -> DecompositionStats {
    let m = cubical_unit_square_manifold();
    let omega = m.exterior_derivative(0);
    let result = m
        .hodge_decompose(&omega, 1)
        .expect("cubical decompose ω = df");
    DecompositionStats {
        omega_norm_sq: norm_sq(omega.as_slice()),
        alpha_norm_sq: norm_sq(result.exact().as_slice()),
        beta_norm_sq: norm_sq(result.co_exact().as_slice()),
        h_norm_sq: norm_sq(result.harmonic().as_slice()),
    }
}

// ---------------------------------------------------------------------------
// Primary D2 assertion: per-component fractional agreement at 1e-6.
// ---------------------------------------------------------------------------

#[test]
fn simplicial_and_cubical_per_component_fractions_agree_at_1e6() {
    let s = simplicial_stats();
    let c = cubical_stats();

    let alpha_frac_s = s.alpha_norm_sq / s.omega_norm_sq;
    let alpha_frac_c = c.alpha_norm_sq / c.omega_norm_sq;
    let beta_frac_s = s.beta_norm_sq / s.omega_norm_sq;
    let beta_frac_c = c.beta_norm_sq / c.omega_norm_sq;
    let h_frac_s = s.h_norm_sq / s.omega_norm_sq;
    let h_frac_c = c.h_norm_sq / c.omega_norm_sq;

    let alpha_diff = (alpha_frac_s - alpha_frac_c).abs();
    let beta_diff = (beta_frac_s - beta_frac_c).abs();
    let h_diff = (h_frac_s - h_frac_c).abs();

    assert!(
        alpha_diff < 1e-6,
        "exact-component fraction disagreement {} exceeds 1e-6 (simplicial: {}, cubical: {})",
        alpha_diff,
        alpha_frac_s,
        alpha_frac_c
    );
    assert!(
        beta_diff < 1e-6,
        "co_exact-component fraction disagreement {} exceeds 1e-6 (simplicial: {}, cubical: {})",
        beta_diff,
        beta_frac_s,
        beta_frac_c
    );
    assert!(
        h_diff < 1e-6,
        "harmonic-component fraction disagreement {} exceeds 1e-6 (simplicial: {}, cubical: {})",
        h_diff,
        h_frac_s,
        h_frac_c
    );
}

// ---------------------------------------------------------------------------
// Per-component vanishing on each backend individually.
// ---------------------------------------------------------------------------

#[test]
fn simplicial_per_component_vanishing_holds() {
    let s = simplicial_stats();
    let beta_frac = s.beta_norm_sq / s.omega_norm_sq;
    let h_frac = s.h_norm_sq / s.omega_norm_sq;
    assert!(
        beta_frac < 1e-6,
        "simplicial: β² / ω² = {} should be < 1e-6",
        beta_frac
    );
    assert!(
        h_frac < 1e-6,
        "simplicial: h² / ω² = {} should be < 1e-6",
        h_frac
    );
}

#[test]
fn cubical_per_component_vanishing_holds() {
    let c = cubical_stats();
    let beta_frac = c.beta_norm_sq / c.omega_norm_sq;
    let h_frac = c.h_norm_sq / c.omega_norm_sq;
    assert!(
        beta_frac < 1e-6,
        "cubical: β² / ω² = {} should be < 1e-6",
        beta_frac
    );
    assert!(
        h_frac < 1e-6,
        "cubical: h² / ω² = {} should be < 1e-6",
        h_frac
    );
}

// ---------------------------------------------------------------------------
// Defence-in-depth: the relaxed scenarios from add-hodge-decomposition H3.
// ---------------------------------------------------------------------------

#[test]
fn simplicial_and_cubical_decompositions_both_satisfy_orthogonality_identity() {
    let s = simplicial_stats();
    let s_sum = s.alpha_norm_sq + s.beta_norm_sq + s.h_norm_sq;
    let s_rel = (s_sum - s.omega_norm_sq).abs() / s.omega_norm_sq.max(1.0);
    assert!(
        s_rel < 1e-6,
        "simplicial orthogonality violated: ‖α‖²+‖β‖²+‖h‖²={}, ‖ω‖²={}, rel={}",
        s_sum,
        s.omega_norm_sq,
        s_rel
    );

    let c = cubical_stats();
    let c_sum = c.alpha_norm_sq + c.beta_norm_sq + c.h_norm_sq;
    let c_rel = (c_sum - c.omega_norm_sq).abs() / c.omega_norm_sq.max(1.0);
    assert!(
        c_rel < 1e-6,
        "cubical orthogonality violated: ‖α‖²+‖β‖²+‖h‖²={}, ‖ω‖²={}, rel={}",
        c_sum,
        c.omega_norm_sq,
        c_rel
    );
}

#[test]
fn simplicial_and_cubical_decompositions_agree_on_vanishing_component_structure() {
    let s = simplicial_stats();
    let c = cubical_stats();

    let s_vanishing = (s.beta_norm_sq + s.h_norm_sq) / s.omega_norm_sq;
    let c_vanishing = (c.beta_norm_sq + c.h_norm_sq) / c.omega_norm_sq;
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

    let agreement_diff = (s_vanishing - c_vanishing).abs();
    assert!(
        agreement_diff < 1e-6,
        "cross-backend vanishing-ratio disagreement {} exceeds 1e-6 (simplicial: {}, cubical: {})",
        agreement_diff,
        s_vanishing,
        c_vanishing
    );
}

#[test]
fn cubical_decomposition_of_pure_exact_1form_is_non_degenerate() {
    let c = cubical_stats();
    assert!(c.omega_norm_sq > 0.0, "ω must be non-trivial");
    assert!(
        c.alpha_norm_sq > c.beta_norm_sq && c.alpha_norm_sq > c.h_norm_sq,
        "cubical: α should dominate, got α²={}, β²={}, h²={}",
        c.alpha_norm_sq,
        c.beta_norm_sq,
        c.h_norm_sq
    );
}

#[test]
fn simplicial_decomposition_of_pure_exact_1form_is_non_degenerate() {
    let s = simplicial_stats();
    assert!(s.omega_norm_sq > 0.0, "ω must be non-trivial");
    assert!(
        s.alpha_norm_sq > s.beta_norm_sq && s.alpha_norm_sq > s.h_norm_sq,
        "simplicial: α should dominate, got α²={}, β²={}, h²={}",
        s.alpha_norm_sq,
        s.beta_norm_sq,
        s.h_norm_sq
    );
}
