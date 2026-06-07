/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::model;
use crate::model::Report;

pub fn print_report(r: &Report) {
    let s = &r.s1;
    println!("Stage 1: differentiate -> kernel");
    println!(
        "  u            = [{:.5}, {:.5}, {:.5}]",
        s.u[0], s.u[1], s.u[2]
    );
    println!(
        "  du/dt kernel = [{:.5}, {:.5}, {:.5}]",
        s.dudt[0], s.dudt[1], s.dudt[2]
    );
    println!(
        "  du/dt exact  = [{:.5}, {:.5}, {:.5}]",
        s.exact_dudt[0], s.exact_dudt[1], s.exact_dudt[2]
    );
    println!(
        "  max abs error: {:.2e}   (exact derivatives, no finite differences)\n",
        s.kernel_err
    );

    println!("Stage 2: Rk4 march of the amplitude, kernel in the loop");
    println!("  steps = {}, t_final = {}", r.steps, r.t_final);
    println!("  a(t) Rk4   = {:.8}", r.a_final);
    println!("  a(t) exact = {:.8}   (= exp(-2·nu·t))", r.a_exact);
    println!(
        "  abs error  = {:.2e}\n",
        model::abs_diff(r.a_final, r.a_exact)
    );

    println!("Manufactured solution reproduced: exact AD derivatives drive the kernel, Rk4 tracks");
    println!("the analytic decay, and the causal monad sequenced the two stages.");
}
