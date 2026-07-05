/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # QTT immersed cylinder — Brinkman-penalized drag, via the CfdFlow DSL
//!
//! Verifies the immersed-body QTT solver (`QttImmersed2d`): a cylinder in a periodic free-stream,
//! enforced by **Brinkman volume penalization** (a smoothed mask drives the velocity to zero inside the
//! body — no cut cells), with drag read as a **tensor-train contraction** of the mask with the velocity
//! deficit. This closes Gap 1 of the plasma-blackout analysis (the immersed body + surface observables).
//!
//! `main` runs the case through `CfdFlow::march` at a ladder of **bond caps** and self-verifies
//! (exit nonzero on break):
//!
//! 1. **No-slip** — the velocity inside the body falls to the penalization floor.
//! 2. **Accuracy vs bond** — the drag coefficient converges as the tensor-train is allowed more rank
//!    (the headline QTT-CFD metric).
//! 3. **Physical drag** — the streamwise drag is positive and `O(1)`.
//!
//! The committed DEC isolated-cylinder `C_d` is reported as a **cross-reference**, disclaimed for the
//! periodic-blockage difference (the periodic penalized box is not the DEC inflow/outflow configuration,
//! so an absolute match is not claimed).
//!
//! Usage:
//!
//! ```text
//! cargo run --release -p deep_causality_cfd --example qtt_cylinder_verification
//! ```

mod config;
mod print_utils;

use deep_causality_cfd::CfdFlow;
use print_utils::BondRow;

/// The working precision for the whole computation (the single alias to change).
pub type FloatType = f64;

/// The grid: `2^L × 2^L` (32² — affordable, resolves the smoothed cylinder).
const L: usize = 5;

fn main() {
    println!("=== QTT immersed cylinder: Brinkman-penalized drag (tensor-train) ===\n");
    println!(
        "Case: nu = {}, dt = {}, steps = {}, eta = {}, U = {}, grid {}^2, precision {}\n",
        config::NU,
        config::DT,
        config::STEPS,
        config::ETA,
        config::U_INF,
        1usize << L,
        core::any::type_name::<FloatType>(),
    );

    // Accuracy-vs-bond ladder: the same case at increasing round bond caps.
    let caps = [4usize, 8, 16, 24];
    let mut rows = Vec::new();
    for &cap in &caps {
        let case = config::build_config(L, cap).unwrap_or_else(|e| fail("QTT cylinder config", e));
        let report = CfdFlow::march(&case)
            .run()
            .unwrap_or_else(|e| fail("QTT cylinder pipeline", e));
        let drag = Into::<f64>::into(*report.series("drag").expect("drag series").last().unwrap());
        let divergence = Into::<f64>::into(
            *report
                .series("divergence")
                .expect("divergence")
                .last()
                .unwrap(),
        );
        let interior_max_speed = print_utils::interior_max_speed(&report, L);
        rows.push(BondRow {
            cap,
            drag,
            interior_max_speed,
            divergence,
        });
    }

    print_utils::render(&rows);
    if print_utils::verify(&rows) {
        print_utils::summary();
    } else {
        std::process::exit(1);
    }
}

/// Print a stage-failure context and its error on stderr, then exit the process non-zero.
fn fail(context: &str, error: impl core::fmt::Debug) -> ! {
    eprintln!("{context} failed: {error:?}");
    std::process::exit(1);
}
