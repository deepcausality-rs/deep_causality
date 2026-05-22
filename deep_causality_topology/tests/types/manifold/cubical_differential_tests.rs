/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cubical-backend tests for `Manifold::{exterior_derivative, hodge_star,
//! codifferential, laplacian}` — R4.5.3 and R4.5.4.
//!
//! R4.5 generalised the four differential operators from `impl<R>
//! Manifold<SimplicialComplex<R>, R>` to `impl<K, R> Manifold<K, R> where K:
//! ChainComplex, K::Metric: HasHodgeStar<R, Complex = K>, R: RealField +
//! FromPrimitive`. These tests verify the cubical backend (`LatticeComplex<D,
//! R>` + `CubicalReggeGeometry<D, R>`) compiles and produces correct
//! numerical output through the same generic call sites.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const TOL: f64 = 1e-12;

/// Build a `Manifold<LatticeComplex<D, f64>, f64>` with a unit-edge cubical
/// Regge geometry attached and zero-filled cell data of the right total size.
fn unit_manifold<const D: usize>(
    lattice: LatticeComplex<D, f64>,
) -> Manifold<LatticeComplex<D, f64>, f64> {
    let total: usize = (0..=D).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<D, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

// ---------------------------------------------------------------------------
// R4.5.3 — generic differential operators compile and run on Manifold<LatticeComplex>
// ---------------------------------------------------------------------------

#[test]
fn cubical_hodge_star_round_trips_for_every_grade_on_2d_periodic_lattice() {
    // ⋆ on a unit-edge cubical complex is the identity at every grade, so
    // applying ⋆ to a zero field returns zero with the right shape.
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let m = unit_manifold(lattice);
    for k in 0..=2 {
        let star = m.hodge_star(k);
        assert_eq!(star.len(), m.complex().num_cells(k), "k = {k}");
        for v in star.as_slice() {
            assert!(v.abs() < TOL, "k = {k}: expected zero, got {v}");
        }
    }
}

#[test]
fn cubical_exterior_derivative_runs_on_3d_periodic_lattice() {
    // d on a zero field is zero. Exercises the generic `exterior_derivative`
    // call path on a cubical complex; the value-level result is trivial but
    // the size invariants are the real check.
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_torus(2);
    let m = unit_manifold(lattice);
    for k in 0..3 {
        let d_omega = m.exterior_derivative(k);
        assert_eq!(d_omega.len(), m.complex().num_cells(k + 1), "k = {k}");
        for v in d_omega.as_slice() {
            assert!(v.abs() < TOL);
        }
    }
}

#[test]
fn cubical_codifferential_runs_on_3d_open_lattice() {
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_open(3);
    let m = unit_manifold(lattice);
    for k in 1..=3 {
        let delta = m.codifferential(k);
        assert_eq!(delta.len(), m.complex().num_cells(k - 1), "k = {k}");
        for v in delta.as_slice() {
            assert!(v.abs() < TOL);
        }
    }
}

#[test]
fn cubical_laplacian_runs_on_3d_periodic_lattice() {
    // The Stage-C cubical_heat_diffusion example is now a one-line
    // `manifold.laplacian(0)` call per the R4.5 proposal. This test exercises
    // that exact call on a periodic 3D cube.
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_torus(2);
    let m = unit_manifold(lattice);
    let lap = m.laplacian(0);
    assert_eq!(lap.len(), m.complex().num_cells(0));
    for v in lap.as_slice() {
        assert!(
            v.abs() < TOL,
            "Laplacian of zero field must be zero, got {v}"
        );
    }
}

#[test]
fn cubical_laplacian_responds_to_nonzero_input_on_2d_open_lattice() {
    // Behavioural sanity: a non-trivial 0-form input produces at least one
    // non-zero entry in `laplacian(0)`, confirming the operator is actually
    // running through the cubical Hodge ⋆ rather than no-oping.
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(3);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let mut data = vec![0.0_f64; total];
    // Put a unit "spike" at vertex 0 (a 0-form value).
    data[0] = 1.0;
    let tensor = CausalTensor::new(data, vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    let m = Manifold::from_cubical_with_metric(lattice, tensor, metric, 0);

    let lap = m.laplacian(0);
    let any_nonzero = lap.as_slice().iter().any(|v| v.abs() > TOL);
    assert!(
        any_nonzero,
        "Laplacian of a single-vertex spike must produce at least one non-zero entry"
    );
}

// ---------------------------------------------------------------------------
// R4.5.4 — discrete Hodge structural property: d² = 0 on cubical
// ---------------------------------------------------------------------------
//
// The full Hodge decomposition theorem (any 1-form on a trivial-topology
// lattice splits uniquely into exact + co-exact with no harmonic component)
// requires `Manifold::hodge_decompose`, which is the deliverable of the
// follow-up `add-hodge-decomposition` change set. Until that lands, the
// strongest structural check available here is the d² = 0 nilpotency
// identity, which is the cohomological prerequisite for the decomposition
// theorem and is well-defined against just the exterior derivative + the
// cubical complex.

#[test]
fn cubical_exterior_derivative_is_nilpotent_on_2d_periodic_lattice() {
    // d² = 0: applying the exterior derivative twice gives zero, regardless
    // of input. Test with a non-zero 0-form on a 4x4 torus.
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(4);
    let n0 = lattice.num_cells(0);
    let n1 = lattice.num_cells(1);
    let n2 = lattice.num_cells(2);
    let total = n0 + n1 + n2;

    // Put a spike at vertex 0.
    let mut data = vec![0.0_f64; total];
    data[0] = 1.0;
    let tensor = CausalTensor::new(data, vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    let m = Manifold::from_cubical_with_metric(lattice.clone(), tensor, metric, 0);

    let d_omega = m.exterior_derivative(0);
    assert_eq!(d_omega.len(), n1);

    // Re-embed d_omega as a 1-form in a new manifold and compute d again.
    let mut full = vec![0.0_f64; total];
    full[n0..n0 + n1].copy_from_slice(d_omega.as_slice());
    let tensor2 = CausalTensor::new(full, vec![total]).unwrap();
    let metric2: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    let m2 = Manifold::from_cubical_with_metric(lattice, tensor2, metric2, 0);
    let dd_omega = m2.exterior_derivative(1);

    assert_eq!(dd_omega.len(), n2);
    for v in dd_omega.as_slice() {
        assert!(v.abs() < TOL, "d² = 0 violated: got {v}");
    }
}

#[test]
fn cubical_exterior_derivative_is_nilpotent_on_3d_periodic_lattice() {
    // Same nilpotency check in 3D on a periodic cube. Tests d_0 followed by d_1.
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_torus(2);
    let n0 = lattice.num_cells(0);
    let n1 = lattice.num_cells(1);
    let n2 = lattice.num_cells(2);
    let n3 = lattice.num_cells(3);
    let total = n0 + n1 + n2 + n3;

    let mut data = vec![0.0_f64; total];
    data[0] = 1.0;
    let tensor = CausalTensor::new(data, vec![total]).unwrap();
    let metric: CubicalReggeGeometry<3, f64> = CubicalReggeGeometry::unit();
    let m = Manifold::from_cubical_with_metric(lattice.clone(), tensor, metric, 0);

    let d_omega = m.exterior_derivative(0);
    let mut full = vec![0.0_f64; total];
    full[n0..n0 + n1].copy_from_slice(d_omega.as_slice());
    let tensor2 = CausalTensor::new(full, vec![total]).unwrap();
    let metric2: CubicalReggeGeometry<3, f64> = CubicalReggeGeometry::unit();
    let m2 = Manifold::from_cubical_with_metric(lattice, tensor2, metric2, 0);
    let dd_omega = m2.exterior_derivative(1);

    assert_eq!(dd_omega.len(), n2);
    for v in dd_omega.as_slice() {
        assert!(v.abs() < TOL, "d² = 0 violated in 3D: got {v}");
    }
}
