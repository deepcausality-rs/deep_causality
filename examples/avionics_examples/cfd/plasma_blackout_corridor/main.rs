/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The plasma-blackout corridor: one continuous descent through flow, plasma, navigation, and control
//!
//! A reentry vehicle punches into the atmosphere at Mach 25. The shock layer ionizes, and past a
//! critical electron density the plasma sheath cuts every GNSS link; RAM-C II measured exactly
//! this blackout. Through the dark the vehicle dead-reckons on its INS while a bounded-correction
//! gate keeps the bank command inside the certified envelope. When the sheath clears, one fix
//! collapses the accumulated drift. This example flies that corridor as **one continuous
//! descent** on the compressible carrier, wiring the corridor §4 chain [1] through [7]:
//!
//! * **[1] flow**: the 2-D compressible marcher on tensor trains. The truth vehicle's altitude
//!   and Mach number select the freestream from the atmosphere schedule each step; the exact
//!   Rankine-Hugoniot jump is enforced on the inflow strip (the shock-fitted boundary), and the
//!   layer behind it is evolved. `T_tr`, `n_tot`, and the pressure are marched projections; no
//!   station constants, no reconstruction stages.
//! * **[2] regime**: `RegimeClassify` turns the freestream Knudsen number into the governing
//!   model and the evolved electron density into the GNSS-denial flag. Blackout onset and exit
//!   are *events the run finds*, not station switches.
//! * **[3] reacting plasma**: the Park two-temperature stages on the evolved state — the
//!   Millikan-White clock on the evolved per-cell pressure, ionization at the controller `Tₐ` on
//!   the evolved per-cell density, sheath renewal at one residence time.
//! * **[4] navigation**: `TrajectoryNav` runs the KS-regularized orbit predict with the ④
//!   aero-force channel as the kick; 17-state ESKF corrections are gated by the *real* blackout
//!   flag.
//! * **[5] counterfactuals**: at the flow-resolved onset the march pauses (`run_until`) and forks
//!   in O(1), copy-on-write. Each candidate bank command continues in its own alternated world
//!   (`alternate_context`, the verbatim core vocabulary) and is scored by its **trajectory-derived
//!   miss** to a shared aim point; the t²-law proxy is printed as a cross-check.
//! * **[6] bounded correction**: `CyberneticCorrect` clamps the commanded bank into the
//!   `SafetyEnvelope`, and `BankSteeredLift` *flies* the clamped command — steering truth and
//!   navigation through the point-mass 3-DOF lift.
//! * **[7] provenance**: the `EffectLog` rides the coupled field across the whole descent.
//!   Regime transitions, nav-mode changes, carrier rebuilds, bounded corrections, and alternation
//!   markers are all in one auditable record.
//!
//! Every simplification is labeled in `constants.rs`. The example self-verifies and exits
//! nonzero on regression (`exit(1)`) or setup failure (`exit(2)`).
//!
//! ```bash
//! cargo run --release -p avionics_examples --example plasma_blackout_corridor
//! ```
mod constants;
mod model;
mod utils_print;

use avionics_examples::blackout::{support, world};
use deep_causality_cfd::{CfdFlow, MarchStop};
use deep_causality_core::AlternatableContext;
use std::process::exit;
use std::time::Instant;

/// The working precision of the whole corridor (flow, plasma, navigation, control). Switch this
/// alias between `f32`, `f64,` and `Float106` (106-bit double-double) and every
/// derived number is computed in this type, so the alias is the only line that changes.
/// Note, Float106 increases compute time tenfold for no tangible gain in this case because
/// model fidelity is the limiting factor.
/// f32, however, underflows in the Ionization kernel causing a division by zero error.
/// The alias itself is shared with the weather-dispersion example through
/// `avionics_examples::blackout::FloatType`.
pub type FloatType = avionics_examples::blackout::FloatType;

fn main() {
    let clock = Instant::now();
    utils_print::print_intro();

    // ── One descent world per commanded bank; the nominal descent is ballistic (zero bank).
    let nominal =
        model::descent_world("nominal_descent", 0.0).unwrap_or_else(|e| support::stop(&e));
    let bank_worlds = model::bank_worlds().unwrap_or_else(|e| support::stop(&e));

    // ── Leg 1: descend until the *flow-resolved* blackout onset.
    // The predicate is the classifier's denial flag: it fires when the evolved sheath's electron
    // density crosses the GPS L1 cutoff — an event the run finds, not a station switch.
    let onset = CfdFlow::compressible_march(&nominal)
        .run_until(
            world::corridor_coupling(1.0),
            world::initial_field(),
            support::trigger(),
            support::ft(0.0),
            |field, _| field.regime().map(|r| r.gnss_denied).unwrap_or(false),
        )
        .unwrap_or_else(|e| support::stop(&e));
    let leg1 = model::snapshot("descent to blackout onset", &onset);
    utils_print::print_leg(&leg1);

    // ── [5] The counterfactual study: fork the onset once per candidate bank command.
    // Each fork is O(1) through shared Arcs, and each branch resumes the *same* onset state in
    // its own alternated world (`!!ContextAlternation!!` in its log); the worlds differ only in
    // the bank command they publish, so the branch trajectories diverge by steering alone. The
    // branches are data-independent, so `continue_branches` flies them concurrently on scoped
    // threads (the `parallel` feature); the reports come back in world order either way.
    let branch_configs: Vec<&_> = bank_worlds.iter().map(|(_, world)| world).collect();
    let reports = onset
        .continue_branches(&branch_configs, constants::BRANCH_STEPS)
        .unwrap_or_else(|e| support::stop(&e));
    // The aim point: the ballistic terminal state offset cross-range, shared by every branch.
    let aim = model::aim_point(model::terminal_position(&reports[0]));
    let branches: Vec<model::BranchScore> = bank_worlds
        .iter()
        .zip(&reports)
        .map(|((deg, _), report)| model::score_branch(*deg, report, aim))
        .collect();
    let committed = model::pick_committed(&branches);
    utils_print::print_branches(&branches, committed);

    // ── Leg 2: fly the committed world through the peak passage (the 61 km RAM-C II station).
    // The world swap is a loud pre-run context alternation; the carried field brings the
    // navigation state, the evolved projections, and the provenance log along. Pausing at the
    // 61 km passage is diagnostic only — the world and its schedule never change.
    let committed_world = &bank_worlds[committed].1;
    let peak = CfdFlow::compressible_march(&nominal)
        .alternate_context(committed_world)
        .run_until(
            world::corridor_coupling(1.0),
            model::carry_field(&onset),
            support::trigger(),
            support::ft(0.0),
            |field, _| {
                field
                    .scalar("flight_altitude")
                    .and_then(|a| a.first().copied())
                    .is_some_and(|a| a <= support::ft(61_000.0))
            },
        )
        .unwrap_or_else(|e| support::stop(&e));
    let leg2 = model::snapshot("peak passage 61 km (committed dwell)", &peak);
    utils_print::print_leg(&leg2);
    // ── Leg 3: continue until the *flow-resolved* exit — the link comes back when the vehicle
    // has decelerated enough that the renewed sheath no longer ionizes past the cutoff.
    let exit_pause = CfdFlow::compressible_march(&nominal)
        .alternate_context(committed_world)
        .run_until(
            world::corridor_coupling(1.0),
            model::carry_field(&peak),
            support::trigger(),
            support::ft(0.0),
            |field, _| field.regime().map(|r| !r.gnss_denied).unwrap_or(false),
        )
        .unwrap_or_else(|e| support::stop(&e));
    let leg3 = model::snapshot("flow-resolved exit", &exit_pause);
    utils_print::print_leg(&leg3);

    // ── Leg 4: reacquisition. A short fixed segment after the exit: the first folded fixes
    // collapse the dead-reckoning drift.
    let reacq = CfdFlow::compressible_march(&nominal)
        .alternate_context(committed_world)
        .march_with(MarchStop::Fixed(constants::REACQ_STEPS))
        .run_until(
            world::corridor_coupling(1.0),
            model::carry_field(&exit_pause),
            support::trigger(),
            support::ft(0.0),
            |_, _| false,
        )
        .unwrap_or_else(|e| support::stop(&e));
    let leg4 = model::snapshot("reacquisition", &reacq);
    utils_print::print_leg(&leg4);

    // ── [7] Provenance, then the coupled validation gates.
    utils_print::print_provenance(reacq.field().log());
    let compression = model::compression_witness(&branches[committed].report_final);
    let rendered_log = format!("{}", reacq.field().log());
    let rebuilds = model::rebuild_count(&rendered_log);
    let ok = utils_print::report(&utils_print::GateInputs {
        leg1: &leg1,
        leg2: &leg2,
        leg3: &leg3,
        leg4: &leg4,
        branches: &branches,
        committed,
        compression,
        rebuilds,
        elapsed_s: clock.elapsed().as_secs_f64(),
        regime_log: &rendered_log,
    });
    if !ok {
        exit(1);
    }
}
