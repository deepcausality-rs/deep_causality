/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The **MMS-verification** CfdFlow solver kind: a manufactured analytic solution checked
//! against a pointwise `FluidTheory` regime evaluator — no DEC march.
//!
//! For each regime a closed-form solution supplies the pointwise inputs (`u`, `∇u`, `∇²u`,
//! `∇p`, …) and an *independent* reference acceleration `∂u/∂t` at a sample point; the regime
//! kernel is evaluated on those inputs and the `mms_error` is the norm of the residual against
//! the reference. The references are exact (not kernel-derived), so a passing error genuinely
//! pins the kernel:
//!
//! - **Incompressible** — the 2D Taylor–Green vortex (Taylor 1923): `∂u/∂t = −2ν u`.
//! - **Euler** — the Taylor–Green field at `t = 0`: the inviscid convective/pressure balance is
//!   exact, so `∂u/∂t = 0`.
//! - **Stokes** — plane Poiseuille flow (Batchelor §4.2): the viscous and pressure terms cancel,
//!   so `∂u/∂t = 0`.
//! - **Compressible** — the Taylor–Green field in its incompressible limit (`∇·τ = ρν∇²u`,
//!   divergence-free): momentum `∂u/∂t = −2ν u`, continuity `∂ρ/∂t = 0`.

use crate::CfdScalar;
use crate::theories::{
    compressible_ns_continuity_rhs, compressible_ns_momentum_rhs, euler_momentum_rhs,
    incompressible_ns_rhs, stokes_momentum_rhs,
};
use crate::traits::Solver;
use crate::types::flow::{CfdFlow, Report};
use deep_causality_physics::{
    AccelerationVector, Density, KinematicViscosity, PhysicsError, Velocity3, VelocityGradient,
};

/// A Navier–Stokes regime — the `FluidTheory` selector for MMS verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Regime {
    /// Incompressible Newtonian momentum (`incompressible_ns_rhs`).
    Incompressible,
    /// Inviscid Euler momentum (`euler_momentum_rhs`).
    Euler,
    /// Creeping Stokes momentum (`stokes_momentum_rhs`).
    Stokes,
    /// Compressible Newtonian momentum + continuity (`compressible_ns_*`).
    Compressible,
}

impl CfdFlow {
    /// Begin an MMS-verification case for a single regime. The regime, viscosity, and density
    /// are set fluently; `run` evaluates the regime kernel against the manufactured reference and
    /// reports the `mms_error` (and `continuity_error` for the compressible regime).
    pub fn verify_mms<R: CfdScalar>(name: impl Into<String>) -> MmsBuilder<R> {
        MmsBuilder::new(name)
    }
}

/// Fluent builder for an MMS-verification case.
pub struct MmsBuilder<R: CfdScalar> {
    name: String,
    regime: Regime,
    nu: R,
    rho: R,
}

impl<R: CfdScalar> MmsBuilder<R> {
    fn new(name: impl Into<String>) -> Self {
        let lift = |x: f64| R::from_f64(x).expect("a real constant lifts into every real field");
        Self {
            name: name.into(),
            regime: Regime::Incompressible,
            nu: lift(0.1),
            rho: lift(1.0),
        }
    }

    /// Select the regime whose pointwise kernel is verified.
    pub fn regime(mut self, regime: Regime) -> Self {
        self.regime = regime;
        self
    }

    /// Set the kinematic viscosity `ν` (default `0.1`).
    pub fn viscosity(mut self, nu: R) -> Self {
        self.nu = nu;
        self
    }

    /// Set the fluid density `ρ` (default `1.0`).
    pub fn density(mut self, rho: R) -> Self {
        self.rho = rho;
        self
    }

    /// Assemble and run the verification case.
    ///
    /// # Errors
    /// Any failure constructing the pointwise quantities or evaluating the regime kernel.
    pub fn run(self) -> Result<Report<R>, PhysicsError> {
        MmsCase {
            name: self.name,
            regime: self.regime,
            nu: self.nu,
            rho: self.rho,
        }
        .run()
    }
}

/// An owned MMS-verification case.
struct MmsCase<R: CfdScalar> {
    name: String,
    regime: Regime,
    nu: R,
    rho: R,
}

impl<R: CfdScalar> Solver<R> for MmsCase<R> {
    fn run(self) -> Result<Report<R>, PhysicsError> {
        let lift = |x: f64| R::from_f64(x).expect("a manufactured constant lifts into R");
        let body = AccelerationVector::<R>::new([R::zero(); 3])?;
        let rho = Density::<R>::new(self.rho)?;
        let nu = KinematicViscosity::<R>::new(self.nu)?;

        let mut report = Report::new(self.name);

        match self.regime {
            Regime::Incompressible => {
                let (u, grad_u, lap, grad_p) = taylor_green_sample::<R>(self.rho);
                let rhs = incompressible_ns_rhs(&u, &grad_u, &lap, &grad_p, &rho, &nu, &body)?
                    .into_inner();
                // Taylor–Green decay: ∂u/∂t = −2ν u.
                let two_nu = lift(2.0) * self.nu;
                let reference = scale(*u.value(), R::zero() - two_nu);
                report.add_series("mms_error", vec![l2_residual(&rhs, &reference)]);
            }
            Regime::Euler => {
                let (u, grad_u, _lap, grad_p) = taylor_green_sample::<R>(self.rho);
                let rhs = euler_momentum_rhs(&u, &grad_u, &grad_p, &rho, &body)?.into_inner();
                // Inviscid Taylor–Green balance at t = 0: ∂u/∂t = 0.
                report.add_series("mms_error", vec![l2_residual(&rhs, &[R::zero(); 3])]);
            }
            Regime::Stokes => {
                // Plane Poiseuille (unidirectional, so the convective term vanishes): μ = ρν, and
                // the pressure gradient −G balances the viscous term. The Stokes RHS reads only
                // ∇²u and ∇p, so the velocity itself is not constructed.
                let g_press = lift(100.0);
                let mu = self.rho * self.nu;
                let lap = [R::zero() - g_press / mu, R::zero(), R::zero()];
                let grad_p = [R::zero() - g_press, R::zero(), R::zero()];
                let rhs = stokes_momentum_rhs(&lap, &grad_p, &rho, &nu, &body)?.into_inner();
                // Steady fully-developed flow: ∂u/∂t = 0.
                report.add_series("mms_error", vec![l2_residual(&rhs, &[R::zero(); 3])]);
            }
            Regime::Compressible => {
                let (u, grad_u, lap, grad_p) = taylor_green_sample::<R>(self.rho);
                // Incompressible limit: ∇·τ = ρν∇²u ⇒ (1/ρ)∇·τ = ν∇²u.
                let div_tau = scale(lap, self.rho * self.nu);
                let rhs =
                    compressible_ns_momentum_rhs(&u, &grad_u, &grad_p, &div_tau, &rho, &body)?
                        .into_inner();
                let two_nu = lift(2.0) * self.nu;
                let reference = scale(*u.value(), R::zero() - two_nu);
                report.add_series("mms_error", vec![l2_residual(&rhs, &reference)]);

                // Divergence-free ⇒ continuity RHS = 0.
                //
                // `div_u` is the trace of the manufactured Jacobian, not a literal zero. Passed as
                // `R::zero()` (as it was) the kernel receives zeros for both `∇ρ` and `∇·u`, so the
                // residual is identically 0 for any implementation linear in those arguments and
                // the gate on it carried no information about the continuity assembly. Deriving
                // `∇·u` from `grad_u` makes the zero a *result*: the manufactured field being
                // divergence-free is what produces it.
                //
                // `∇ρ` stays zero because constant density is part of the manufactured solution,
                // not an assumption about the kernel.
                //
                // BREAKING CONDITION: perturb the manufactured Jacobian so its trace is non-zero,
                // or break `continuity_rhs_kernel`'s `ρ ∇·u` term, and this residual leaves 0.
                let j = grad_u.value();
                let div_u = j[0][0] + j[1][1] + j[2][2];
                let continuity = compressible_ns_continuity_rhs(&rho, &u, &[R::zero(); 3], div_u);
                report.add_series("continuity_error", vec![continuity.abs()]);
            }
        }
        Ok(report)
    }
}

/// The 2D Taylor–Green vortex sampled at `(x, y) = (π/4, π/4)`, `t = 0`:
/// `u = (½, −½, 0)`, with the analytic Jacobian, Laplacian, and pressure gradient
/// (`∇p = (ρ/2, ρ/2, 0)`). The single shared sample for the incompressible, Euler, and
/// compressible references.
fn taylor_green_sample<R: CfdScalar>(
    rho: R,
) -> (Velocity3<R>, VelocityGradient<R>, [R; 3], [R; 3]) {
    let lift = |x: f64| R::from_f64(x).expect("a Taylor–Green constant lifts into R");
    let half = lift(0.5);
    let neg_half = R::zero() - half;
    let u = Velocity3::<R>::new_unchecked([half, neg_half, R::zero()]);
    let grad_u = VelocityGradient::<R>::new_unchecked([
        [neg_half, half, R::zero()],
        [neg_half, half, R::zero()],
        [R::zero(); 3],
    ]);
    // ∇²u = (−2u, −2v, 0) = (−1, 1, 0).
    let lap = [R::zero() - R::one(), R::one(), R::zero()];
    // ∇p = (ρ/2, ρ/2, 0).
    let grad_p = [rho * half, rho * half, R::zero()];
    (u, grad_u, lap, grad_p)
}

/// `a · s` componentwise.
fn scale<R: CfdScalar>(a: [R; 3], s: R) -> [R; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// The Euclidean norm of `rhs − reference` — the MMS residual.
fn l2_residual<R: CfdScalar>(rhs: &[R; 3], reference: &[R; 3]) -> R {
    let mut sum = R::zero();
    for i in 0..3 {
        let d = rhs[i] - reference[i];
        sum += d * d;
    }
    sum.sqrt()
}
