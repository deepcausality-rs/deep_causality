/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Validation rung 4 (`cfd-gap.md` §7): 2D Taylor–Green decay on
//! `square_torus` with a convergence table.
//!
//! The kinetic-energy envelope is `E(t) = E(0)·exp(−4νk²t)` for the
//! single-mode vortex with wavenumber `k = 2π/n` on the unit-spacing
//! lattice. The Chorin splitting bounds the *temporal* order of the march
//! at fixed `dt`, so the gate here is **spatial**: the envelope error
//! against the ladder `n ∈ [8, 16, 32]` at fixed `dt` and horizon, with
//! the observed order computed across the full ladder.
//!
//! Gates: f64 (and Float106 at the f64 gate) require observed spatial
//! order ≥ 1.9; f32 runs the `[8, 16]` ladder at a looser documented gate
//! (per-grid tolerance 8%, error must shrink by ≥ 2×) because its CG
//! tolerance floor (≈ 1.2e-5) sits within reach of the n = 32 signal.

use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_physics::{DecNsSolver, dec_kinetic_energy};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const NU: f64 = 0.05;
const DT: f64 = 0.2;
const STEPS: usize = 50;

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

fn tg_vertex_tensor<R>(manifold: &Manifold<LatticeComplex<2, R>, R>, n: usize) -> CausalTensor<R>
where
    R: RealField + FromPrimitive,
{
    let k = 2.0 * std::f64::consts::PI / (n as f64);
    let n0 = manifold.complex().num_cells(0);
    let mut vertex = vec![R::zero(); 2 * n0];
    for (vi, v) in manifold.complex().iter_cells(0).enumerate() {
        let (x, y) = (v.position()[0] as f64, v.position()[1] as f64);
        vertex[2 * vi] = R::from_f64((k * x).sin() * (k * y).cos()).unwrap();
        vertex[2 * vi + 1] = R::from_f64(-(k * x).cos() * (k * y).sin()).unwrap();
    }
    CausalTensor::new(vertex, vec![2 * n0]).unwrap()
}

/// Marches one grid and returns the relative envelope error
/// `|E_T/E_0 − exp(−4νk²T)| / exp(−4νk²T)`.
fn envelope_error<R>(n: usize) -> f64
where
    R: RealField
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display
        + Into<f64>,
{
    let manifold = unit_manifold::<R>(n);
    let solver = DecNsSolver::new(
        &manifold,
        R::from_f64(NU).unwrap(),
        R::from_f64(DT).unwrap(),
        None,
    )
    .unwrap();

    let state = solver
        .seed_from_vertex_vectors(&tg_vertex_tensor(&manifold, n))
        .unwrap();
    let e0: f64 = dec_kinetic_energy(&manifold, state.as_one_form())
        .unwrap()
        .into();

    let run = solver.run_n(state, STEPS).unwrap();
    let e_t: f64 = dec_kinetic_energy(&manifold, run.state().as_one_form())
        .unwrap()
        .into();

    let k = 2.0 * std::f64::consts::PI / (n as f64);
    let t = DT * STEPS as f64;
    let analytic = (-4.0 * NU * k * k * t).exp();
    ((e_t / e0) - analytic).abs() / analytic
}

#[test]
fn tg_2d_decay_second_order_f64() {
    let errors: Vec<f64> = [8usize, 16, 32]
        .into_iter()
        .map(envelope_error::<f64>)
        .collect();

    // Per-grid envelope tolerance (coarsest grid, documented): the n = 8
    // discrete-Laplacian truncation alone contributes ≈ 6.4%.
    assert!(errors[0] < 0.08, "n=8 envelope error too large: {errors:?}");

    // Observed spatial order across the full ladder: ln(e0/e2)/ln(4) ≥ 1.9.
    let order = (errors[0] / errors[2]).ln() / 4.0_f64.ln();
    assert!(
        order >= 1.9,
        "observed spatial order {order:.2} below 1.9: {errors:?}"
    );
}

#[test]
fn tg_2d_decay_float106_at_f64_gate() {
    let errors: Vec<f64> = [8usize, 16, 32]
        .into_iter()
        .map(envelope_error::<Float106>)
        .collect();

    assert!(errors[0] < 0.08, "n=8 envelope error too large: {errors:?}");
    let order = (errors[0] / errors[2]).ln() / 4.0_f64.ln();
    assert!(
        order >= 1.9,
        "observed spatial order {order:.2} below 1.9: {errors:?}"
    );
}

/// f32 gate (documented, looser): `[8, 16]` ladder, per-grid tolerance 8%,
/// at least a 2× error reduction per refinement. The f32 CG tolerance
/// floor makes the n = 32 signal unreliable at this precision.
#[test]
fn tg_2d_decay_f32_at_documented_gate() {
    let errors: Vec<f64> = [8usize, 16]
        .into_iter()
        .map(envelope_error::<f32>)
        .collect();

    assert!(errors[0] < 0.08, "n=8 envelope error too large: {errors:?}");
    assert!(
        errors[1] < errors[0] / 2.0,
        "f32 envelope error did not shrink by 2x: {errors:?}"
    );
}
