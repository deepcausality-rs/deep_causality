/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `DecNsRate` tests: the assembled RHS against the Stage 0 pointwise
//! oracle at second order, the viscous-sign decay pin, exact body-force
//! additivity, all three precision backends, and every construction
//! rejection. The in-loop `expect`s of `eval` are documented coverage
//! exemptions (their invariants are construction-validated here).

use deep_causality_calculus::{DifferentiableField, DifferentiateFieldExt, Scalar};
use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_physics::{
    AccelerationVector, BodyForceOneForm, DecNsRate, Density, KinematicViscosity, Velocity3,
    VelocityGradient, VelocityOneForm, dec_kinetic_energy, incompressible_ns_rhs_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const NU: f64 = 0.01;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

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

fn metric_free_manifold(n: usize) -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(n);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    Manifold::from_cubical(lattice, data, 0)
}

fn tg_velocity(k: f64, x: f64, y: f64) -> [f64; 2] {
    [
        (k * x).sin() * (k * y).cos(),
        -(k * x).cos() * (k * y).sin(),
    ]
}

/// The sampled Taylor–Green field as an edge cochain via the de Rham map.
fn tg_edge_form(manifold: &Manifold<LatticeComplex<2, f64>, f64>, n: usize) -> CausalTensor<f64> {
    let k = 2.0 * std::f64::consts::PI / (n as f64);
    let complex = manifold.complex();
    let n0 = complex.num_cells(0);
    let mut vertex_field = vec![0.0; 2 * n0];
    for (vi, v) in complex.iter_cells(0).enumerate() {
        let u2 = tg_velocity(k, v.position()[0] as f64, v.position()[1] as f64);
        vertex_field[2 * vi] = u2[0];
        vertex_field[2 * vi + 1] = u2[1];
    }
    let vertex_tensor = CausalTensor::new(vertex_field, vec![2 * n0]).unwrap();
    manifold.de_rham(&vertex_tensor).unwrap()
}

// ---------------------------------------------------------------------------
// The Stage 0 pointwise oracle (tangent functor + Lamb identity)
// ---------------------------------------------------------------------------

struct TgComponent {
    comp: usize,
    k: f64,
}

impl DifferentiableField<2> for TgComponent {
    fn run<S: Scalar>(&self, p: &[S; 2]) -> S {
        let k = S::from_f64(self.k).expect("k lifts");
        match self.comp {
            0 => (k * p[0]).sin() * (k * p[1]).cos(),
            _ => S::from_f64(0.0).expect("zero lifts") - (k * p[0]).cos() * (k * p[1]).sin(),
        }
    }
}

struct TgKineticEnergy {
    k: f64,
}

impl DifferentiableField<2> for TgKineticEnergy {
    fn run<S: Scalar>(&self, p: &[S; 2]) -> S {
        let u = TgComponent { comp: 0, k: self.k }.run(p);
        let v = TgComponent { comp: 1, k: self.k }.run(p);
        let half = S::from_f64(0.5).expect("half lifts");
        (u * u + v * v) * half
    }
}

/// Oracle RHS along `axis` at `(x, y)`: kernel value (`∇p = 0`, `g = 0`,
/// `ρ = 1`) plus the kinetic-energy gradient (the Lamb-form identity).
fn oracle_rhs_component(k: f64, x: f64, y: f64, axis: usize) -> f64 {
    let u2 = tg_velocity(k, x, y);
    let u = Velocity3::new_unchecked([u2[0], u2[1], 0.0]);

    let g0 = TgComponent { comp: 0, k }.gradient(&[x, y]);
    let g1 = TgComponent { comp: 1, k }.gradient(&[x, y]);
    let grad_u = VelocityGradient::new_unchecked([
        [g0[0], g0[1], 0.0],
        [g1[0], g1[1], 0.0],
        [0.0, 0.0, 0.0],
    ]);

    let lap = [-2.0 * k * k * u2[0], -2.0 * k * k * u2[1], 0.0];

    let rhs = incompressible_ns_rhs_kernel(
        &u,
        &grad_u,
        &lap,
        &[0.0, 0.0, 0.0],
        &Density::new_unchecked(1.0),
        &KinematicViscosity::new_unchecked(NU),
        &AccelerationVector::new_unchecked([0.0, 0.0, 0.0]),
    )
    .expect("kernel evaluation with rho = 1 cannot fail");

    let grad_ke = TgKineticEnergy { k }.gradient(&[x, y]);
    rhs.value()[axis] + grad_ke[axis]
}

// ---------------------------------------------------------------------------
// RHS agreement with the oracle (task 1.4)
// ---------------------------------------------------------------------------

/// The full assembled rate (`−i_u du − ν Δ u`) matches the pointwise
/// oracle at second observed order over the refinement ladder — the
/// Stage 0 capstone cross-check, now exercised through `DecNsRate::eval`.
#[test]
fn rate_matches_pointwise_oracle_at_second_order() {
    let mut rel_errors = Vec::new();

    for n in [8usize, 16, 32] {
        let k = 2.0 * std::f64::consts::PI / (n as f64);
        let manifold = unit_manifold::<f64>(n);
        let u_flat = tg_edge_form(&manifold, n);
        let rate = DecNsRate::new(&manifold, NU, None).unwrap();
        let u = VelocityOneForm::new(u_flat, &manifold).unwrap();
        let rhs = rate.eval_unprojected(&u);

        let mut max_err = 0.0_f64;
        let mut max_ref = 0.0_f64;
        for (i, cell) in manifold.complex().iter_cells(1).enumerate() {
            let axis = cell.orientation().trailing_zeros() as usize;
            let p = cell.position();
            let (mx, my) = if axis == 0 {
                (p[0] as f64 + 0.5, p[1] as f64)
            } else {
                (p[0] as f64, p[1] as f64 + 0.5)
            };
            let dec = rhs.as_tensor().as_slice()[i];
            let oracle = oracle_rhs_component(k, mx, my, axis);
            max_err = max_err.max((dec - oracle).abs());
            max_ref = max_ref.max(oracle.abs());
        }
        rel_errors.push(max_err / max_ref);
    }

    assert!(
        rel_errors[1] < rel_errors[0] / 3.0,
        "rate vs oracle not second order (first refinement): {rel_errors:?}"
    );
    assert!(
        rel_errors[2] < rel_errors[1] / 3.0,
        "rate vs oracle not second order (second refinement): {rel_errors:?}"
    );
}

// ---------------------------------------------------------------------------
// Viscous-sign pin (task 1.4)
// ---------------------------------------------------------------------------

/// One explicit step along the rate decreases the energy of a discretely
/// divergence-free Laplacian eigenfield when `ν > 0`. With the
/// anti-diffusion sign error the energy would grow; this pins
/// `−ν Δ_dR = +ν∇²`.
fn assert_viscous_decay<R>()
where
    R: RealField
        + deep_causality_topology::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display,
{
    let n = 8usize;
    let manifold = unit_manifold::<R>(n);
    let complex = manifold.complex();
    let n1 = complex.num_cells(1);
    let k = 2.0 * std::f64::consts::PI / (n as f64);

    // Shear eigenfield: u_x(y) = sin(k y) on x-edges, zero on y-edges.
    // Constant along x, so it is exactly divergence-free discretely.
    let mut edge = vec![R::zero(); n1];
    for (i, cell) in complex.iter_cells(1).enumerate() {
        let axis = cell.orientation().trailing_zeros() as usize;
        if axis == 0 {
            let y = cell.position()[1] as f64;
            edge[i] = R::from_f64((k * y).sin()).unwrap();
        }
    }
    let u = VelocityOneForm::new(CausalTensor::new(edge, vec![n1]).unwrap(), &manifold).unwrap();

    let nu = R::from_f64(0.1).unwrap();
    let dt = R::from_f64(0.05).unwrap();
    let rate = DecNsRate::new(&manifold, nu, None).unwrap();
    assert_eq!(rate.nu(), nu);

    let stepped = u.clone() + rate.eval_unprojected(&u) * dt;

    let e0 = dec_kinetic_energy(&manifold, u.as_tensor()).unwrap();
    let e1 = dec_kinetic_energy(&manifold, stepped.as_tensor()).unwrap();
    assert!(
        e1 < e0,
        "viscous step must dissipate energy: E0 {e0}, E1 {e1}"
    );
}

#[test]
fn viscous_sign_decays_energy_f64() {
    assert_viscous_decay::<f64>();
}

#[test]
fn viscous_sign_decays_energy_f32() {
    assert_viscous_decay::<f32>();
}

#[test]
fn viscous_sign_decays_energy_float106() {
    assert_viscous_decay::<Float106>();
}

// ---------------------------------------------------------------------------
// Body-force additivity (task 1.4)
// ---------------------------------------------------------------------------

/// `rate_with_g(u) − rate_without_g(u) = g`, exactly.
#[test]
fn body_force_enters_additively_and_exactly() {
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let n1 = manifold.complex().num_cells(1);

    let u_flat = tg_edge_form(&manifold, n);
    let u = VelocityOneForm::new(u_flat, &manifold).unwrap();

    let g_vals: Vec<f64> = (0..n1).map(|i| 0.25 + 0.01 * (i as f64)).collect();
    let g = BodyForceOneForm::new(
        CausalTensor::new(g_vals.clone(), vec![n1]).unwrap(),
        &manifold,
    )
    .unwrap();

    let rate_plain = DecNsRate::new(&manifold, NU, None).unwrap();
    let rate_forced = DecNsRate::new(&manifold, NU, Some(&g)).unwrap();

    let rhs_plain = rate_plain.eval_unprojected(&u);
    let rhs_forced = rate_forced.eval_unprojected(&u);

    for (i, g_val) in g_vals.iter().enumerate().take(n1) {
        let diff = rhs_forced.as_tensor().as_slice()[i] - rhs_plain.as_tensor().as_slice()[i];
        assert!(
            (diff - g_val).abs() <= 1e-12,
            "body force must enter additively (machine rounding) at edge {i}: {diff} vs {g_val}"
        );
    }
}

// ---------------------------------------------------------------------------
// Construction rejections (task 1.5)
// ---------------------------------------------------------------------------

#[test]
fn rejects_one_dimensional_lattice() {
    let lattice: LatticeComplex<1, f64> = LatticeComplex::new([8], [true]);
    let total: usize = (0..=1).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<1, f64> = CubicalReggeGeometry::unit();
    let manifold = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

    let err = DecNsRate::new(&manifold, NU, None).unwrap_err();
    assert!(err.to_string().contains("dimension >= 2"), "{err}");
}

#[test]
fn rejects_metric_free_manifold() {
    let manifold = metric_free_manifold(6);
    let err = DecNsRate::new(&manifold, NU, None).unwrap_err();
    assert!(err.to_string().contains("metric"), "{err}");
}

#[test]
fn rejects_nan_viscosity() {
    let manifold = unit_manifold::<f64>(6);
    let err = DecNsRate::new(&manifold, f64::NAN, None).unwrap_err();
    assert!(err.to_string().contains("finite"), "{err}");
}

#[test]
fn rejects_infinite_viscosity() {
    let manifold = unit_manifold::<f64>(6);
    let err = DecNsRate::new(&manifold, f64::INFINITY, None).unwrap_err();
    assert!(err.to_string().contains("finite"), "{err}");
}

#[test]
fn rejects_negative_viscosity() {
    let manifold = unit_manifold::<f64>(6);
    let err = DecNsRate::new(&manifold, -0.01, None).unwrap_err();
    assert!(err.to_string().contains("negative"), "{err}");
}

#[test]
fn rejects_mismatched_body_force() {
    let manifold = unit_manifold::<f64>(6);
    let other = unit_manifold::<f64>(4);
    let n1_other = other.complex().num_cells(1);
    let g = BodyForceOneForm::new(
        CausalTensor::new(vec![0.1; n1_other], vec![n1_other]).unwrap(),
        &other,
    )
    .unwrap();

    let err = DecNsRate::new(&manifold, NU, Some(&g)).unwrap_err();
    assert!(err.to_string().contains("edge coefficients"), "{err}");
}

// ---------------------------------------------------------------------------
// The projected rate (the marching surface)
// ---------------------------------------------------------------------------

/// `eval_projected` returns a divergence-free rate: the projector sits
/// inside the marched ODE.
#[test]
fn projected_rate_is_divergence_free() {
    use deep_causality_physics::dec_divergence_residual;
    use deep_causality_topology::HodgeDecomposeOptions;

    let n = 8usize;
    let manifold = unit_manifold::<f64>(n);
    let u_flat = tg_edge_form(&manifold, n);
    let rate = DecNsRate::new(&manifold, NU, None).unwrap();
    let u = VelocityOneForm::new(u_flat, &manifold).unwrap();

    let projected = rate
        .eval_projected(&u, &HodgeDecomposeOptions::default())
        .unwrap();
    let residual = dec_divergence_residual(&manifold, projected.as_tensor()).unwrap();
    assert!(residual < 1e-8, "projected rate residual {residual}");
}

/// A starved CG budget surfaces the projection failure as a `Result`.
#[test]
fn projected_rate_cg_starvation_returns_error() {
    use deep_causality_topology::HodgeDecomposeOptions;

    let n = 8usize;
    let manifold = unit_manifold::<f64>(n);
    let u_flat = tg_edge_form(&manifold, n);
    let rate = DecNsRate::new(&manifold, NU, None).unwrap();
    let u = VelocityOneForm::new(u_flat, &manifold).unwrap();

    let err = rate
        .eval_projected(
            &u,
            &HodgeDecomposeOptions {
                tolerance: None,
                max_iterations: Some(1),
            },
        )
        .unwrap_err();
    assert!(err.to_string().contains("Leray projection failed"), "{err}");
}
