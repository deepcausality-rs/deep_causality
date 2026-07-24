/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Display layer: render the `CfdFlow::verify` report (the stage-1 derivatives + kernel residual,
//! and the stage-2 amplitude march) at the working precision [`FloatType`].

use crate::FloatType;
use crate::config;
use deep_causality_cfd::{EvidenceClass, Report};

/// Render the two-stage manufactured-solution verification report.
pub fn render(r: &Report<FloatType>) {
    let u = r.series("velocity").expect("velocity series");
    let kernel = r.series("kernel_dudt").expect("kernel_dudt series");
    let exact = r.series("exact_dudt").expect("exact_dudt series");
    let kernel_err = r.series("mms_error").expect("mms_error series")[0];

    println!("Stage 1: differentiate -> kernel");
    println!("  u            = [{:.5}, {:.5}, {:.5}]", u[0], u[1], u[2]);
    println!(
        "  du/dt kernel = [{:.5}, {:.5}, {:.5}]",
        kernel[0], kernel[1], kernel[2]
    );
    println!(
        "  du/dt exact  = [{:.5}, {:.5}, {:.5}]",
        exact[0], exact[1], exact[2]
    );
    println!("  max abs error: {kernel_err:.2e}   (exact derivatives, no finite differences)\n");

    let a_final = r.series("amplitude_final").expect("amplitude_final series")[0];
    let a_exact = r.series("amplitude_exact").expect("amplitude_exact series")[0];
    let t_final = config::ft(config::DT) * config::ft(config::STEPS as f64);

    println!("Stage 2: Rk4 march of the amplitude, kernel in the loop");
    println!("  steps = {}, t_final = {t_final}", config::STEPS);
    println!("  a(t) Rk4   = {a_final:.8}");
    println!("  a(t) exact = {a_exact:.8}   (= exp(-2·nu·t))");
    println!("  abs error  = {:.2e}\n", (a_final - a_exact).abs());

    println!("Manufactured solution reproduced: exact AD derivatives drive the kernel, Rk4 tracks");
    println!("the analytic decay, and the causal monad sequenced the two stages.");
}

/// Precision-scaled bound on the MMS residuals.
///
/// Both stages compare against a **closed form** — the exact Taylor-Green `du/dt` and the analytic
/// amplitude decay `exp(-2*nu*t)` — with exact autodiff derivatives, so a correct kernel lands at
/// the working precision's floor. The bound is `1e4 * epsilon` of `FloatType`, which tracks the
/// alias rather than pinning an `f64` number: ~3e-12 at f32, ~2e-12 at f64, and correspondingly
/// tighter at Float106. Measured at f64: stage 1 = 1.11e-16, stage 2 = 6.66e-16, so ~4 orders of
/// headroom.
fn residual_bound() -> f64 {
    // `Into` (not `f64::from`) is the crate-wide FloatType -> f64 display-boundary conversion, and
    // it keeps working if the alias changes to f32 or Float106.
    1.0e4 * Into::<f64>::into(FloatType::EPSILON)
}

/// Self-verification (exit nonzero on break): both manufactured-solution residuals sit at the
/// working precision's floor.
///
/// This harness previously reported the residuals and exited zero unconditionally — its only
/// `process::exit` was the setup-error helper — so a kernel regression would have printed a large
/// number and still passed. verification/README.md's stated convention ("exits with a nonzero
/// status the moment its invariant or reference check fails") did not hold here.
///
/// BREAKING CONDITIONS: perturb the Navier-Stokes kernel (or the exact-derivative supply) and the
/// stage-1 residual leaves the floor; break the Rk4 tableau and stage 2 does.
pub fn verify(r: &Report<FloatType>) -> bool {
    let kernel_err: f64 = r.series("mms_error").expect("mms_error series")[0];
    let a_final = r.series("amplitude_final").expect("amplitude_final series")[0];
    let a_exact = r.series("amplitude_exact").expect("amplitude_exact series")[0];
    let amp_err = Into::<f64>::into(a_final - a_exact).abs();

    let bound = residual_bound();
    let mut ok = true;
    println!("\n--- manufactured-solution gates ---");
    for (label, err) in [
        ("stage 1: kernel vs exact du/dt", kernel_err),
        ("stage 2: Rk4 vs analytic decay", amp_err),
    ] {
        let pass = err <= bound;
        println!(
            "  [{}] [{}] {label}: {err:.3e} vs bound {bound:.3e} (closed form)",
            if pass { "PASS" } else { "FAIL" },
            EvidenceClass::Reference,
        );
        ok &= pass;
    }
    ok
}
