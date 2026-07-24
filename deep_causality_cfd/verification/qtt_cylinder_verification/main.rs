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
use deep_causality_cfd::PhysicsError;
use print_utils::BondRow;

/// The working precision for the whole computation (the single alias to change).
pub type FloatType = f64;

/// The grid: `2^L × 2^L`. Raised from `L = 5` (32²) to `L = 8` (256²) so the Brinkman penalization
/// layer `√(ην)` is resolved by the grid (`close-qtt-solver-envelope` item 10): at `L = 5` the layer
/// was 7× thinner than a cell and the reported `C_d` tracked the mask smoothing width, not the body.
/// `L = 8` resolves four of the five η-ladder points (`η ≥ dx²/ν = 0.012`); the smallest, `η = 0.008`,
/// is below that floor and is the under-resolved tail the ladder still carries (the full ladder is
/// resolved only at `L = 9`). See the config `DT`/`ETA` derivations and `main`'s bond-cap note.
const L: usize = 8;

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

    // Accuracy-vs-bond ladder: the same case at increasing round bond caps. Raised from [4, 8, 16, 24]
    // to [24, 48] for the finer L = 8 grid: a fixed low cap represents a coarser mask on a finer grid,
    // and the mask [0, 1] guard (item 14) rejects bond 4/8/16 at L = 8 as a >5%-out-of-range mask
    // (measured min χ = −0.15 at bond 4 on 256²). Bond 24 gives min χ = −1.4e-3, bond 48 gives −7e-7 —
    // both valid — so these two rungs still demonstrate rank convergence on a mask the guard accepts.
    let caps = [24usize, 48];
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

    // The parameter ladders. The bond ladder above says only that the *compression* has saturated;
    // it cannot say whether the reported drag has a limit. These two sweep the parameters that
    // actually move it: the penalization parameter (whose η → 0 limit is what would license calling
    // the penalization integral a drag) and the mask smoothing width (a purely numerical choice).
    let sweep_cap = *caps.last().expect("at least one bond cap");
    let eta_rows = sweep("eta", &config::ETA_LADDER, |eta| {
        config::build_config_with(L, sweep_cap, eta, config::SMOOTH_CELLS)
    });
    let smooth_rows = sweep("smoothing", &config::SMOOTH_LADDER, |s| {
        config::build_config_with(L, sweep_cap, config::ETA, s)
    });

    print_utils::render(&rows);
    print_utils::render_ladders(&eta_rows, &smooth_rows, sweep_cap);
    if print_utils::verify(&rows, &eta_rows, &smooth_rows) {
        print_utils::summary();
    } else {
        std::process::exit(1);
    }
}

/// Run one parameter ladder, returning `(parameter value, C_d, interior max|u|)` per rung.
fn sweep(
    name: &str,
    values: &[f64],
    build: impl Fn(f64) -> Result<deep_causality_cfd::QttMarchConfig<FloatType>, PhysicsError>,
) -> Vec<print_utils::LadderRow> {
    values
        .iter()
        .map(|&v| {
            let case = build(v).unwrap_or_else(|e| fail(&format!("QTT cylinder {name} config"), e));
            let report = CfdFlow::march(&case)
                .run()
                .unwrap_or_else(|e| fail(&format!("QTT cylinder {name} march"), e));
            let drag =
                Into::<f64>::into(*report.series("drag").expect("drag series").last().unwrap());
            print_utils::LadderRow {
                value: v,
                drag,
                interior_max_speed: print_utils::interior_max_speed(&report, L),
            }
        })
        .collect()
}

/// Print a stage-failure context and its error on stderr, then exit the process non-zero.
fn fail(context: &str, error: impl core::fmt::Debug) -> ! {
    eprintln!("{context} failed: {error:?}");
    std::process::exit(1);
}
