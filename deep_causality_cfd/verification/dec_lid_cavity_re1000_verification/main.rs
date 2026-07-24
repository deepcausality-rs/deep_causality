/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Lid-driven cavity at Re 1000, DEC-native — via the CfdFlow DSL
//!
//! The square cavity with a moving lid is the canonical wall-bounded incompressible benchmark: the
//! steady state at Re 1000 has a primary vortex near the center and counter-rotating eddies in the
//! bottom corners, tabulated by Ghia, Ghia & Shin (1982), J. Comput. Phys. 48, 387–411 — the
//! reference every cavity solver is compared against.
//!
//! The case is declared through the `deep_causality_cfd` configuration layer
//! ([`config::build_march_config`]) and run through the **CfdFlow** DSL: an all-walls box mesh, the
//! DEC incompressible solver at `ν = U/Re`, the y-max lid, and a rest seed. `CfdFlow::march` lowers
//! onto the same projected DEC step the hand-rolled loop used, so the marched field is reproduced
//! exactly; [`print_utils`] turns it into the centerline CSVs, the Ghia-table RMSE, and the detected
//! vortex centers.
//!
//! Usage:
//!
//! ```text
//! cargo run --release -p deep_causality_cfd --example dec_lid_cavity_re1000_verification [grid] [t_end]
//! cargo run --release -p deep_causality_cfd --example dec_lid_cavity_re1000_verification trend
//! ```
//!
//! `grid` defaults to 65 (minutes of runtime); the reporting resolution is 129 with `t_end ≥ 150`
//! (hours — Ghia's own grid). The `trend` mode is the refinement-trend verification (17² → 33² at
//! time-converged horizons, gated, nonzero exit on violation) — it lives here rather than in the test
//! suite because tests stay fast by design while verification runs as long as it needs. Output:
//!
//! - `cavity_centerline_u.csv` / `cavity_centerline_v.csv` — computed centerline profiles at every
//!   grid station plus the Ghia stations with reference values and pointwise differences.
//! - stdout — the run header, the centerline RMSE, and the detected vortex centers (primary and
//!   bottom corner eddies, located at the streamfunction extrema) against Ghia's node-snapped values.

mod config;
mod print_utils;

use config::RE;
use deep_causality_cfd::{CfdFlow, EvidenceClass, StepView};
use std::env;

/// The working precision for the whole CFD computation. **This is the single alias to change**: the
/// manifold metric, the projection CG, and the DEC march all run at this precision (`f32`, `f64`, or
/// `Float106` with `use deep_causality_num::Float106;`). The Ghia-table centerline/vortex analysis
/// downcasts to `f64` at the display boundary.
pub type FloatType = f64;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.get(1).map(String::as_str) == Some("trend") {
        run_trend();
        return;
    }
    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(65);
    let t_end: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(100.0);
    let h = config::grid_spacing(n);
    let dt = config::time_step(h);
    let steps = config::step_count(t_end, dt);

    println!("# DEC lid-driven cavity, Re = {RE}");
    println!("# grid {n}x{n} (h = {h:.5}), dt = {dt:.5}, t_end = {t_end}, steps = {steps}");

    let u_form = march(n, t_end);
    print_utils::render(&u_form, n, config::ft(h));
}

/// March the Re-1000 cavity from rest on an `n × n` grid to `t_end` through the `CfdFlow` DSL; returns
/// the final velocity edge cochain at native [`FloatType`] for the Ghia-reference analysis.
fn march(n: usize, t_end: f64) -> Vec<FloatType> {
    let h = config::grid_spacing(n);
    let dt = config::time_step(h);
    let steps = config::step_count(t_end, dt);

    let case = config::build_march_config(n, h, dt, steps)
        .unwrap_or_else(|e| fail("cavity configuration", e));

    // B1: the caller owns the geometry; `CfdFlow` borrows it for the run.
    let manifold = case
        .materialize()
        .unwrap_or_else(|e| fail("cavity geometry", e));

    let report_every = (steps / 20).max(1);
    let report = CfdFlow::march(&case)
        .on(&manifold)
        .run_with(|step: &StepView<'_, 2, FloatType>| {
            let s = step.step();
            if s % report_every == 0 {
                eprintln!("# t = {:8.2} ({s}/{steps})", Into::<f64>::into(step.time()));
            }
        })
        .unwrap_or_else(|e| fail("cavity march", e));

    report
        .final_field()
        .expect("the marching report carries the final field")
        .to_vec()
}

/// Time-converged horizons; exits nonzero on a violated gate.
fn run_trend() {
    println!("# DEC lid-driven cavity, Re = {RE}: refinement trend vs Ghia (1982)");
    let mut results: Vec<(usize, FloatType)> = Vec::new();
    for n in config::TREND_GRIDS {
        let h = config::ft(config::grid_spacing(n));
        let u_form = march(n, config::TREND_T_END);
        let (u_p, v_p) = print_utils::centerline_profiles(&u_form, n, h);
        let rmse = print_utils::centerline_rmse(&u_p, &v_p, h);
        println!(
            "grid {n:>3}², t_end {}: centerline RMSE = {:.4}",
            config::TREND_T_END,
            Into::<f64>::into(rmse),
        );
        results.push((n, rmse));
    }
    // Gates from the pinning measurements (time-converged 0.252 / 0.133, ~25 % headroom) plus the
    // strict refinement-trend margin. Compared in native `FloatType` (the `f64` gates lift via `ft`).
    //
    // Evidence class: **tripwire** for all three. The two RMSE bounds carry headroom measured from
    // their own pinning run, so clearing them is evidence of non-regression, not of agreement with
    // Ghia — the Ghia table values are the reference the RMSE is *computed against*, and they are
    // reported separately by the default (non-trend) mode. The trend margin is likewise a pinned
    // strictness, not a published quantity.
    //
    // BREAKING CONDITIONS: a solver change that raises either RMSE past its bound fails gates 1-2;
    // a refinement that stops improving (or reverses) fails gate 3.
    let coarse = results[0].1;
    let fine = results[1].1;
    let mut failed = false;
    let mut gate = |label: &str, pass: bool, detail: String| {
        println!(
            "  [{}] [{}] {label}: {detail}",
            if pass { "PASS" } else { "FAIL" },
            EvidenceClass::Tripwire
        );
        if !pass {
            failed = true;
        }
    };
    println!("# refinement-trend gates (bounds pinned from this harness's own measurements)");
    gate(
        "coarse RMSE within its pinned bound",
        coarse < config::ft(config::TREND_COARSE_GATE),
        format!(
            "{}² RMSE {:.4} vs pinned {}",
            config::TREND_GRIDS[0],
            Into::<f64>::into(coarse),
            config::TREND_COARSE_GATE
        ),
    );
    gate(
        "fine RMSE within its pinned bound",
        fine < config::ft(config::TREND_FINE_GATE),
        format!(
            "{}² RMSE {:.4} vs pinned {}",
            config::TREND_GRIDS[1],
            Into::<f64>::into(fine),
            config::TREND_FINE_GATE
        ),
    );
    gate(
        "RMSE strictly decreases under refinement",
        fine < coarse - config::ft(config::TREND_MARGIN),
        format!(
            "fine {:.4} vs coarse {:.4} (margin {})",
            Into::<f64>::into(fine),
            Into::<f64>::into(coarse),
            config::TREND_MARGIN
        ),
    );
    if failed {
        std::process::exit(1);
    }
    println!("# trend holds: RMSE decreases under refinement");
}

/// Print a stage-failure context and its error on stderr, then exit the process non-zero.
pub(crate) fn fail(context: &str, error: impl core::fmt::Debug) -> ! {
    eprintln!("{context} failed: {error:?}");
    std::process::exit(1);
}
