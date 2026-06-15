/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Solver construction and configuration tests: delegated rate rejections,
//! `dt` validation, CFL-factor validation, getters.

use deep_causality_cfd::DecNsSolver;
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

fn unit_manifold<R>(n: usize) -> Manifold<LatticeComplex<2, R>, R>
where
    R: RealField + FromPrimitive,
{
    let lattice: LatticeComplex<2, R> = LatticeComplex::square_torus(n);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![R::zero(); total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, R> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

#[test]
fn constructor_getters_report_configuration() {
    let manifold = unit_manifold::<f64>(6);
    let solver = DecNsSolver::new(&manifold, 0.02, 0.1, None).unwrap();
    assert_eq!(solver.dt(), 0.1);
    assert_eq!(solver.nu(), 0.02);
    // Unit Regge geometry: every edge has length 1.
    assert_eq!(solver.dx_min(), 1.0);
    assert_eq!(solver.rate().nu(), 0.02);
}

#[test]
fn constructor_delegates_rate_rejections() {
    let manifold = unit_manifold::<f64>(6);
    // Negative viscosity travels through the rate's validation.
    let err = DecNsSolver::new(&manifold, -1.0, 0.1, None).unwrap_err();
    assert!(err.to_string().contains("negative"), "{err}");
}

#[test]
fn constructor_rejects_bad_dt() {
    let manifold = unit_manifold::<f64>(6);
    for bad in [0.0, -0.1, f64::NAN, f64::INFINITY] {
        let err = DecNsSolver::new(&manifold, 0.01, bad, None).unwrap_err();
        assert!(err.to_string().contains("dt"), "{err}");
    }
}

#[test]
fn cfl_factor_builder_validates() {
    let manifold = unit_manifold::<f64>(6);

    let ok = DecNsSolver::new(&manifold, 0.01, 0.1, None)
        .unwrap()
        .with_cfl_factors(0.5, 0.4);
    assert!(ok.is_ok());

    for (a, d) in [
        (0.0, 0.9),
        (-1.0, 0.9),
        (f64::NAN, 0.9),
        (0.9, 0.0),
        (0.9, -1.0),
        (0.9, f64::INFINITY),
    ] {
        let err = DecNsSolver::new(&manifold, 0.01, 0.1, None)
            .unwrap()
            .with_cfl_factors(a, d)
            .unwrap_err();
        assert!(err.to_string().contains("safety factors"), "{err}");
    }
}

#[test]
fn debug_is_implemented() {
    let manifold = unit_manifold::<f64>(4);
    let solver = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    assert!(!format!("{solver:?}").is_empty());
}

#[test]
fn constructor_rejects_edgeless_lattice() {
    // A zero-shape lattice has no edges; the dx_min scan must reject it
    // instead of configuring an unusable CFL guard.
    let lattice: LatticeComplex<2, f64> = LatticeComplex::new([0, 0], [true, true]);
    let data = CausalTensor::new(vec![], vec![0]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    let manifold = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

    let err = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap_err();
    assert!(err.to_string().contains("no edges"), "{err}");
}
