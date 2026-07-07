/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the turbulence-predictability example.
//!
//! The Lorenz system is Saltzman and Lorenz's three-mode truncation of Rayleigh-Bénard
//! convection, the minimal model of atmospheric convective turbulence. At `ρ = 28` it is chaotic,
//! with a positive leading Lyapunov exponent, so two computations that differ only in
//! floating-point precision drift apart exponentially. That is the mechanism that caps a
//! turbulence forecast: the flow magnifies the smallest error until the prediction is meaningless,
//! and the lead time before that happens grows with the number of correct digits.
//!
//! Everything here is written once over the `Scalar` bound, so the same rate field and the same
//! `Rk4` march run at `f32`, `f64`, or `Float106`. Precision is a type parameter, nothing more.

use core::ops::{Add, Mul};
use deep_causality_algebra::Real;
use deep_causality_calculus::{EndoArrow, Rk4, Scalar};
use deep_causality_num::Float106;

/// A point in the convective-flow state space, generic over the working scalar `S`.
#[derive(Clone, Copy, Default, Debug)]
pub struct Vec3<S> {
    pub x: S,
    pub y: S,
    pub z: S,
}

// The two operations `Rk4` needs from its state: vector addition and scaling by the (scalar) step.
impl<S: Scalar> Add for Vec3<S> {
    type Output = Vec3<S>;
    #[inline]
    fn add(self, o: Vec3<S>) -> Vec3<S> {
        Vec3 {
            x: self.x + o.x,
            y: self.y + o.y,
            z: self.z + o.z,
        }
    }
}

impl<S: Scalar> Mul<S> for Vec3<S> {
    type Output = Vec3<S>;
    #[inline]
    fn mul(self, s: S) -> Vec3<S> {
        Vec3 {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
}

/// Classic Lorenz / Saltzman parameters: the Rayleigh-Bénard 3-mode truncation in its chaotic
/// regime (`σ = 10`, `ρ = 28`, `β = 8/3`), where convection is turbulent.
#[derive(Clone, Copy, Debug)]
pub struct ConvectionParams {
    pub sigma: f64,
    pub rho: f64,
    pub beta: f64,
}

impl Default for ConvectionParams {
    fn default() -> Self {
        Self {
            sigma: 10.0,
            rho: 28.0,
            beta: 8.0 / 3.0,
        }
    }
}

/// Lift an exact `f64` constant into the working scalar `S` at whatever precision is in play.
fn lift<S: Scalar>(x: f64) -> S {
    S::from_f64(x).expect("constant lifts into the working scalar")
}

/// The convective rate field `dv/dt = f(v)` (the Lorenz right-hand side), as a closure over the
/// working scalar. This is the entire flow model: three lines of convection physics,
/// precision-agnostic.
pub fn convection_rate<S: Scalar>(p: &ConvectionParams) -> impl Fn(&Vec3<S>) -> Vec3<S> {
    let sigma = lift::<S>(p.sigma);
    let rho = lift::<S>(p.rho);
    let beta = lift::<S>(p.beta);
    move |v: &Vec3<S>| Vec3 {
        x: sigma * (v.y - v.x),
        y: v.x * (rho - v.z) - v.y,
        z: v.x * v.y - beta * v.z,
    }
}

/// March the convective flow at precision `S` with the `Rk4` endo-arrow, recording the state every
/// `steps_per_sample` steps. The integrator is a single arrow; the loop only takes snapshots.
pub fn run<S: Scalar>(
    p: &ConvectionParams,
    dt: f64,
    ic: [f64; 3],
    samples: usize,
    steps_per_sample: usize,
) -> Vec<Vec3<S>> {
    let stepper = Rk4::new(lift::<S>(dt), convection_rate::<S>(p));
    let mut state = Vec3 {
        x: lift::<S>(ic[0]),
        y: lift::<S>(ic[1]),
        z: lift::<S>(ic[2]),
    };
    let mut trajectory = Vec::with_capacity(samples + 1);
    trajectory.push(state);
    for _ in 0..samples {
        state = stepper.iterate_n(state, steps_per_sample);
        trajectory.push(state);
    }
    trajectory
}

/// Lift an `f32` flow state into `Float106` (exact, since `f32 ⊂ f64 ⊂ Float106`).
pub fn f32_to_106(v: Vec3<f32>) -> Vec3<Float106> {
    Vec3 {
        x: Float106::from_f64(v.x as f64),
        y: Float106::from_f64(v.y as f64),
        z: Float106::from_f64(v.z as f64),
    }
}

/// Lift an `f64` flow state into `Float106` (exact).
pub fn f64_to_106(v: Vec3<f64>) -> Vec3<Float106> {
    Vec3 {
        x: Float106::from_f64(v.x),
        y: Float106::from_f64(v.y),
        z: Float106::from_f64(v.z),
    }
}

/// Euclidean state-space distance, evaluated in `Float106` so the metric never limits resolution.
pub fn distance(a: Vec3<Float106>, b: Vec3<Float106>) -> Float106 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// The first sample time at which `traj` has separated from `reference` by more than `threshold`
/// in state space: this precision's forecast horizon for the flow.
pub fn forecast_horizon(
    traj: &[Vec3<Float106>],
    reference: &[Vec3<Float106>],
    sample_dt: f64,
    threshold: f64,
) -> Option<f64> {
    let thr = Float106::from_f64(threshold);
    traj.iter()
        .zip(reference)
        .position(|(a, b)| distance(*a, *b) > thr)
        .map(|k| k as f64 * sample_dt)
}

/// Whether every sampled state stayed finite (no overflow / NaN along the march).
pub fn all_finite(traj: &[Vec3<Float106>]) -> bool {
    traj.iter()
        .all(|v| v.x.is_finite() && v.y.is_finite() && v.z.is_finite())
}

// =============================================================================
// Result types (carried by the causal monad, rendered by print_utils)
// =============================================================================

/// Leading Lyapunov exponent of the classic Lorenz attractor, used to report the horizon law.
pub const LYAPUNOV: f64 = 0.906;

/// The three forecasts, all expressed in `Float106` for a like-for-like comparison.
#[derive(Default, Clone, Debug)]
pub struct Forecasts {
    pub f32_106: Vec<Vec3<Float106>>,
    pub f64_106: Vec<Vec3<Float106>>,
    pub ref_106: Vec<Vec3<Float106>>,
}

/// One row of the divergence table: time and the state-space distance of each precision from the
/// Float106 reference, pre-formatted.
#[derive(Default, Clone, Debug)]
pub struct Row {
    pub t: f64,
    pub d_f32: String,
    pub d_f64: String,
}

/// The analysis result: per-precision forecast horizons and the divergence table.
#[derive(Default, Clone, Debug)]
pub struct Report {
    pub h_f32: Option<f64>,
    pub h_f64: Option<f64>,
    pub rows: Vec<Row>,
}

impl Report {
    /// The horizon the digit-count law predicts for a given machine epsilon: `ln(L/ε) / λ`, with
    /// the attractor scale `L ≈ 1` at the unit threshold.
    pub fn horizon_law(epsilon: f64) -> f64 {
        -epsilon.ln() / LYAPUNOV
    }
}
