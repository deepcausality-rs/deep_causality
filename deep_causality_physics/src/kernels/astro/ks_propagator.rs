/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Exact 3-D two-body propagation by **Kustaanheimo–Stiefel (KS) regularisation** — the
//! singularity-free, perturbation-ready generalisation of the planar
//! [`TwoBodyPropagator`](crate::TwoBodyPropagator) (FS-1). This is the B1 conformal core of the
//! perturbed-conformal trajectory split (Gap-3 Resolution 1); the plasma-blackout reentry arc is
//! highly eccentric / near-radial, exactly where an orbital-elements propagator is ill-conditioned and
//! KS is not.
//!
//! # Method
//! KS lifts the 3-D Kepler position `r ∈ ℝ³` to a 4-vector `u ∈ ℝ⁴` (`|r| = |u|²`) and reparametrises
//! time by the fictitious time `s` with `dt = |u|² ds`. In `s` the **bound** motion is the constant
//! 4-D harmonic oscillator `u'' + ω₀² u = 0`, `ω₀² = −h/2` (`h < 0` the specific energy), so
//! `u(s) = u₀·cos(ω₀ s) + (u₀'/ω₀)·sin(ω₀ s)` — a constant-generator matrix exponential, exact to
//! round-off. Physical time is the closed-form integral `t(s) = ∫₀ˢ |u|² ds'`; a monotone Newton solve
//! inverts `t(s) = dt` (a well-conditioned Kepler-equation analogue).
//!
//! # References
//! * Stiefel, E. L. & Scheifele, G., *Linear and Regular Celestial Mechanics*, Springer (1971).
//! * Battin, R. H., *An Introduction to the Mathematics and Methods of Astrodynamics*, AIAA (1999).

use crate::PhysicsError;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// An exact 3-D two-body (Kepler) propagator via KS regularisation, built from a single physical
/// state. Propagation is the constant-generator matrix exponential in the KS fictitious time; it is
/// exact to floating-point round-off for any `dt`, with no step-size error.
#[derive(Clone, Copy, Debug)]
pub struct KsPropagator<R> {
    gravitational_parameter: R,
    /// KS oscillator frequency `ω₀ = √(−h/2)`.
    omega: R,
    /// The epoch KS coordinate `A = u₀` (with `|A|² = |r₀|`).
    a: [R; 4],
    /// The scaled epoch KS velocity `B = u₀'/ω₀`.
    b: [R; 4],
    /// Cached `|A|²`, `|B|²`, `A·B` for the closed-form `t(s)`.
    aa: R,
    bb: R,
    ab: R,
    /// Semi-major axis `a = μ/(4 ω₀²)`.
    semi_major_axis: R,
}

impl<R> KsPropagator<R>
where
    R: RealField + FromPrimitive,
{
    /// Build the propagator from a 3-D physical state `(position, velocity)` about a primary with
    /// gravitational parameter `gm = GM`.
    ///
    /// # Errors
    /// [`PhysicsError::PhysicalInvariantBroken`] if `gm ≤ 0` or the state is not bound (`energy ≥ 0`);
    /// [`PhysicsError::Singularity`] on a non-positive radius; numeric-conversion failures.
    pub fn from_state(position: [R; 3], velocity: [R; 3], gm: R) -> Result<Self, PhysicsError> {
        if gm <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Gravitational parameter GM must be positive".into(),
            ));
        }
        let two = Self::lit(2.0)?;
        let four = Self::lit(4.0)?;

        let radius =
            (position[0] * position[0] + position[1] * position[1] + position[2] * position[2])
                .sqrt();
        if radius <= R::zero() {
            return Err(PhysicsError::Singularity("Radius must be positive".into()));
        }
        let v2 = velocity[0] * velocity[0] + velocity[1] * velocity[1] + velocity[2] * velocity[2];
        let energy = v2 / two - gm / radius;
        if energy >= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "State is not a bound orbit (energy >= 0); only ellipses are supported".into(),
            ));
        }
        let omega = (-energy / two).sqrt();
        let semi_major_axis = gm / (four * omega * omega);

        // KS lift r -> u (gauge branch keeps the leading component well away from zero).
        let a = Self::ks_lift(position, radius);
        // u' = ½ L(u)ᵀ (v; 0); then B = u'/ω₀.
        let uprime = Self::l_transpose_times_v(&a, velocity);
        let half = R::one() / two;
        let b = [
            uprime[0] * half / omega,
            uprime[1] * half / omega,
            uprime[2] * half / omega,
            uprime[3] * half / omega,
        ];

        let aa = Self::dot4(&a, &a);
        let bb = Self::dot4(&b, &b);
        let ab = Self::dot4(&a, &b);

        Ok(Self {
            gravitational_parameter: gm,
            omega,
            a,
            b,
            aa,
            bb,
            ab,
            semi_major_axis,
        })
    }

    /// The state `(position, velocity)` at time `dt` after the epoch — exact for any `dt`.
    ///
    /// # Errors
    /// Numeric-conversion failures.
    pub fn propagate(&self, dt: R) -> Result<([R; 3], [R; 3]), PhysicsError> {
        let s = self.solve_fictitious_time(dt)?;
        let phi = self.omega * s;
        let (cp, sp) = (phi.cos(), phi.sin());

        // u(s) and u'(s).
        let mut u = [R::zero(); 4];
        let mut up = [R::zero(); 4];
        for i in 0..4 {
            u[i] = self.a[i] * cp + self.b[i] * sp;
            up[i] = self.omega * (-self.a[i] * sp + self.b[i] * cp);
        }
        let radius = Self::dot4(&u, &u);

        // r = first three components of L(u)·u; dr/ds = 2·L(u)·u'; v = (dr/ds)/radius.
        let r3 = Self::l_times(&u, &u);
        let drds = Self::l_times(&u, &up);
        let two = Self::lit(2.0)?;
        let velocity = [
            two * drds[0] / radius,
            two * drds[1] / radius,
            two * drds[2] / radius,
        ];
        Ok((r3, velocity))
    }

    /// Semi-major axis `a` (m).
    pub fn semi_major_axis(&self) -> R {
        self.semi_major_axis
    }
    /// Gravitational parameter `GM` (m³/s²).
    pub fn gravitational_parameter(&self) -> R {
        self.gravitational_parameter
    }
    /// Mean motion `n = √(GM/a³)` (rad/s).
    pub fn mean_motion(&self) -> R {
        (self.gravitational_parameter
            / (self.semi_major_axis * self.semi_major_axis * self.semi_major_axis))
            .sqrt()
    }
    /// Orbital period `T = 2π/n` (s).
    ///
    /// # Errors
    /// Numeric-conversion failures.
    pub fn period(&self) -> Result<R, PhysicsError> {
        let two = Self::lit(2.0)?;
        Ok(two * R::pi() / self.mean_motion())
    }

    // ── helpers ─────────────────────────────────────────────────────────────────────────────────────

    /// `t(s) = ∫₀ˢ |u|² ds'` and the radius `|u(s)|²` at `s` — the value and derivative for Newton.
    fn t_and_radius(&self, s: R) -> Result<(R, R), PhysicsError> {
        let two = Self::lit(2.0)?;
        let four = Self::lit(4.0)?;
        let phi = self.omega * s;
        let (cp, sp) = (phi.cos(), phi.sin());
        let sin2 = two * sp * cp; // sin 2φ
        let cos2 = cp * cp - sp * sp; // cos 2φ
        let c_lin = (self.aa + self.bb) / two;
        let t = c_lin * s
            + (self.aa - self.bb) * sin2 / (four * self.omega)
            + self.ab * (R::one() - cos2) / (two * self.omega);
        let radius = self.aa * cp * cp + two * self.ab * cp * sp + self.bb * sp * sp;
        Ok((t, radius))
    }

    /// Invert `t(s) = dt` by a monotone Newton iteration (`dt/ds = |u|² > 0`).
    fn solve_fictitious_time(&self, dt: R) -> Result<R, PhysicsError> {
        let two = Self::lit(2.0)?;
        let c_lin = (self.aa + self.bb) / two; // the s-average of |u|²; exact asymptotic slope
        let mut s = dt / c_lin;
        let tol = Self::lit(1e-15)?;
        for _ in 0..100 {
            let (t, radius) = self.t_and_radius(s)?;
            let d = (t - dt) / radius;
            s -= d;
            if d.abs() < tol * (s.abs() + R::one()) {
                break;
            }
        }
        Ok(s)
    }

    /// KS lift `r → u` with a gauge choice that keeps the pivot component large (Stiefel–Scheifele).
    fn ks_lift(r: [R; 3], radius: R) -> [R; 4] {
        let two = Self::lit(2.0).expect("2.0 lifts into every real field");
        let half = R::one() / two;
        if r[0] >= R::zero() {
            let u1 = (half * (radius + r[0])).sqrt();
            [u1, r[1] / (two * u1), r[2] / (two * u1), R::zero()]
        } else {
            let u2 = (half * (radius - r[0])).sqrt();
            [r[1] / (two * u2), u2, R::zero(), r[2] / (two * u2)]
        }
    }

    /// First three components of `L(u)·w` (the KS matrix acting on a 4-vector).
    fn l_times(u: &[R; 4], w: &[R; 4]) -> [R; 3] {
        [
            u[0] * w[0] - u[1] * w[1] - u[2] * w[2] + u[3] * w[3],
            u[1] * w[0] + u[0] * w[1] - u[3] * w[2] - u[2] * w[3],
            u[2] * w[0] + u[3] * w[1] + u[0] * w[2] + u[1] * w[3],
        ]
    }

    /// `L(u)ᵀ · (v; 0)` — used to recover the KS velocity from the physical velocity.
    fn l_transpose_times_v(u: &[R; 4], v: [R; 3]) -> [R; 4] {
        [
            u[0] * v[0] + u[1] * v[1] + u[2] * v[2],
            -u[1] * v[0] + u[0] * v[1] + u[3] * v[2],
            -u[2] * v[0] - u[3] * v[1] + u[0] * v[2],
            u[3] * v[0] - u[2] * v[1] + u[1] * v[2],
        ]
    }

    fn dot4(x: &[R; 4], y: &[R; 4]) -> R {
        x[0] * y[0] + x[1] * y[1] + x[2] * y[2] + x[3] * y[3]
    }

    fn lit(x: f64) -> Result<R, PhysicsError> {
        R::from_f64(x)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64 failed".into()))
    }
}

/// One **Strang-split** perturbed step (the B1 between-step perturbation hook): half-kick the velocity
/// with the caller-supplied non-conformal Cartesian acceleration, drift exactly along the KS conformal
/// core for `dt`, then half-kick again. The perturbation is never expressed inside the KS algebra
/// (FS-2). 2nd-order accurate; the exact core is untouched when `accel` returns zero.
///
/// `accel(position, velocity) -> acceleration` is the non-conformal force per unit mass (e.g. the ④
/// aero force + J2).
///
/// # Errors
/// As [`KsPropagator::from_state`] / [`KsPropagator::propagate`] (the half-kicked state must stay bound).
pub fn ks_strang_step<R, A>(
    position: [R; 3],
    velocity: [R; 3],
    gm: R,
    dt: R,
    accel: A,
) -> Result<([R; 3], [R; 3]), PhysicsError>
where
    R: RealField + FromPrimitive,
    A: Fn([R; 3], [R; 3]) -> [R; 3],
{
    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64 failed".into()))?;
    let half_dt = dt / two;

    // Half-kick in physical Cartesian velocity.
    let a0 = accel(position, velocity);
    let v_half = [
        velocity[0] + a0[0] * half_dt,
        velocity[1] + a0[1] * half_dt,
        velocity[2] + a0[2] * half_dt,
    ];

    // Exact conformal drift.
    let (r1, v1) = KsPropagator::from_state(position, v_half, gm)?.propagate(dt)?;

    // Half-kick.
    let a1 = accel(r1, v1);
    let v_out = [
        v1[0] + a1[0] * half_dt,
        v1[1] + a1[1] * half_dt,
        v1[2] + a1[2] * half_dt,
    ];
    Ok((r1, v_out))
}
