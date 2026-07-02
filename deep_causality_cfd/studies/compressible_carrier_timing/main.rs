// SPDX-License-Identifier: MIT
// Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

//! Compressible-carrier timing study (Task 0 of `add-compressible-blackout-carrier`): can the
//! body-fitted 3-D compressible marcher carry the plasma-blackout corridor inside the
//! minutes-not-hours budget?
//!
//! Everything downstream hangs on two numbers this study measures per configuration:
//!
//! 1. **Per-step wall-clock** of `CompressibleMarcher3dFitted::step` on the body-fitted shell —
//!    the corridor marches roughly `CORRIDOR_STEPS` coupled steps (legs plus the branch study),
//!    so the projected corridor time is `per_step × CORRIDOR_STEPS` plus coupling overhead.
//! 2. **Assembly cost** of the marcher (metric + acoustic inverse) — the continuous-descent host
//!    rebuilds the solver when the scheduled freestream drifts, so the rebuild budget in
//!    "equivalent steps" sets the freestream-drift tolerance.
//!
//! The study sweeps the shell at two resolutions and two bond caps, prints projections against
//! the budget, and self-verifies: at least one configuration must fit, and the recommended
//! configuration (the largest one inside the budget) is printed as the go/no-go record. A
//! regression (no configuration fits) exits nonzero, which is the documented trigger for the
//! `CompressibleMarcher2d` fallback decision.

use deep_causality_cfd::{
    BodyFittedCoordinate3d, CartesianIdentity, CompressibleMarcher2d, CompressibleMarcher3dFitted,
    EulerState3d, EulerStateTt2d, EulerStateTt3d, Marcher, quantize_2d, quantize_3d,
};
use deep_causality_tensor::{CausalTensor, Truncation};
use std::time::Instant;

const TAU: f64 = std::f64::consts::TAU;
const GAMMA: f64 = 1.4;
/// The corridor's step budget: three leg spans plus the counterfactual branch study.
const CORRIDOR_STEPS: usize = 200;
/// The minutes-not-hours budget for one corridor run, in seconds.
const BUDGET_S: f64 = 600.0;
/// Steps timed per configuration (after one untimed warmup step).
const TIMED_STEPS: usize = 5;

fn main() {
    println!(
        "=== Compressible-carrier timing (Task 0): 3-D fitted shell vs the 2-D fallback ===\n"
    );
    println!(
        "Budget: {CORRIDOR_STEPS} corridor steps inside {:.0} s ({:.0} min); {TIMED_STEPS} timed steps per case.\n",
        BUDGET_S,
        BUDGET_S / 60.0
    );

    let mut results = Vec::new();

    println!(
        "  case          grid    bond cap   assembly    per-step     projected corridor   verdict"
    );
    // The 3-D fitted shell: measured at the smallest candidate only — the 16^3 numbers already
    // decide the family (over budget by >3x), so larger grids are a foregone conclusion.
    report_case("3d-fitted", 4, 3, 16, measure_3d(4, 16), &mut results);
    // The 2-D compressible fallback: the corridor's stage stack is 2-D already, so this carrier
    // is a like-for-like upgrade of the shipped incompressible corridor.
    for (l, cap) in [(5usize, 16usize), (5, 32), (6, 16), (6, 32)] {
        report_case("2d", l, 2, cap, measure_2d(l, cap), &mut results);
    }
    println!();

    // The rebuild budget: how many equivalent steps one solver rebuild costs, per configuration.
    println!(
        "  Rebuild guidance (assembly cost in equivalent steps -> freestream-drift tolerance):"
    );
    for (family, l, dim, cap, m, _) in &results {
        let eq_steps = m.assembly_s / m.per_step_s;
        println!(
            "    {family:<9} {n:>3}^{dim} cap {cap:>3}: one rebuild ~ {eq:.2} steps; ~10 rebuilds/run adds {add:.2}% to the march",
            n = 1usize << l,
            eq = eq_steps,
            add = 100.0 * 10.0 * eq_steps / CORRIDOR_STEPS as f64,
        );
    }
    println!();

    // The go/no-go record: the largest configuration inside the budget, 3-D preferred.
    let pick = results
        .iter()
        .filter(|(_, _, _, _, _, fits)| *fits)
        .max_by_key(|(_, l, dim, cap, _, _)| (*dim, *l, *cap));
    match pick {
        Some((family, l, dim, cap, m, _)) => {
            println!(
                "=== GO: corridor carrier {family} at {n}^{dim}, bond cap {cap} ({step:.3} s/step, peak bond {bond}). ===",
                n = 1usize << l,
                step = m.per_step_s,
                bond = m.peak_bond,
            );
        }
        None => {
            println!("=== NO-GO: no configuration fits the budget. ===");
            std::process::exit(1);
        }
    }
}

type CaseRow = (&'static str, usize, usize, usize, Measurement, bool);

fn report_case(
    family: &'static str,
    l: usize,
    dim: usize,
    cap: usize,
    measured: Result<Measurement, deep_causality_cfd::PhysicsError>,
    results: &mut Vec<CaseRow>,
) {
    match measured {
        Ok(m) => {
            let projected = m.per_step_s * CORRIDOR_STEPS as f64;
            let fits = projected <= BUDGET_S;
            println!(
                "  {family:<11}  {n:>3}^{dim}    {cap:>4}       {asm:>7.3} s   {step:>7.3} s    {proj:>7.1} s ({mins:>5.1} min)   {verdict}",
                n = 1usize << l,
                asm = m.assembly_s,
                step = m.per_step_s,
                proj = projected,
                mins = projected / 60.0,
                verdict = if fits { "fits" } else { "over budget" },
            );
            results.push((family, l, dim, cap, m, fits));
        }
        Err(e) => {
            println!(
                "  {family:<11}  {n:>3}^{dim}    {cap:>4}       FAILED: {e}",
                n = 1usize << l
            );
        }
    }
}

struct Measurement {
    assembly_s: f64,
    per_step_s: f64,
    peak_bond: usize,
}

/// Assemble the 2-D compressible marcher at `2^l` per axis with bond cap `cap`, then time
/// `TIMED_STEPS` steps of a smooth freestream-plus-bump state (one untimed warmup step first).
fn measure_2d(l: usize, cap: usize) -> Result<Measurement, deep_causality_cfd::PhysicsError> {
    let tr = Truncation::<f64>::by_bond(cap)?;
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let (dt, s_ref) = (0.002, 1.3);

    let t0 = Instant::now();
    let metric = CartesianIdentity::<f64>::new(l, l, dx, dx, tr)?;
    let marcher = CompressibleMarcher2d::new(metric, GAMMA, dt, s_ref, tr)?;
    let assembly_s = t0.elapsed().as_secs_f64();

    // A smooth, positive 2-D state: unit freestream with a sinusoidal density bump.
    let tot = n * n;
    let (mut rho, mut mx, mut my, mut e) = (
        Vec::with_capacity(tot),
        Vec::with_capacity(tot),
        Vec::with_capacity(tot),
        Vec::with_capacity(tot),
    );
    let (u, v) = (0.1, -0.05);
    for i in 0..n {
        for j in 0..n {
            let (x, y) = (i as f64 / n as f64, j as f64 / n as f64);
            let d = 1.0 + 0.2 * (TAU * x).sin() * (TAU * y).sin();
            rho.push(d);
            mx.push(d * u);
            my.push(d * v);
            e.push(1.0 / (GAMMA - 1.0) + 0.5 * d * (u * u + v * v));
        }
    }
    let encode =
        |vv: &Vec<f64>| quantize_2d(&CausalTensor::new(vv.clone(), vec![n, n]).unwrap(), &tr);
    let mut state: EulerStateTt2d<f64> = [encode(&rho)?, encode(&mx)?, encode(&my)?, encode(&e)?];

    // Warmup (excluded): the first step pays one-off layout costs.
    state = marcher.advance(&state, &())?;

    let mut peak_bond = 0usize;
    let t1 = Instant::now();
    for _ in 0..TIMED_STEPS {
        state = marcher.advance(&state, &())?;
        let b = state.iter().map(|t| t.max_bond()).max().unwrap_or(0);
        if b > peak_bond {
            peak_bond = b;
        }
    }
    let per_step_s = t1.elapsed().as_secs_f64() / TIMED_STEPS as f64;

    Ok(Measurement {
        assembly_s,
        per_step_s,
        peak_bond,
    })
}

/// Assemble the shell marcher at `2^l` per axis with bond cap `cap`, then time `TIMED_STEPS`
/// steps of a smooth freestream-plus-bump state (one untimed warmup step first).
fn measure_3d(l: usize, cap: usize) -> Result<Measurement, deep_causality_cfd::PhysicsError> {
    let tr = Truncation::<f64>::by_bond(cap)?;
    let dx = 1.0 / (1usize << l) as f64;
    let (dt, s_ref) = (0.002, 1.3);

    let t0 = Instant::now();
    let shell = BodyFittedCoordinate3d::<f64>::new(l, l, l, 0.5, 1.0, 0.4, 1.5, 0.0, TAU, tr)?;
    let marcher = CompressibleMarcher3dFitted::new(shell, dx, GAMMA, dt, s_ref, tr)?;
    let assembly_s = t0.elapsed().as_secs_f64();

    let state = shell_state(l);
    let n = 1usize << l;
    let encode =
        |v: &Vec<f64>| quantize_3d(&CausalTensor::new(v.clone(), vec![n, n, n]).unwrap(), &tr);
    let mut u: EulerStateTt3d<f64> = [
        encode(&state[0])?,
        encode(&state[1])?,
        encode(&state[2])?,
        encode(&state[3])?,
        encode(&state[4])?,
    ];

    // Warmup (excluded): the first step pays one-off layout costs.
    u = marcher.advance(&u, &())?;

    let mut peak_bond = 0usize;
    let t1 = Instant::now();
    for _ in 0..TIMED_STEPS {
        u = marcher.advance(&u, &())?;
        let b = u.iter().map(|t| t.max_bond()).max().unwrap_or(0);
        if b > peak_bond {
            peak_bond = b;
        }
    }
    let per_step_s = t1.elapsed().as_secs_f64() / TIMED_STEPS as f64;

    Ok(Measurement {
        assembly_s,
        per_step_s,
        peak_bond,
    })
}

/// A smooth, positive state on the shell lattice: unit freestream density/pressure with a radial
/// bump — the same family the fitted-marcher gates march.
fn shell_state(l: usize) -> EulerState3d<f64> {
    let n = 1usize << l;
    let tot = n * n * n;
    let mut rho = Vec::with_capacity(tot);
    let mut mx = Vec::with_capacity(tot);
    let mut my = Vec::with_capacity(tot);
    let mut mz = Vec::with_capacity(tot);
    let mut e = Vec::with_capacity(tot);
    let (u, v, w) = (0.1, -0.05, 0.05);
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let (x, y, z) = (
                    i as f64 / n as f64,
                    j as f64 / n as f64,
                    k as f64 / n as f64,
                );
                let d = 1.0 + 0.2 * (TAU * x).sin() * (TAU * y).sin() * (TAU * z).sin();
                rho.push(d);
                mx.push(d * u);
                my.push(d * v);
                mz.push(d * w);
                e.push(1.0 / (GAMMA - 1.0) + 0.5 * d * (u * u + v * v + w * w));
            }
        }
    }
    [rho, mx, my, mz, e]
}
