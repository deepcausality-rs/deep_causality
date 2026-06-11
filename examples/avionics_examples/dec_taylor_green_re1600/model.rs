/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the Re-1600 Taylor–Green run: the lattice manifold, the
//! classic initial field, the solver configuration, and the two flow
//! stages (seed, march). `main.rs` orchestrates them as a causal flow.
//!
//! Generic over the precision type `R` (`f32`, `f64`, `Float106`, …) so
//! the same pipeline composes at any precision the framework supports —
//! every struct carries `R`, every computed quantity stays at `R`, and
//! the exact `f64` specifications (Re, the CFL step, π) enter once
//! through `from_f64` and never come back down. The display layer is the
//! only place a value is cast for presentation.

use core::fmt::Debug;

use deep_causality_core::{CausalityError, CausalityErrorEnum};
use deep_causality_num::RealField;
use deep_causality_physics::{
    DecNsScalar, DecNsSolver, SolenoidalField, VelocityOneForm, dec_kinetic_energy,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

/// The benchmark Reynolds number of the workshop case (exact specification).
pub const RE: f64 = 1600.0;

/// CFL-safe time step on the unit-spacing lattice (`max|u| ≈ 1`, default
/// safety factor 0.9). Exact specification, lifted once.
pub const CFL_DT: f64 = 0.2;

/// Lifts an exact `f64` specification into the working precision, with
/// the target type inferred at the call site — `let nu: R = flt!(…)`, or
/// any position where `R` is already pinned by the surrounding
/// expression. Routed through `FromPrimitive` (not `From<f64>`) so the
/// same macro serves `f32`, `f64`, and `Float106` alike.
macro_rules! flt {
    ($x:expr) => {
        deep_causality_num::FromPrimitive::from_f64($x).expect("specification lifts into R")
    };
}

/// One sample of the dissipation curve, in convective units, at the
/// working precision.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Sample<R: RealField> {
    pub t_star: R,
    pub energy_per_vol: R,
    pub dissipation: R,
}

/// The march result the flow carries to the printer, at the working
/// precision.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Report<R: RealField> {
    pub series: Vec<Sample<R>>,
}

/// The periodic cubic lattice with the unit Regge metric at `R`.
pub fn unit_manifold3<R: DecNsScalar>(n: usize) -> Manifold<LatticeComplex<3, R>, R> {
    let lattice: LatticeComplex<3, R> = LatticeComplex::cubic_torus(n);
    let total: usize = (0..=3).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![R::zero(); total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<3, R> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

/// The unit wavenumber of the `[0, n]³` lattice, at `R`: `k = 2π/n`.
fn wavenumber<R: DecNsScalar>(n: usize) -> R {
    let two_pi: R = flt!(2.0 * std::f64::consts::PI);
    let n_r: R = flt!(n as f64);
    two_pi / n_r
}

/// The solver at `Re = 1600`: on this lattice `U = 1` and `L = 1/k`, so
/// `ν = U·L/Re = 1/(k·Re)`, computed at `R`.
pub fn solver<R: DecNsScalar>(
    manifold: &Manifold<LatticeComplex<3, R>, R>,
    n: usize,
) -> Result<DecNsSolver<'_, 3, R>, CausalityError> {
    let re: R = flt!(RE);
    let nu = R::one() / (wavenumber::<R>(n) * re);
    DecNsSolver::new(manifold, nu, flt!(CFL_DT), None)
        .map_err(|e| err(&format!("solver configuration: {e}")))
}

/// Stage 1: sample the classic 3D Taylor–Green field at the vertices —
/// the trigonometry runs at `R` — and seed it through the de Rham map and
/// the `t = 0` projection. The flow carries the projected edge cochain; a
/// projection failure short-circuits.
pub fn stage_seed<R: DecNsScalar>(
    solver: &DecNsSolver<'_, 3, R>,
    manifold: &Manifold<LatticeComplex<3, R>, R>,
    n: usize,
) -> Result<CausalTensor<R>, CausalityError> {
    let k = wavenumber::<R>(n);
    let n0 = manifold.complex().num_cells(0);
    let mut vertex = vec![R::zero(); 3 * n0];
    for (vi, v) in manifold.complex().iter_cells(0).enumerate() {
        let p = v.position();
        // Integer lattice coordinates lift exactly; sin/cos run at R.
        let xi: R = flt!(p[0] as f64);
        let yi: R = flt!(p[1] as f64);
        let zi: R = flt!(p[2] as f64);
        let (x, y, z) = (k * xi, k * yi, k * zi);
        vertex[3 * vi] = x.sin() * y.cos() * z.cos();
        vertex[3 * vi + 1] = R::zero() - x.cos() * y.sin() * z.cos();
        vertex[3 * vi + 2] = R::zero();
    }
    let vertex_tensor = CausalTensor::new(vertex, vec![3 * n0]).unwrap();

    solver
        .seed_from_vertex_vectors(&vertex_tensor)
        .map(|state| state.as_one_form().clone())
        .map_err(|e| err(&format!("seeding: {e}")))
}

/// Stage 2: re-enter the type-state (the projection is near-free on the
/// already-projected cochain), march to the horizon, and collect the
/// energy/dissipation series in convective units, all at `R`. A CG
/// failure or CFL violation short-circuits with the failing step in the
/// message.
pub fn stage_march<R: DecNsScalar>(
    solver: &DecNsSolver<'_, 3, R>,
    manifold: &Manifold<LatticeComplex<3, R>, R>,
    n: usize,
    t_star_max: f64,
    seeded: CausalTensor<R>,
) -> Result<Report<R>, CausalityError> {
    let k = wavenumber::<R>(n);
    let volume: R = flt!((n * n * n) as f64);
    let cfl_dt: R = flt!(CFL_DT);
    let dt_star = k * cfl_dt;
    let steps = (t_star_max / (2.0 * std::f64::consts::PI / (n as f64) * CFL_DT)).ceil() as usize;

    // Back into the SolenoidalField type-state through its only door.
    let velocity =
        VelocityOneForm::new(seeded, manifold).map_err(|e| err(&format!("re-entry: {e}")))?;
    let (mut state, _potential) = SolenoidalField::from_leray_projection(&velocity, manifold)
        .map_err(|e| err(&format!("re-entry projection: {e}")))?;

    let energy_per_vol = |s: &SolenoidalField<R>| -> Result<R, CausalityError> {
        dec_kinetic_energy(manifold, s.as_one_form())
            .map(|e| e / volume)
            .map_err(|e| err(&format!("energy: {e}")))
    };

    let mut series = Vec::with_capacity(steps + 1);
    let mut e_prev = energy_per_vol(&state)?;
    let mut t_star = R::zero();
    series.push(Sample {
        t_star,
        energy_per_vol: e_prev,
        dissipation: R::zero(),
    });

    for step in 1..=steps {
        let output = solver
            .step(&state)
            .map_err(|e| err(&format!("step {step}: {e}")))?;
        state = output.into_state();
        let e = energy_per_vol(&state)?;
        t_star += dt_star;
        series.push(Sample {
            t_star,
            energy_per_vol: e,
            // Dissipation in convective units: −dE*/dt*, at R.
            dissipation: (e_prev - e) / dt_star,
        });
        e_prev = e;
    }

    Ok(Report { series })
}

fn err(msg: &str) -> CausalityError {
    CausalityError::new(CausalityErrorEnum::Custom(msg.into()))
}
