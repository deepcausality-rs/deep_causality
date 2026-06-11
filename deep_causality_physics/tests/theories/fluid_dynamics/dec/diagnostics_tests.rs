/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! DEC diagnostic tests: energy convergence to the analytic Taylor–Green
//! value over the refinement ladder, enstrophy of a constant field, the
//! helicity dimension guard and the helical ABC value, max speed, and
//! every error branch.

use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_physics::{
    dec_divergence_residual, dec_enstrophy, dec_helicity, dec_kinetic_energy, dec_max_speed,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

fn unit_manifold2<R>(n: usize) -> Manifold<LatticeComplex<2, R>, R>
where
    R: RealField + FromPrimitive,
{
    let lattice: LatticeComplex<2, R> = LatticeComplex::square_torus(n);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![R::zero(); total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, R> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn unit_manifold3(n: usize) -> Manifold<LatticeComplex<3, f64>, f64> {
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_torus(n);
    let total: usize = (0..=3).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<3, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

/// The sampled 2D Taylor–Green edge form at precision `R`.
fn tg_edge_form<R>(manifold: &Manifold<LatticeComplex<2, R>, R>, n: usize) -> CausalTensor<R>
where
    R: RealField
        + deep_causality_topology::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display,
{
    let k = 2.0 * std::f64::consts::PI / (n as f64);
    let n0 = manifold.complex().num_cells(0);
    let mut vertex = vec![R::zero(); 2 * n0];
    for (vi, v) in manifold.complex().iter_cells(0).enumerate() {
        let (x, y) = (v.position()[0] as f64, v.position()[1] as f64);
        vertex[2 * vi] = R::from_f64((k * x).sin() * (k * y).cos()).unwrap();
        vertex[2 * vi + 1] = R::from_f64(-(k * x).cos() * (k * y).sin()).unwrap();
    }
    let t = CausalTensor::new(vertex, vec![2 * n0]).unwrap();
    manifold.de_rham(&t).unwrap()
}

/// Kinetic energy of the sampled TG field converges to the analytic
/// `E = n²/4` at the discretization order over the ladder.
#[test]
fn tg_energy_converges_to_analytic_value() {
    let mut rel_errors = Vec::new();
    for n in [8usize, 16, 32] {
        let manifold = unit_manifold2::<f64>(n);
        let u = tg_edge_form(&manifold, n);
        let e = dec_kinetic_energy(&manifold, &u).unwrap();
        let analytic = (n * n) as f64 / 4.0;
        rel_errors.push((e - analytic).abs() / analytic);
    }
    assert!(
        rel_errors[1] < rel_errors[0] / 3.0 && rel_errors[2] < rel_errors[1] / 3.0,
        "TG energy not second order: {rel_errors:?}"
    );
}

/// Energy is finite and positive at all three precisions.
#[test]
fn energy_at_three_precisions() {
    fn check<R>()
    where
        R: RealField
        + deep_causality_topology::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display,
    {
        let manifold = unit_manifold2::<R>(8);
        let u = tg_edge_form(&manifold, 8);
        let e = dec_kinetic_energy(&manifold, &u).unwrap();
        assert!(e > R::zero() && e.is_finite());
    }
    check::<f32>();
    check::<f64>();
    check::<Float106>();
}

/// A constant (harmonic mean-flow) field has zero vorticity, hence zero
/// enstrophy, exactly.
#[test]
fn enstrophy_of_constant_field_is_zero() {
    let manifold = unit_manifold2::<f64>(8);
    let n1 = manifold.complex().num_cells(1);
    let constant = CausalTensor::new(vec![0.7; n1], vec![n1]).unwrap();
    let z = dec_enstrophy(&manifold, &constant).unwrap();
    assert_eq!(z, 0.0);
}

/// Enstrophy of the TG field is strictly positive.
#[test]
fn enstrophy_of_tg_is_positive() {
    let manifold = unit_manifold2::<f64>(8);
    let u = tg_edge_form(&manifold, 8);
    assert!(dec_enstrophy(&manifold, &u).unwrap() > 0.0);
}

/// Helicity is rejected on a 2D manifold with a dimension-naming error.
#[test]
fn helicity_rejected_in_2d() {
    let manifold = unit_manifold2::<f64>(6);
    let n1 = manifold.complex().num_cells(1);
    let u = CausalTensor::new(vec![0.1; n1], vec![n1]).unwrap();
    let err = dec_helicity(&manifold, &u).unwrap_err();
    assert!(err.to_string().contains("3D"), "{err}");
}

/// The ABC (Beltrami) flow has strictly positive helicity in 3D.
#[test]
fn abc_flow_has_positive_helicity() {
    let n = 8usize;
    let manifold = unit_manifold3(n);
    let k = 2.0 * std::f64::consts::PI / (n as f64);
    let n0 = manifold.complex().num_cells(0);
    let mut vertex = vec![0.0; 3 * n0];
    for (vi, v) in manifold.complex().iter_cells(0).enumerate() {
        let p = v.position();
        let (x, y, z) = (p[0] as f64, p[1] as f64, p[2] as f64);
        vertex[3 * vi] = (k * z).sin() + (k * y).cos();
        vertex[3 * vi + 1] = (k * x).sin() + (k * z).cos();
        vertex[3 * vi + 2] = (k * y).sin() + (k * x).cos();
    }
    let t = CausalTensor::new(vertex, vec![3 * n0]).unwrap();
    let u = manifold.de_rham(&t).unwrap();
    let h = dec_helicity(&manifold, &u).unwrap();
    assert!(h > 0.0, "ABC helicity must be positive, got {h}");
}

/// Max speed of the sampled TG field is close to the analytic 1.
#[test]
fn tg_max_speed_is_near_one() {
    let manifold = unit_manifold2::<f64>(16);
    let u = tg_edge_form(&manifold, 16);
    let s = dec_max_speed(&manifold, &u).unwrap();
    assert!((s - 1.0).abs() < 0.15, "max speed {s} not near 1");
}

/// Divergence residual of an exactly divergence-free shear field is zero.
#[test]
fn divergence_residual_of_shear_field_is_zero() {
    let manifold = unit_manifold2::<f64>(8);
    let complex = manifold.complex();
    let n1 = complex.num_cells(1);
    let k = 2.0 * std::f64::consts::PI / 8.0;
    let mut edge = vec![0.0; n1];
    for (i, cell) in complex.iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize == 0 {
            edge[i] = (k * cell.position()[1] as f64).sin();
        }
    }
    let u = CausalTensor::new(edge, vec![n1]).unwrap();
    let r = dec_divergence_residual(&manifold, &u).unwrap();
    assert!(
        r < 1e-14,
        "shear field must be exactly divergence-free, got {r}"
    );
}

// ---------------------------------------------------------------------------
// Error branches
// ---------------------------------------------------------------------------

#[test]
fn wrong_length_is_rejected_by_every_diagnostic() {
    let manifold = unit_manifold2::<f64>(6);
    let bad = CausalTensor::new(vec![0.0; 3], vec![3]).unwrap();

    assert!(dec_kinetic_energy(&manifold, &bad).is_err());
    assert!(dec_enstrophy(&manifold, &bad).is_err());
    assert!(dec_divergence_residual(&manifold, &bad).is_err());
    assert!(dec_max_speed(&manifold, &bad).is_err());

    let manifold3 = unit_manifold3(4);
    assert!(dec_helicity(&manifold3, &bad).is_err());
}

#[test]
fn metric_free_manifold_is_rejected() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(6);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let n1 = lattice.num_cells(1);
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let manifold = Manifold::from_cubical(lattice, data, 0);
    let u = CausalTensor::new(vec![0.1; n1], vec![n1]).unwrap();

    assert!(dec_kinetic_energy(&manifold, &u).is_err());
    assert!(dec_max_speed(&manifold, &u).is_err());
}
