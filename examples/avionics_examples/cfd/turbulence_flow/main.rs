/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Turbulence predictability in avionics: the forecast horizon of a chaotic flow
//!
//! Avionics must reckon with turbulence (atmospheric convection, thermals, storm cells, wake and
//! clear-air turbulence) for structural loads, ride quality, and control. Turbulent flow is
//! chaotic, so any forecast of it has a hard predictability horizon: past some lead time the
//! prediction is worthless however good the model, because the flow amplifies the smallest error
//! exponentially. This example locates that horizon and shows what moves it.
//!
//! The testbed is the Lorenz system, Saltzman and Lorenz's three-mode truncation of
//! Rayleigh-Bénard convection. It is the original model of atmospheric convective turbulence and
//! the birthplace of chaos theory: small enough to run in a few lines, yet exhibiting the property
//! that caps every turbulence forecast, exponential error growth (leading Lyapunov exponent
//! `λ ≈ 0.906`).
//!
//! The avionics-relevant result is that the horizon is set by the smallest error in the
//! computation, and the irreducible floor is floating-point roundoff. How far ahead a chaotic flow
//! can be trusted is therefore, in the end, a question of arithmetic precision: `Float106` roughly
//! doubles the trustworthy horizon that `f64` can reach.
//!
//! The example rests on three DeepCausality pillars and little else. The integration operator
//! `Rk4` marches the flow; the model is written once over `Scalar`, so precision is a type
//! parameter (`f32` / `f64` / `Float106`); and the causal monad `PropagatingEffect` sequences
//! simulate then analyse, short-circuiting if a trajectory leaves the finite range.

mod model;
mod print_utils;

use deep_causality_core::{CausalFlow, CausalityError, CausalityErrorEnum, PropagatingEffect};
use deep_causality_num::Float106;
use model::{ConvectionParams, Forecasts, Report, Row, Vec3};

fn main() {
    println!(
        "=== Turbulence predictability: the forecast horizon of a chaotic convective flow ===\n"
    );

    let params = ConvectionParams::default();
    let dt = 0.005_f64; // Rk4 step
    let ic = [1.0_f64, 1.0, 1.0]; // exactly representable, so all precisions start identical
    let sample_dt = 0.5_f64; // record the state twice per time unit
    let samples = 120usize; // march to T = 60
    let steps_per_sample = (sample_dt / dt) as usize; // 100 steps between snapshots

    // The workflow is a causal-monad chain: forecast at three precisions, then (only if every
    // trajectory stayed finite) analyse the divergence. A blow-up short-circuits the error channel.
    CausalFlow::effect()
        .next(move |_| simulate(&params, dt, ic, samples, steps_per_sample).into())
        .next(move |sims| analyse(sims, sample_dt).into())
        .run(
            |report| print_utils::print_report(&report),
            |err| eprintln!("Turbulence-forecast pipeline failed: {err:?}"),
        );
}

/// Stage 1: forecast the same flow at f32, f64, and Float106, then lift the two low-precision
/// trajectories into Float106 so every comparison happens in the widest type.
fn simulate(
    p: &ConvectionParams,
    dt: f64,
    ic: [f64; 3],
    samples: usize,
    steps_per_sample: usize,
) -> PropagatingEffect<Forecasts> {
    let f32_traj = model::run::<f32>(p, dt, ic, samples, steps_per_sample);
    let f64_traj = model::run::<f64>(p, dt, ic, samples, steps_per_sample);
    let ref_traj = model::run::<Float106>(p, dt, ic, samples, steps_per_sample);

    let f32_106: Vec<Vec3<Float106>> = f32_traj.into_iter().map(model::f32_to_106).collect();
    let f64_106: Vec<Vec3<Float106>> = f64_traj.into_iter().map(model::f64_to_106).collect();

    if !model::all_finite(&ref_traj) {
        return fail("a trajectory diverged to a non-finite state");
    }

    PropagatingEffect::pure(Forecasts {
        f32_106,
        f64_106,
        ref_106: ref_traj,
    })
}

/// Stage 2: measure each precision's forecast horizon (where it parts from the Float106 trajectory
/// by more than one state-space unit) and tabulate the divergence over time.
fn analyse(s: Forecasts, sample_dt: f64) -> PropagatingEffect<Report> {
    let threshold = 1.0;
    let h_f32 = model::forecast_horizon(&s.f32_106, &s.ref_106, sample_dt, threshold);
    let h_f64 = model::forecast_horizon(&s.f64_106, &s.ref_106, sample_dt, threshold);

    let mut rows = Vec::new();
    let mut t = 5.0_f64;
    while t <= 55.0 {
        let k = (t / sample_dt) as usize;
        if k < s.ref_106.len() {
            rows.push(Row {
                t,
                d_f32: format!("{:.2e}", model::distance(s.f32_106[k], s.ref_106[k])),
                d_f64: format!("{:.2e}", model::distance(s.f64_106[k], s.ref_106[k])),
            });
        }
        t += 5.0;
    }

    PropagatingEffect::pure(Report { h_f32, h_f64, rows })
}

fn fail<T: Default + Clone + std::fmt::Debug>(msg: &str) -> PropagatingEffect<T> {
    PropagatingEffect::from_error(CausalityError::new(CausalityErrorEnum::Custom(msg.into())))
}
