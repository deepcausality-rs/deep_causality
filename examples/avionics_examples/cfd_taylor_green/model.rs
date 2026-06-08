/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the Taylor-Green MMS verification: the scalar-generic field equations, the
//! tangent-functor plumbing that turns them into `∇u`, `∇²u` and `∇p`, the Navier-Stokes
//! right-hand-side wrapper, the amplitude-march rate, and the exact-solution references. `main.rs`
//! orchestrates the verification as a monadic chain.
//!
//! Precision is a parameter. The physical constants (`ν`, `ρ`, the sample time) are exact `f64`
//! lifted into the working scalar with `from_f64`; the working precision `FloatType` carries the
//! whole computation: the autodiff scalar, the kernel arithmetic, and the Rk4 accumulation. Raise
//! `FloatType` to gain accuracy where the flow needs it.
//!
//! Taylor-Green vortex, 2-D embedded in 3-D with `w = 0`:
//!
//!     u =  sin x · cos y · F(t),   v = −cos x · sin y · F(t),   w = 0,
//!     p = (ρ/4)(cos 2x + cos 2y) · F(t)²,   F(t) = exp(−2 ν t).

use crate::FloatType;
use deep_causality_calculus::{DifferentiableField, DifferentiateFieldExt, Scalar};
use deep_causality_num::{Dual, FromPrimitive};

/// `exp(−x)` at the working scalar. The `Scalar` bound already exposes `exp`, so this evaluates the
/// transcendental at `S` for any precision (f32, f64, Float106) with no concrete-type assumption.
fn neg_exp<S: Scalar>(x: S) -> S {
    (-x).exp()
}
use deep_causality_physics::{
    AccelerationVector, Density, KinematicViscosity, PhysicsError, Velocity3, VelocityGradient,
    incompressible_ns_rhs_kernel,
};

/// Lift an exact `f64` constant into the working precision `FloatType`. The fully-qualified call
/// forces the `FromPrimitive` trait method; some scalars (e.g. `Float106`) carry an inherent
/// `from_f64` returning the value directly, which would otherwise shadow it.
pub fn ft(x: f64) -> FloatType {
    <FloatType as FromPrimitive>::from_f64(x).expect("constant lifts into FloatType")
}

// =============================================================================
// Scalar-generic field equations (parameters are exact f64 lift sources)
// =============================================================================

/// The decay factor `F(t) = exp(−2 ν t)`, computed entirely at the working scalar `S`. The exact
/// inputs `ν` and `t` are lifted with `from_f64`, then every arithmetic step (the product and the
/// transcendental) runs at `S`, so no precision is lost when `S` is wider than `f64`.
fn decay<S: Scalar>(nu: f64, t: f64) -> S {
    let two = S::from_f64(2.0).expect("two lifts into the working scalar");
    let nu = S::from_f64(nu).expect("ν lifts into the working scalar");
    let t = S::from_f64(t).expect("t lifts into the working scalar");
    neg_exp(two * nu * t)
}

/// Taylor-Green velocity component `comp` (0 = u, 1 = v, 2 = w) at point `p` and time `t`.
pub fn tg_velocity<S: Scalar>(comp: usize, p: &[S; 3], nu: f64, t: f64) -> S {
    let f = decay::<S>(nu, t);
    match comp {
        0 => p[0].sin() * p[1].cos() * f,
        1 => -(p[0].cos() * p[1].sin() * f),
        _ => S::from_f64(0.0).expect("zero lifts into the working scalar"),
    }
}

/// Taylor-Green pressure at point `p` and time `t`.
pub fn tg_pressure<S: Scalar>(p: &[S; 3], nu: f64, t: f64, rho: f64) -> S {
    let f = decay::<S>(nu, t);
    let f2 = f * f;
    let rho_s = S::from_f64(rho).expect("ρ lifts into the working scalar");
    let four = S::from_f64(4.0).expect("four lifts into the working scalar");
    let two = S::from_f64(2.0).expect("two lifts into the working scalar");
    (rho_s / four) * ((two * p[0]).cos() + (two * p[1]).cos()) * f2
}

/// One velocity component as a differentiable field of `(x, y, z)`. Its gradient is a row of `∇u`.
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

/// The pressure as a differentiable field of `(x, y, z)`. Its gradient is `∇p`.
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

// =============================================================================
// Exact spatial derivatives via the tangent functor
// =============================================================================

/// Velocity vector `u` at the sample point.
pub fn velocity(base: [FloatType; 3], nu: f64, t: f64) -> [FloatType; 3] {
    core::array::from_fn(|comp| tg_velocity::<FloatType>(comp, &base, nu, t))
}

/// Velocity Jacobian `∇u` with `value[i][j] = ∂u_i/∂x_j` (each row from `gradient`).
pub fn velocity_jacobian(base: [FloatType; 3], nu: f64, t: f64) -> [[FloatType; 3]; 3] {
    core::array::from_fn(|comp| TgVelocityField { comp, nu, t }.gradient(&base))
}

/// Second partial `∂²f/∂x_axis²` at `base`, by seeding axis `axis` as a doubly-nested variable and
/// the other coordinates as nested constants. All coordinates ride the working precision directly.
fn second_partial(field: &TgVelocityField, base: [FloatType; 3], axis: usize) -> FloatType {
    let seed: [Dual<Dual<FloatType>>; 3] = core::array::from_fn(|k| {
        if k == axis {
            Dual::variable(Dual::variable(base[k]))
        } else {
            Dual::constant(Dual::constant(base[k]))
        }
    });
    field.run(&seed).derivative().derivative()
}

/// Velocity Laplacian `∇²u`: `Σ_axis ∂²u_comp/∂x_axis²` from nested duals.
pub fn velocity_laplacian(base: [FloatType; 3], nu: f64, t: f64) -> [FloatType; 3] {
    core::array::from_fn(|comp| {
        let field = TgVelocityField { comp, nu, t };
        (0..3).map(|axis| second_partial(&field, base, axis)).sum()
    })
}

/// Pressure gradient `∇p` from `gradient`.
pub fn pressure_gradient(base: [FloatType; 3], nu: f64, t: f64, rho: f64) -> [FloatType; 3] {
    TgPressureField { nu, t, rho }.gradient(&base)
}

// =============================================================================
// Navier-Stokes RHS and the amplitude-march rate
// =============================================================================

/// The incompressible Navier-Stokes right-hand side `∂u/∂t`, wrapping the kernel and its typed
/// quantities around the raw arrays the AD layer produces.
pub fn navier_stokes_rhs(
    u: &[FloatType; 3],
    grad_u: &[[FloatType; 3]; 3],
    lap_u: &[FloatType; 3],
    grad_p: &[FloatType; 3],
    nu: f64,
    rho: f64,
) -> Result<[FloatType; 3], PhysicsError> {
    let u_q = Velocity3::new(*u)?;
    let gu = VelocityGradient::new(*grad_u)?;
    let rho_q = Density::new(ft(rho))?;
    let nu_q = KinematicViscosity::new(ft(nu))?;
    let body = AccelerationVector::new([ft(0.0); 3])?;
    Ok(incompressible_ns_rhs_kernel(&u_q, &gu, lap_u, grad_p, &rho_q, &nu_q, &body)?.into_inner())
}

/// Build the Rk4 rate field for the decaying amplitude `a(t)`. The Taylor-Green field keeps its
/// spatial shape, so velocity, `∇u` and `∇²u` scale with `a` and pressure with `a²`. The rate runs
/// the SAME kernel at every step, then projects the acceleration onto the initial velocity to
/// recover `da/dt`, so the march exercises the whole pipeline rather than a closed form.
pub fn amplitude_rate(
    u0: [FloatType; 3],
    grad_u0: [[FloatType; 3]; 3],
    lap_u0: [FloatType; 3],
    grad_p0: [FloatType; 3],
    nu: f64,
    rho: f64,
) -> impl Fn(&FloatType) -> FloatType {
    let rho_q = Density::new_unchecked(ft(rho));
    let nu_q = KinematicViscosity::new_unchecked(ft(nu));
    let body = AccelerationVector::new_unchecked([ft(0.0); 3]);
    let u0_sq = u0[0] * u0[0] + u0[1] * u0[1] + u0[2] * u0[2];

    move |a: &FloatType| {
        let a = *a;
        let u = Velocity3::new_unchecked([a * u0[0], a * u0[1], a * u0[2]]);
        let gu = VelocityGradient::new_unchecked(scale_3x3(grad_u0, a));
        let lap = scale_3(lap_u0, a);
        let gp = scale_3(grad_p0, a * a);
        let accel = incompressible_ns_rhs_kernel(&u, &gu, &lap, &gp, &rho_q, &nu_q, &body)
            .expect("kernel evaluates")
            .into_inner();
        (accel[0] * u0[0] + accel[1] * u0[1] + accel[2] * u0[2]) / u0_sq
    }
}

// =============================================================================
// Exact-solution references (the manufactured truth)
// =============================================================================

/// Exact Taylor-Green time derivative `∂u/∂t = −2 ν u`, evaluated at the working precision.
pub fn exact_time_derivative(u: &[FloatType; 3], nu: f64) -> [FloatType; 3] {
    let factor = ft(-2.0) * ft(nu);
    [factor * u[0], factor * u[1], factor * u[2]]
}

/// Exact amplitude decay `a(t) = exp(−2 ν t)`, evaluated at the working precision (the transcendental
/// runs at `FloatType`, not at `f64`).
pub fn exact_decay(nu: f64, t: FloatType) -> FloatType {
    neg_exp(ft(2.0) * ft(nu) * t)
}

/// Absolute value at the working precision. The `Scalar` bound exposes `abs`, so this avoids the
/// inherent-vs-trait ambiguity a concrete `FloatType::abs` would hit on scalars like `Float106`.
fn fabs<S: Scalar>(x: S) -> S {
    x.abs()
}

/// Maximum at the working precision, by comparison. Uses only `PartialOrd` (from `Scalar`), so it
/// needs neither an `Ord` nor a `Float`-trait bound.
fn fmax<S: Scalar>(a: S, b: S) -> S {
    if a > b { a } else { b }
}

/// The largest component-wise absolute difference between two vectors.
pub fn max_abs_diff(a: &[FloatType; 3], b: &[FloatType; 3]) -> FloatType {
    (0..3).map(|i| fabs(a[i] - b[i])).fold(ft(0.0), fmax)
}

/// Absolute difference of two scalars at the working precision.
pub fn abs_diff(a: FloatType, b: FloatType) -> FloatType {
    fabs(a - b)
}

fn scale_3(v: [FloatType; 3], s: FloatType) -> [FloatType; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn scale_3x3(m: [[FloatType; 3]; 3], s: FloatType) -> [[FloatType; 3]; 3] {
    core::array::from_fn(|i| core::array::from_fn(|j| m[i][j] * s))
}

/// Output of stage 1: the field at the sample point, its exact spatial derivatives, the kernel
/// time derivative, and the residual against the manufactured solution.
#[derive(Default, Clone, Debug)]
pub struct Stage1 {
    pub(crate) u: [FloatType; 3],
    pub(crate) grad_u: [[FloatType; 3]; 3],
    pub(crate) lap_u: [FloatType; 3],
    pub(crate) grad_p: [FloatType; 3],
    pub(crate) dudt: [FloatType; 3],
    pub(crate) exact_dudt: [FloatType; 3],
    pub(crate) kernel_err: FloatType,
}

/// Output of stage 2: the marched amplitude against the exact decay.
#[derive(Default, Clone, Debug)]
pub struct Report {
    pub(crate) s1: Stage1,
    pub(crate) a_final: FloatType,
    pub(crate) a_exact: FloatType,
    pub(crate) t_final: FloatType,
    pub(crate) steps: usize,
}
