/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Manufactured solutions for MMS verification — the corpus library plus the [`Manufactured`] seam.
//!
//! The Method of Manufactured Solutions feeds a kernel the *exact* spatial derivatives of an
//! analytic field and checks that it returns the exact time derivative. Computing those derivatives
//! by the **tangent functor** (`deep_causality_calculus` autodiff) makes them free of finite
//! differences and exact to machine precision — a recurring need in CFD (verification, sensitivity,
//! adjoints), so the autodiff bridge is corpus, not example-local.
//!
//! - [`Manufactured`] is the seam: any analytic field plugs in by yielding the pointwise inputs and
//!   the exact `∂u/∂t` reference at a sample point.
//! - [`TaylorGreen`] is the corpus solution for the canonical Taylor–Green vortex (a standard CFD
//!   verification benchmark), built on the autodiff bridge.

use crate::types::CfdScalar;
use deep_causality_calculus::{DifferentiableField, DifferentiateFieldExt, Scalar};
use deep_causality_num_dual::Dual;
use deep_causality_physics::PhysicsError;

/// The pointwise inputs an MMS kernel residual needs at a sample point, plus the exact reference.
#[derive(Debug, Clone, Copy)]
pub struct ManufacturedSample<R: CfdScalar> {
    /// Velocity `u`.
    pub velocity: [R; 3],
    /// Velocity Jacobian `∇u` with `[i][j] = ∂u_i/∂x_j`.
    pub velocity_jacobian: [[R; 3]; 3],
    /// Velocity Laplacian `∇²u`.
    pub velocity_laplacian: [R; 3],
    /// Pressure gradient `∇p`.
    pub pressure_gradient: [R; 3],
    /// The exact Eulerian acceleration `∂u/∂t` (the manufactured reference).
    pub exact_time_derivative: [R; 3],
}

/// A manufactured analytic solution: the MMS seam. A corpus solution (e.g. [`TaylorGreen`]) or a
/// caller-supplied field both implement it; the verification workflow consumes it.
pub trait Manufactured<R: CfdScalar> {
    /// Sample the field at point `p` and time `t`: the pointwise kernel inputs and the exact
    /// `∂u/∂t` reference (derivatives via the tangent functor — exact, no finite differences).
    fn sample(&self, p: &[R; 3], t: f64) -> ManufacturedSample<R>;

    /// The fluid density `ρ` (exact `f64` specification).
    fn density(&self) -> f64;

    /// The kinematic viscosity `ν` (exact `f64` specification).
    fn viscosity(&self) -> f64;
}

/// The Taylor–Green vortex (2D embedded in 3D, `w = 0`), an exact solution of the incompressible
/// Navier–Stokes equations and the canonical MMS benchmark:
///
/// ```text
/// u =  sin x · cos y · F(t),   v = −cos x · sin y · F(t),   w = 0,
/// p = (ρ/4)(cos 2x + cos 2y) · F(t)²,   F(t) = exp(−2 ν t).
/// ```
///
/// Its defining property: each velocity component decays as a single exponential, so
/// `∂u/∂t = −2ν u` — an independent reference for the kernel residual.
#[derive(Debug, Clone, Copy)]
pub struct TaylorGreen {
    nu: f64,
    rho: f64,
}

impl TaylorGreen {
    /// A Taylor–Green vortex with kinematic viscosity `nu` and density `rho`.
    pub fn new(nu: f64, rho: f64) -> Self {
        Self { nu, rho }
    }
}

impl<R: CfdScalar + Scalar> Manufactured<R> for TaylorGreen {
    fn sample(&self, p: &[R; 3], t: f64) -> ManufacturedSample<R> {
        let velocity = velocity::<R>(p, self.nu, t);
        ManufacturedSample {
            velocity,
            velocity_jacobian: velocity_jacobian::<R>(p, self.nu, t),
            velocity_laplacian: velocity_laplacian::<R>(p, self.nu, t),
            pressure_gradient: pressure_gradient::<R>(p, self.nu, t, self.rho),
            // Taylor–Green decay: ∂u/∂t = −2ν u.
            exact_time_derivative: exact_time_derivative::<R>(&velocity, self.nu),
        }
    }

    fn density(&self) -> f64 {
        self.rho
    }

    fn viscosity(&self) -> f64 {
        self.nu
    }
}

// ---------------------------------------------------------------------------
// Scalar-generic field equations + the tangent-functor derivatives
// ---------------------------------------------------------------------------

/// `exp(−x)` at the working scalar.
fn neg_exp<S: Scalar>(x: S) -> S {
    (-x).exp()
}

/// `F(t) = exp(−2 ν t)`, at the working scalar `S` (exact `ν`, `t` lifted once).
fn decay<S: Scalar>(nu: f64, t: f64) -> S {
    let two = S::from_f64(2.0).expect("two lifts into the working scalar");
    let nu = S::from_f64(nu).expect("ν lifts into the working scalar");
    let t = S::from_f64(t).expect("t lifts into the working scalar");
    neg_exp(two * nu * t)
}

/// Taylor–Green velocity component `comp` (0 = u, 1 = v, 2 = w) at point `p`, time `t`.
fn tg_velocity<S: Scalar>(comp: usize, p: &[S; 3], nu: f64, t: f64) -> S {
    let f = decay::<S>(nu, t);
    match comp {
        0 => p[0].sin() * p[1].cos() * f,
        1 => -(p[0].cos() * p[1].sin() * f),
        _ => S::from_f64(0.0).expect("zero lifts into the working scalar"),
    }
}

/// Taylor–Green pressure at point `p`, time `t`.
fn tg_pressure<S: Scalar>(p: &[S; 3], nu: f64, t: f64, rho: f64) -> S {
    let f = decay::<S>(nu, t);
    let f2 = f * f;
    let rho_s = S::from_f64(rho).expect("ρ lifts into the working scalar");
    let four = S::from_f64(4.0).expect("four lifts into the working scalar");
    let two = S::from_f64(2.0).expect("two lifts into the working scalar");
    (rho_s / four) * ((two * p[0]).cos() + (two * p[1]).cos()) * f2
}

/// One velocity component as a differentiable field of `(x, y, z)`; its gradient is a row of `∇u`.
struct TgVelocityField {
    comp: usize,
    nu: f64,
    t: f64,
}

impl DifferentiableField<3> for TgVelocityField {
    fn run<S: Scalar>(&self, p: &[S; 3]) -> S {
        tg_velocity(self.comp, p, self.nu, self.t)
    }
}

/// The pressure as a differentiable field of `(x, y, z)`; its gradient is `∇p`.
struct TgPressureField {
    nu: f64,
    t: f64,
    rho: f64,
}

impl DifferentiableField<3> for TgPressureField {
    fn run<S: Scalar>(&self, p: &[S; 3]) -> S {
        tg_pressure(p, self.nu, self.t, self.rho)
    }
}

/// Velocity vector `u` at the sample point.
fn velocity<R: CfdScalar + Scalar>(base: &[R; 3], nu: f64, t: f64) -> [R; 3] {
    core::array::from_fn(|comp| tg_velocity::<R>(comp, base, nu, t))
}

/// Velocity Jacobian `∇u` (`[i][j] = ∂u_i/∂x_j`, each row from `gradient`).
fn velocity_jacobian<R: CfdScalar + Scalar>(base: &[R; 3], nu: f64, t: f64) -> [[R; 3]; 3] {
    core::array::from_fn(|comp| TgVelocityField { comp, nu, t }.gradient(base))
}

/// Second partial `∂²f/∂x_axis²` at `base`, seeding `axis` as a doubly-nested variable.
fn second_partial<R: CfdScalar + Scalar>(field: &TgVelocityField, base: &[R; 3], axis: usize) -> R {
    let seed: [Dual<Dual<R>>; 3] = core::array::from_fn(|k| {
        if k == axis {
            Dual::variable(Dual::variable(base[k]))
        } else {
            Dual::constant(Dual::constant(base[k]))
        }
    });
    field.run(&seed).derivative().derivative()
}

/// Velocity Laplacian `∇²u`: `Σ_axis ∂²u_comp/∂x_axis²` from nested duals.
fn velocity_laplacian<R: CfdScalar + Scalar>(base: &[R; 3], nu: f64, t: f64) -> [R; 3] {
    core::array::from_fn(|comp| {
        let field = TgVelocityField { comp, nu, t };
        (0..3)
            .map(|axis| second_partial::<R>(&field, base, axis))
            .fold(R::zero(), |acc, x| acc + x)
    })
}

/// Pressure gradient `∇p` from `gradient`.
fn pressure_gradient<R: CfdScalar + Scalar>(base: &[R; 3], nu: f64, t: f64, rho: f64) -> [R; 3] {
    TgPressureField { nu, t, rho }.gradient(base)
}

/// Exact Taylor–Green time derivative `∂u/∂t = −2 ν u`.
fn exact_time_derivative<R: CfdScalar>(u: &[R; 3], nu: f64) -> [R; 3] {
    let factor = R::from_f64(-2.0 * nu).expect("−2ν lifts into R");
    [factor * u[0], factor * u[1], factor * u[2]]
}

// ---------------------------------------------------------------------------
// MMS-verification config (the "what"); the workflow runs in `CfdFlow::verify`
// ---------------------------------------------------------------------------

/// An owned MMS-verification configuration: a [`Manufactured`] solution `M`, the sample point/time,
/// and an optional kernel-in-the-loop amplitude march. Built by
/// [`CfdConfigBuilder::verify`](crate::CfdConfigBuilder); run by [`CfdFlow::verify`](crate::CfdFlow).
pub struct VerifyConfig<R: CfdScalar, M: Manufactured<R>> {
    pub(crate) name: String,
    pub(crate) manufactured: M,
    pub(crate) point: [R; 3],
    pub(crate) t: f64,
    /// Optional `(dt, steps)` for the amplitude march against `exp(−2νt)`.
    pub(crate) amplitude_march: Option<(R, usize)>,
}

/// Fluent builder for a [`VerifyConfig`].
pub struct VerifyConfigBuilder<R: CfdScalar, M: Manufactured<R>> {
    name: String,
    manufactured: M,
    point: Option<[R; 3]>,
    t: f64,
    amplitude_march: Option<(R, usize)>,
}

impl<R: CfdScalar, M: Manufactured<R>> VerifyConfigBuilder<R, M> {
    pub(crate) fn new(name: impl Into<String>, manufactured: M) -> Self {
        Self {
            name: name.into(),
            manufactured,
            point: None,
            t: 0.0,
            amplitude_march: None,
        }
    }

    /// The sample point and time at which the kernel residual is evaluated (required).
    pub fn sample_at(mut self, point: [R; 3], t: f64) -> Self {
        self.point = Some(point);
        self.t = t;
        self
    }

    /// Add a kernel-in-the-loop amplitude march (`dt`, `steps`) against the analytic decay.
    pub fn amplitude_march(mut self, dt: R, steps: usize) -> Self {
        self.amplitude_march = Some((dt, steps));
        self
    }

    /// Finalize the configuration.
    ///
    /// # Errors
    /// `PhysicsError::DimensionMismatch` when the sample point was not set.
    pub fn build(self) -> Result<VerifyConfig<R, M>, PhysicsError> {
        let point = self.point.ok_or_else(|| {
            PhysicsError::DimensionMismatch(
                "CfdConfigBuilder::verify: a sample point is required".into(),
            )
        })?;
        Ok(VerifyConfig {
            name: self.name,
            manufactured: self.manufactured,
            point,
            t: self.t,
            amplitude_march: self.amplitude_march,
        })
    }
}
