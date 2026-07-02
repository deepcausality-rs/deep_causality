/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The plasma-blackout corridor: flow, plasma, navigation, and control in one coupling
//!
//! A reentry vehicle punches into the atmosphere at Mach 25. The shock layer ionizes, and past a
//! critical electron density the plasma sheath cuts every GNSS link; RAM-C II measured exactly
//! this blackout. Through the dark the vehicle dead-reckons on its INS while a bounded-correction
//! gate keeps the bank command inside the certified envelope. When the sheath clears, one fix
//! collapses the accumulated drift. This example marches that corridor as a single composed
//! coupling in the `CfdFlow` DSL, wiring the corridor §4 chain [1] through [7]:
//!
//! * **[1] flow**: the QTT (tensor-train) incompressible carrier over an immersed blunt forebody.
//!   The entire march runs on compressed, bond-capped trains, never on the dense grid.
//! * **[2] regime**: `RegimeClassify` turns the Knudsen number into the governing model and the
//!   electron density into the GNSS-denial flag. Every regime change lands in the provenance log.
//! * **[3] reacting plasma**: the Tier-A Park-2T LER stages. Rankine-Hugoniot post-shock
//!   temperature, lagging ionization, electron density, pressure closure.
//! * **[4] navigation**: `TrajectoryNav` runs the KS-regularized orbit predict with the ④
//!   aero-force channel as the kick; 17-state ESKF corrections are gated by the *real* blackout
//!   flag.
//! * **[5] counterfactuals**: at blackout onset the march pauses (`run_until`) and forks in O(1),
//!   copy-on-write. Each candidate bank angle continues in its own alternated world
//!   (`alternate_context`, the verbatim core vocabulary) and is scored into a `BranchOutcome`.
//! * **[6] bounded correction**: `CyberneticCorrect` clamps the guidance bank command into the
//!   `SafetyEnvelope` through a cybernetic `control_step`; an unrecoverable breach
//!   short-circuits.
//! * **[7] provenance**: the `EffectLog` rides the coupled field across all three legs and
//!   surfaces on every report. Regime transitions, nav-mode changes, bounded corrections, and
//!   alternation markers are all in one auditable record.
//!
//! The corridor flies three legs over the RAM-C II trajectory: approach (thin air, chemistry
//! frozen, GNSS aided), peak heating (the Mach-25 station: ionization saturates, blackout, dead
//! reckoning, the branch study), and exit (decelerated, sheath cleared, reacquisition). Every
//! Tier-A simplification is labeled in `constants.rs`. The carried `CoupledField` threads the
//! navigation state, the reacting fraction, and the provenance log through every leg. The example
//! self-verifies and exits nonzero on regression.
//!
//! ```bash
//! cargo run --release -p avionics_examples --example plasma_blackout_corridor
//! ```

mod constants;
mod model;
mod utils_print;

use deep_causality_cfd::CfdFlow;
use deep_causality_core::AlternatableContext;
use std::process::exit;

/// Switch this alias to `f32` for low precision or `f64` for standard precision. The whole
/// corridor (flow, plasma, navigation, control) is generic over it.
pub type FloatType = f64;

fn main() {
    utils_print::print_intro();

    // ── Leg 1: approach (~90 km; chemistry frozen, GNSS aided) ────────────────────────────────
    // run_until with a never-firing predicate marches the full leg and hands back the *paused*
    // state: the carried CoupledField (nav engine, reacting fraction, provenance log) the next
    // leg resumes from. A plain run would only return a Report and lose the carried state.
    let approach = model::world(&constants::APPROACH).unwrap_or_else(|e| stop(&e));
    let pause1 = CfdFlow::qtt_march(&approach)
        .run_until(
            model::corridor_coupling(&constants::APPROACH),
            model::initial_field(&constants::APPROACH),
            model::trigger(),
            model::ft(constants::SCALAR_KAPPA),
            |_, _| false,
        )
        .unwrap_or_else(|e| stop(&e));
    let leg1 = model::snapshot("approach ~90 km", &pause1);
    utils_print::print_leg(&leg1);

    // ── Leg 2a: peak heating (61 km). March *until blackout onset*, then pause. ───────────────
    let peak = model::world(&constants::PEAK).unwrap_or_else(|e| stop(&e));
    let onset = CfdFlow::qtt_march(&peak)
        .run_until(
            model::corridor_coupling(&constants::PEAK),
            model::carry_field(&pause1, &constants::PEAK),
            model::trigger(),
            model::ft(constants::SCALAR_KAPPA),
            |field, _| field.regime().map(|r| r.gnss_denied).unwrap_or(false),
        )
        .unwrap_or_else(|e| stop(&e));
    let leg2a = model::snapshot("peak 61 km (blackout onset)", &onset);
    utils_print::print_leg(&leg2a);

    // ── [5] The branch study: fork the onset once per candidate bank angle ────────────────────
    // Each fork is O(1) through shared Arcs. Each branch resumes the *same* onset state in its
    // own alternated world (`!!ContextAlternation!!` in its log) and reports its continued
    // segment.
    let bank_worlds = model::bank_worlds().unwrap_or_else(|e| stop(&e));
    let mut branches = Vec::new();
    for (bank_deg, world) in &bank_worlds {
        let report = onset
            .fork()
            .alternate_context(world)
            .continue_march(constants::BRANCH_STEPS)
            .unwrap_or_else(|e| stop(&e));
        branches.push(model::score_branch(*bank_deg, &report));
    }
    let committed = model::pick_committed(&branches);
    utils_print::print_branches(&branches, committed);

    // ── Leg 2b: the committed dwell. Fly the chosen world through the blackout. ───────────────
    // The committed continuation is itself a loud pre-run context alternation: the nominal peak
    // world swapped for the winning bank world, resumed from the carried onset field.
    let committed_world = &bank_worlds[committed].1;
    let pause3 = CfdFlow::qtt_march(&peak)
        .alternate_context(committed_world)
        .run_until(
            model::corridor_coupling(&constants::PEAK),
            model::carry_field(&onset, &constants::PEAK),
            model::trigger(),
            model::ft(constants::SCALAR_KAPPA),
            |_, _| false,
        )
        .unwrap_or_else(|e| stop(&e));
    let leg2b = model::snapshot("peak 61 km (committed dwell)", &pause3);
    utils_print::print_leg(&leg2b);

    // ── Leg 3: exit (decelerated, sheath cleared). Reacquisition. ─────────────────────────────
    let exit_world = model::world(&constants::EXIT).unwrap_or_else(|e| stop(&e));
    let pause4 = CfdFlow::qtt_march(&exit_world)
        .run_until(
            model::corridor_coupling(&constants::EXIT),
            model::carry_field(&pause3, &constants::EXIT),
            model::trigger(),
            model::ft(constants::SCALAR_KAPPA),
            |_, _| false,
        )
        .unwrap_or_else(|e| stop(&e));
    let leg3 = model::snapshot("exit ~30 km", &pause4);
    utils_print::print_leg(&leg3);

    // ── [7] Provenance, then the coupled validation gates ─────────────────────────────────────
    utils_print::print_provenance(pause4.field().log());
    let compression = model::compression_witness(&branches[committed].report_final);
    let ok = utils_print::report(&leg1, &leg2a, &leg2b, &leg3, &branches, compression);
    if !ok {
        exit(1);
    }
}

fn stop(e: &deep_causality_cfd::PhysicsError) -> ! {
    eprintln!("corridor setup failed: {e}");
    exit(2)
}
