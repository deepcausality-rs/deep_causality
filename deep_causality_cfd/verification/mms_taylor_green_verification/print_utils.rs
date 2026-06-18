/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Display layer: render the `CfdFlow::verify` report (the stage-1 derivatives + kernel residual,
//! and the stage-2 amplitude march) at the working precision [`FloatType`].

use crate::FloatType;
use crate::config;
use deep_causality_cfd::Report;

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
