/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # QTT 2-D Taylor–Green vortex — quantized-tensor-train incompressible solver, via the CfdFlow DSL
//!
//! Verifies the new `QttIncompressible2d` solver (a 2-D incompressible Navier–Stokes flowfield that
//! lives in, and evolves as, a tensor train) against the **closed-form 2-D Taylor–Green vortex**
//! (Taylor & Green, 1937): `u = −cos(x)sin(y)`, `v = sin(x)cos(y)`, decaying as `e^{-2νt}` (the
//! nonlinear convection is a pure gradient, removed by the pressure projection).
//!
//! This `main` orchestrates the run: it builds each case through the configuration layer
//! ([`config::build_config`]) and **marches it through the CfdFlow DSL** (`CfdFlow::march`) over a
//! `2^L × 2^L` refinement ladder, then computes the convection-operator check. [`print_utils`] measures
//! the reports against the analytic reference and self-verifies four things, exiting nonzero on break:
//!
//! 1. **Convergence to the published reference** — the final-field error vs. the analytic decay
//!    strictly decreases under refinement to a pinned bound, at the expected ~2nd order.
//! 2. **Correct nonlinear convection** — because single-mode Taylor–Green's convective term is a pure
//!    gradient the projection removes, the marched decay alone cannot test it; so the solver's `u·∇u`
//!    is checked directly against the closed form `−½ sin(2x)`.
//! 3. **Incompressibility** — the post-projection divergence residual stays at the projection floor.
//! 4. **MPS compression** — the headline tensor-network metric (bond dimension vs. dense storage).
//!
//! Usage:
//!
//! ```text
//! cargo run --release -p deep_causality_cfd --example qtt_taylor_green_verification [max_level]
//! ```
//!
//! `max_level` (default 5) extends the ladder to a `2^max_level` grid. The labeled report and the
//! closing verdict are on stdout; any gate's `FAIL:` line is on stderr (exit nonzero on break).

mod config;
mod print_utils;

use deep_causality_cfd::{
    CfdFlow, PhysicsError, QttIncompressible2d, Report, dequantize_2d, quantize_2d,
};
use deep_causality_tensor::CausalTensor;

/// The working precision for the whole computation. **This is the single alias to change** (`f32`,
/// `f64`, or `Float106` with `use deep_causality_num::Float106;`). The configuration and display
/// layers import it from here.
pub type FloatType = f64;

fn main() {
    let max_level: usize = std::env::args()
        .nth(1)
        .map(|a| a.parse().expect("max_level must be an integer"))
        .unwrap_or(5);

    // The ladder starts at L=3 (8x8); `max_level` extends or truncates it (minimum two levels so the
    // observed convergence order is defined).
    let levels: Vec<usize> = (3..=max_level.max(4)).collect();

    println!("=== QTT 2-D Taylor-Green vortex: tensor-train incompressible solver ===\n");
    println!(
        "Case: nu = {}, dt = {}, steps = {}, t_final = {:.3}, precision {}\n",
        config::NU,
        config::DT,
        config::STEPS,
        config::DT * config::STEPS as f64,
        core::any::type_name::<FloatType>(),
    );

    // ── Run the refinement ladder through the CfdFlow DSL ────────────────
    //   config::build_config (configuration) ──► CfdFlow::march run ──► owned Report
    // Configuration and workflow composition are separated: the container is built by the config
    // layer, and `CfdFlow` composes + runs here in `main`.
    // ─────────────────────────────────────────────────────────────────────
    let mut runs: Vec<(usize, Report<FloatType>)> = Vec::new();
    for &l in &levels {
        let case_config =
            config::build_config(l).unwrap_or_else(|e| fail("QTT Taylor-Green config", e));
        let report = CfdFlow::march(&case_config)
            .run()
            .unwrap_or_else(|e| fail("QTT Taylor-Green pipeline", e));
        runs.push((l, report));
    }

    // Convection-operator check at the finest level — the nonlinear term single-mode TG masks.
    let finest_l = *levels.last().expect("at least one level");
    let convection =
        convection_operator_error(finest_l).unwrap_or_else(|e| fail("QTT convection check", e));

    // Measure the reports against the analytic reference, then render + self-verify.
    let measured = print_utils::measure(&runs);
    print_utils::render(&measured, convection);
    if print_utils::verify(&measured, convection) {
        print_utils::summary();
    } else {
        std::process::exit(1);
    }
}

/// **Convection-operator computation.** Runs the shipped solver's `rate_pair` on the analytic
/// Taylor–Green field and compares the `u`-component of its convection against the closed form
/// `−½ sin(2x)`. A zero-viscosity instance is used so `rate_pair` returns the pure convection
/// `−(u·∇)u`, isolating the same convection operator the `ν > 0` marcher runs. Returns
/// `(max_abs_err, amp_computed, amp_analytic)` — a small error whose *computed* amplitude matches the
/// analytic one proves the nonlinear term is real and correct. (Single-mode TG's convective term is a
/// pure gradient the projection removes, so the marched decay cannot test it.)
///
/// # Errors
/// Any codec / operator failure.
fn convection_operator_error(l: usize) -> Result<(f64, f64, f64), PhysicsError> {
    let n = 1usize << l;
    let dx = 2.0 * std::f64::consts::PI / n as f64;
    let t = config::trunc();
    let dxf = config::spacing(l);

    let mut ud = vec![config::ft(0.0); n * n];
    let mut vd = vec![config::ft(0.0); n * n];
    for i in 0..n {
        for j in 0..n {
            let (x, y) = (i as f64 * dx, j as f64 * dx);
            ud[i * n + j] = config::ft(config::tg_u(x, y));
            vd[i * n + j] = config::ft(config::tg_v(x, y));
        }
    }
    let u = quantize_2d(&CausalTensor::new(ud, vec![n, n])?, &t)?;
    let v = quantize_2d(&CausalTensor::new(vd, vec![n, n])?, &t)?;

    // Route the convection through the shipped solver, not a `gradient_x`/`gradient_y` re-assembly: a
    // re-assembly gates a copy of the operator rather than the one the marcher runs (the gate-BM-A
    // lesson, AUDIT-REPORT §5c). A zero-viscosity instance makes `rate_pair` return the pure convection
    // `−(u·∇)u` (the diffusion term vanishes), which is the same convection the ν > 0 marcher uses (ν
    // only scales the diffusion). So `ru` holds `−(u·∇)u`; the loop below negates it to read `(u·∇)u`.
    let solver =
        QttIncompressible2d::new(l, l, dxf, dxf, config::ft(config::DT), config::ft(0.0), t)?;
    let (ru, _rv) = solver.rate_pair(&u, &v)?;
    let cu = dequantize_2d(&ru, l, l)?;
    let cs = cu.as_slice();

    // `amp` must be read off `cs` — the solver's convection field — not off `analytic`. Taken from
    // the closed form it is identically 0.5 on any grid, so a gate on it would admit every possible
    // output, including an all-zero convection term. Reading it from `cs` is what makes the
    // "the nonlinear term is not a no-op" check falsifiable.
    //
    // BREAKING CONDITION: a zeroed convection in the shipped `rate_pair` collapses `amp_computed` to 0,
    // failing the amplitude-ratio gate; a sign flip is caught by the `incompressible_2d` `rate_pair`
    // unit test.
    let mut max_err = 0.0f64;
    let mut amp_computed = 0.0f64;
    let mut amp_analytic = 0.0f64;
    for i in 0..n {
        let analytic = -0.5 * (2.0 * (i as f64 * dx)).sin();
        amp_analytic = amp_analytic.max(analytic.abs());
        for j in 0..n {
            // `ru = −(u·∇)u`, so negate to read the convection `(u·∇)u` the summary reports.
            let computed = -Into::<f64>::into(cs[i * n + j]);
            max_err = max_err.max((computed - analytic).abs());
            amp_computed = amp_computed.max(computed.abs());
        }
    }
    Ok((max_err, amp_computed, amp_analytic))
}

/// Print a stage-failure context and its error on stderr, then exit the process non-zero.
fn fail(context: &str, error: impl core::fmt::Debug) -> ! {
    eprintln!("{context} failed: {error:?}");
    std::process::exit(1);
}
