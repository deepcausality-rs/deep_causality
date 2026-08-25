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

// ---------------------------------------------------------------------------
// The mass matrices are read as diagonals, and the operators degrade to zero
// rather than panicking when a matrix and a field disagree on length.
// ---------------------------------------------------------------------------

/// A triangle whose Hodge stars are supplied verbatim, so a test can put an
/// off-diagonal entry and a missing diagonal into `⋆₀` and observe how the
/// codifferential's per-row mass lookup treats each.
fn triangle_with_handmade_star_zero(
    star_zero: deep_causality_linear::CsrMatrix<f64>,
) -> SimplicialManifold<f64, f64> {
    use deep_causality_linear::CsrMatrix;
    use deep_causality_topology::{Simplex, SimplicialComplex, Skeleton};

    let sk0 = Skeleton::new(
        0,
        vec![
            Simplex::new(vec![0]),
            Simplex::new(vec![1]),
            Simplex::new(vec![2]),
        ],
    );
    let sk1 = Skeleton::new(
        1,
        vec![
            Simplex::new(vec![0, 1]),
            Simplex::new(vec![0, 2]),
            Simplex::new(vec![1, 2]),
        ],
    );
    let sk2 = Skeleton::new(2, vec![Simplex::new(vec![0, 1, 2])]);

    let d1 = CsrMatrix::from_triplets(
        3,
        3,
        &[
            (0, 0, -1i8),
            (1, 0, 1),
            (0, 1, -1),
            (2, 1, 1),
            (1, 2, -1),
            (2, 2, 1),
        ],
    )
    .unwrap();
    let d2 = CsrMatrix::from_triplets(3, 1, &[(0, 0, 1i8), (1, 0, -1), (2, 0, 1)]).unwrap();
    let cob = vec![d1.transpose(), d2.transpose()];

    let star_one =
        CsrMatrix::from_triplets(3, 3, &[(0, 0, 1.0f64), (1, 1, 1.0), (2, 2, 1.0)]).unwrap();
    let star_two = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1.0f64)]).unwrap();

    let complex = SimplicialComplex::new(
        vec![sk0, sk1, sk2],
        vec![d1, d2],
        cob,
        vec![star_zero, star_one, star_two],
    );
    let regge = ReggeGeometry::new(CausalTensor::new(vec![1.0f64; 3], vec![3]).unwrap());
    let data = CausalTensor::new(vec![0.0f64; 7], vec![7]).unwrap();
    Manifold::with_metric(complex, data, Some(regge), 0).unwrap()
}

/// `δ` divides row `i` by the `(i, i)` entry of `⋆_{k-1}`. A row that stores an
/// off-diagonal entry ahead of its diagonal must be scanned past, not read at
/// its first entry; a row that stores no diagonal at all has no mass to divide
/// by and yields zero.
///
/// `⋆₀` here is `(0,0) = 1`, row 1 = `[(0, 0.5), (1, 2.0)]`, row 2 empty, and
/// `⋆₁` is the identity, so `δω = ⋆₀⁻¹ ∂₁ ω` on `ω = (1, 2, 3)` is
/// `(-3/1, -2/2, 0) = (-3, -1, 0)`.
#[test]
fn codifferential_scans_past_off_diagonal_mass_and_zeroes_a_massless_row() {
    use deep_causality_linear::CsrMatrix;

    let star_zero = CsrMatrix::from_triplets(
        3,
        3,
        &[(0, 0, 1.0f64), (1, 0, 0.5), (1, 1, 2.0)], // row 2 carries no entry at all
    )
    .unwrap();
    let m = triangle_with_handmade_star_zero(star_zero);

    let out = m.codifferential_of(&[1.0, 2.0, 3.0], 1);
    assert_eq!(out.as_slice(), [-3.0f64, -1.0, 0.0].as_slice());
}

/// `d` is sized by the (k+1)-skeleton, not by the stored coboundary matrix. With
/// no coboundary operator supplied the matvec has nothing to contract against
/// and the result is padded out to the edge count as the zero 1-form.
#[test]
fn exterior_derivative_pads_to_the_next_skeleton_when_the_operator_is_absent() {
    use deep_causality_topology::utils_tests::create_triangle_complex;

    let complex = create_triangle_complex();
    let data = CausalTensor::new(vec![0.0f64; 7], vec![7]).unwrap();
    let m: SimplicialManifold<f64, f64> = Manifold::new(complex, data, 0).unwrap();

    let out = m.exterior_derivative_of(&[1.0, 2.0, 3.0], 0);
    assert_eq!(out.shape(), vec![3], "one coefficient per 1-simplex");
    assert_eq!(out.as_slice(), [0.0f64, 0.0, 0.0].as_slice());
}

/// `δ` on a k-form whose length does not match the k-cell count contracts
/// nothing and returns the zero (k-1)-form of the correct length.
#[test]
fn codifferential_of_a_wrong_length_form_is_the_zero_form() {
    let m = triangle_with_metric();
    let n0 = m.complex().num_cells(0);

    let out = m.codifferential_of(&[1.0], 1);
    assert_eq!(out.shape(), vec![n0]);
    assert!(out.as_slice().iter().all(|&x| x == 0.0));
}

/// The per-grade view of the manifold's data slab reads as zero when the slab is
/// shorter than the grade needs, so `d` of the stored 0-form is the zero 1-form
/// rather than an out-of-bounds read.
#[test]
fn stored_form_of_a_too_short_data_slab_reads_as_zero() {
    let lattice = LatticeComplex::<2, f64>::new([3, 3], [false, false]);
    let n1 = lattice.num_cells(1);
    // One value for a lattice that needs nine at grade 0 alone.
    let data = CausalTensor::new(vec![7.0f64], vec![1]).unwrap();
    let m = Manifold::from_cubical(lattice, data, 0);

    let out = m.exterior_derivative(0);
    assert_eq!(out.shape(), vec![n1]);
    assert!(out.as_slice().iter().all(|&x| x == 0.0));
}
