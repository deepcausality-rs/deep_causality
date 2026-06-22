/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage tests for the differential-operator edge branches that the broad
//! property suites do not pin precisely:
//!
//! * `codifferential.rs`: the `k == 0` early return and the per-row zero-mass
//!   guard inside the generic codifferential mass loop.
//! * `exterior.rs`: the highest-grade `d` early return.
//! * `de_rham.rs`: the `sharp` per-axis `count == 0` fallback (a degenerate
//!   extent-1 lattice axis carries no edges, so every vertex averages over an
//!   empty incident set and the component collapses to zero).
//! * `interior_product.rs`: the `k == 0 || k > D` grade guard, the operand
//!   length mismatches, and the missing-metric error.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold, ReggeGeometry,
    SimplicialManifold, TopologyErrorEnum,
};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn cubical_unit_2d(
    shape: [usize; 2],
    periodic: [bool; 2],
) -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice = LatticeComplex::<2, f64>::new(shape, periodic);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    Manifold::from_cubical_with_metric(lattice, data, CubicalReggeGeometry::unit(), 0)
}

/// A metric-bearing simplicial triangle manifold.
fn triangle_with_metric() -> SimplicialManifold<f64, f64> {
    use deep_causality_topology::PointCloud;
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();
    let complex = pc.triangulate(1.5).unwrap();
    let skeleton1 = complex.skeletons()[1].clone();
    let edge_lengths = vec![1.0_f64; skeleton1.simplices().len()];
    let regge = ReggeGeometry::new(
        CausalTensor::new(edge_lengths, vec![skeleton1.simplices().len()]).unwrap(),
    );
    let total = complex.total_simplices();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    Manifold::with_metric(complex, data, Some(regge), 0).unwrap()
}

// ---------------------------------------------------------------------------
// codifferential.rs:52 — k == 0 early return (the empty (k-1)-form)
// ---------------------------------------------------------------------------

#[test]
fn codifferential_of_grade_zero_is_empty() {
    let m = triangle_with_metric();
    let out = m.codifferential_of(&[1.0, 2.0, 3.0], 0);
    assert_eq!(out.len(), 0, "delta of a 0-form is the empty (-1)-form");
}

// ---------------------------------------------------------------------------
// codifferential.rs:95,101,102 — the per-row mass loop (break on the diagonal
// match, the `mass_val.abs() > tol` accept, and the zero-mass else branch).
// A standard metric exercises the accept path; the loop and break are walked
// for every (k-1)-cell.
// ---------------------------------------------------------------------------

#[test]
fn codifferential_of_grade_one_walks_mass_loop() {
    let m = cubical_unit_2d([4, 4], [true, true]);
    let n1 = m.complex().num_cells(1);
    let field: Vec<f64> = (0..n1).map(|i| (i as f64).sin()).collect();
    let out = m.codifferential_of(&field, 1);
    assert_eq!(out.len(), m.complex().num_cells(0));
}

/// The generic (non-cubical) codifferential mass loop: a simplicial manifold
/// with a Regge metric routes through `codifferential.rs`'s per-row diagonal
/// search + inverse-mass weighting (the `break` and the accept branch).
#[test]
fn codifferential_of_grade_one_on_simplicial_manifold() {
    let m = triangle_with_metric();
    let n1 = m.complex().num_cells(1);
    let field: Vec<f64> = (0..n1).map(|i| 1.0 + i as f64).collect();
    let out = m.codifferential_of(&field, 1);
    assert_eq!(out.len(), m.complex().num_cells(0));
}

// ---------------------------------------------------------------------------
// exterior.rs — the highest-grade `d` early return (k >= max_dim → empty).
// ---------------------------------------------------------------------------

#[test]
fn exterior_derivative_at_top_grade_is_empty() {
    let m = cubical_unit_2d([4, 4], [true, true]);
    // On a 2D lattice the top grade is 2; d of a 2-form is empty.
    let n2 = m.complex().num_cells(2);
    let top = vec![1.0_f64; n2];
    let out = m.exterior_derivative_of(&top, 2);
    assert_eq!(out.len(), 0, "d of the top-grade form is zero");
}

/// `d` on a simplicial manifold (vertices → edges) walks the generic coboundary
/// matvec and the result-size normalization.
#[test]
fn exterior_derivative_on_simplicial_manifold() {
    let m = triangle_with_metric();
    let n0 = m.complex().num_cells(0);
    let field: Vec<f64> = (0..n0).map(|i| i as f64).collect();
    let out = m.exterior_derivative_of(&field, 0);
    assert_eq!(out.len(), m.complex().num_cells(1));
}

// ---------------------------------------------------------------------------
// de_rham.rs:225 — sharp per-axis count == 0 fallback on a degenerate
// extent-1 axis (no edges along that axis → every vertex averages an empty
// incident set → the component is R::zero()).
// ---------------------------------------------------------------------------

#[test]
fn sharp_on_extent_one_axis_yields_zero_component() {
    // Axis 1 has extent 1, so it carries no 1-cells: the y component of the
    // vector proxy is exactly zero at every vertex.
    let lattice = LatticeComplex::<2, f64>::new([4, 1], [false, false]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let n1 = lattice.num_cells(1);
    let n0 = lattice.num_cells(0);
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let m = Manifold::from_cubical_with_metric(lattice, data, CubicalReggeGeometry::unit(), 0);

    let edge = CausalTensor::new(vec![1.0_f64; n1], vec![n1]).unwrap();
    let sharp = m.sharp(&edge).unwrap();

    // Layout is vertex * D + axis; the axis-1 (y) slot of every vertex is zero.
    assert_eq!(sharp.len(), n0 * 2);
    for v in 0..n0 {
        let y = sharp.as_slice()[v * 2 + 1];
        assert_eq!(
            y, 0.0,
            "extent-1 axis has no edges, so its component is zero"
        );
    }
}

// ---------------------------------------------------------------------------
// interior_product.rs — the grade guard, length mismatches, and missing metric.
// ---------------------------------------------------------------------------

#[test]
fn interior_product_rejects_grade_zero() {
    let m = cubical_unit_2d([4, 4], [true, true]);
    let n1 = m.complex().num_cells(1);
    let x = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let omega = CausalTensor::new(
        vec![0.0; m.complex().num_cells(0)],
        vec![m.complex().num_cells(0)],
    )
    .unwrap();
    let err = m.interior_product(&x, &omega, 0).unwrap_err();
    assert!(matches!(err.0, TopologyErrorEnum::InvalidGradeOperation(_)));
}

#[test]
fn interior_product_rejects_grade_above_dimension() {
    let m = cubical_unit_2d([4, 4], [true, true]);
    let n1 = m.complex().num_cells(1);
    let x = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let omega = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    // D = 2, so k = 3 is out of range.
    let err = m.interior_product(&x, &omega, 3).unwrap_err();
    assert!(matches!(err.0, TopologyErrorEnum::InvalidGradeOperation(_)));
}

#[test]
fn interior_product_rejects_wrong_contraction_field_length() {
    let m = cubical_unit_2d([4, 4], [true, true]);
    let n2 = m.complex().num_cells(2);
    let bad_x = CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(); // not num_cells(1)
    let omega = CausalTensor::new(vec![0.0; n2], vec![n2]).unwrap();
    let err = m.interior_product(&bad_x, &omega, 2).unwrap_err();
    assert!(matches!(err.0, TopologyErrorEnum::DimensionMismatch(_)));
}

#[test]
fn interior_product_rejects_wrong_form_operand_length() {
    let m = cubical_unit_2d([4, 4], [true, true]);
    let n1 = m.complex().num_cells(1);
    let x = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let bad_omega = CausalTensor::new(vec![0.0; 2], vec![2]).unwrap(); // not num_cells(2)
    let err = m.interior_product(&x, &bad_omega, 2).unwrap_err();
    assert!(matches!(err.0, TopologyErrorEnum::DimensionMismatch(_)));
}

#[test]
fn interior_product_without_metric_is_rejected() {
    let lattice = LatticeComplex::<2, f64>::new([4, 4], [true, true]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let n1 = lattice.num_cells(1);
    let n2 = lattice.num_cells(2);
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let m = Manifold::from_cubical(lattice, data, 0); // no metric

    let x = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let omega = CausalTensor::new(vec![0.0; n2], vec![n2]).unwrap();
    let err = m.interior_product(&x, &omega, 2).unwrap_err();
    assert!(matches!(err.0, TopologyErrorEnum::InvalidInput(_)));
}
