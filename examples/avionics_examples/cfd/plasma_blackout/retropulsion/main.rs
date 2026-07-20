/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Plasma-retropulsion descent — blackout exit through ignition to touchdown
//!
//! The third plasma-blackout example, and the one that closes the family's loop: the corridor flies
//! a descent, the weather example generates the dispersion table, and this example **consumes that
//! table in flight** and carries the descent to the ground under a retro burn.
//!
//! Five acts:
//!
//! 1. **PLAN** — the measured day's temperature departure interpolates `weather_table.csv`; the
//!    interpolated row sizes the ignition margin and the propellant reserve.
//! 2. **CORRIDOR** — the existing descent, inherited with the burn stack composed and the throttle
//!    at zero.
//! 3. **COAST + BURN** — one march call, so ignition is a published-command event inside one world
//!    rather than a stack swap. Mid-burn the marched, plume-coupled state is **forked** and a small
//!    throttle roster continues from it.
//! 4. **TERMINAL** — cutoff, a subsonic re-seed under its own gamma, and the descent to touchdown.
//!
//! **What the counterfactual measures, and what it does not.** The M1 de-risk measurement came back
//! **AMBER** on imprint fidelity: a compressible forcing region does not reproduce the
//! Jarvinen-Adams drag collapse at this fidelity. So the in-flight drag authority is the **cited A0
//! correlation**, not a decrement contracted from the field. What M1 measured *green* is the
//! state-fork machinery itself — an O(1) copy-on-write fork whose branches spread with the
//! intervention — so the fork here is a genuine fork of the marched state, carrying the flow-realism
//! and fork-economics witnesses that a parameter fork cannot express at all.
//!
//! The DSL never exits or prints: `main` maps the merged `Verdict` to an exit code (0 all gates
//! pass, 1 gate regression, 2 setup failure).
//!
//! ```bash
//! cargo run --release -p avionics_examples --example plasma_blackout_retropulsion
//! ```

mod constants;
mod model;
mod utils_print;

use avionics_examples::shared::constants::DT_FLIGHT;
use avionics_examples::shared::{utils, world};
use deep_causality_cfd::{
    CfdFlow, CompressibleMarchConfig, IGNITION_COMMIT_AIDED_FIELD, IGNITION_COMMIT_MACH_FIELD,
    IGNITION_COMMIT_Q_FIELD, IGNITION_COMMIT_SIGMA_FIELD, IGNITION_COMMIT_STEP_FIELD,
    IGNITION_LATCH_FIELD, MarchStop, PhysicsError, StudyError, StudyView, Verdict,
};
use std::cell::RefCell;
use std::process::ExitCode;
use std::time::Instant;

/// The scalar precision the whole example runs at.
///
/// Deliberately per-example: switching this one alias to `deep_causality_num::Float106` runs the
/// entire descent in double-double without touching a solver, and each example makes that choice
/// for itself.
pub type FloatType = f64;

fn main() -> ExitCode {
    let leg_err = |stage: &'static str| move |e: PhysicsError| StudyError::in_stage(stage, e);

    let outcome: Result<Verdict, StudyError> = (|| {
        let clock = Instant::now();
        utils_print::print_intro();

        // ── Act 0: PLAN. The measured day interpolates the recorded dispersion table. ──────────
        let table = model::load_dispersion_table().map_err(leg_err("setup: weather table"))?;
        let informed = model::day_belief(&table, utils::ft(constants::MEASURED_D_TEMP));
        let uninformed = model::standard_day_belief(&table);
        utils_print::print_plan(&informed, &uninformed);

        // The belief counterfactual's witness: how much ignition margin the two guidances demand.
        let belief_separation_m = (informed.margin_m - uninformed.margin_m).abs();

        // ── Act 1: CORRIDOR. The inherited descent, burn stack composed, throttle at zero. ─────
        let corridor_world =
            model::trunk_world(constants::ONSET_STEPS).map_err(leg_err("setup: corridor world"))?;
        let onset = CfdFlow::march(&corridor_world)
            .couple(world::powered_descent_coupling(1.0, 0, informed.margin_m))
            .trigger(utils::trigger())
            .kappa(utils::ft(0.0))
            .from_field(world::powered_initial_field())
            .until(|field, _| field.regime().map(|r| r.gnss_denied).unwrap_or(false))
            .map_err(leg_err("act 1: corridor to blackout onset"))?;
        utils_print::print_act("CORRIDOR — descent to blackout onset", &onset);

        // ── Acts 2+3: COAST, COMMIT, BURN — one march call. ────────────────────────────────────
        //
        // A coupling stack is fixed per march call and `MarchState` carries the coupled field but
        // not the marched fluid tensor, so a leg boundary at ignition would re-seed the flow and the
        // fork below would fork a state from which the plume had already been discarded.
        let burn_world =
            model::burn_trunk_world(constants::BURN_STEPS).map_err(leg_err("setup: burn world"))?;
        let burn = CfdFlow::march(&burn_world)
            .couple(world::powered_descent_coupling(1.0, 0, informed.margin_m))
            .trigger(utils::trigger())
            .kappa(utils::ft(0.0))
            .from(onset.state())
            .until(|field, _| {
                // Pause inside the burn: the engine is lit and the plume is on the layer.
                field
                    .scalar("ignited")
                    .and_then(|s| s.first().copied())
                    .is_some_and(|v| v > 0.0)
            })
            .map_err(leg_err("act 2-3: coast, commit, burn"))?;
        utils_print::print_act("COAST + BURN — ignition corridor committed", &burn);

        // The trunk's state at the fork. Each branch is scored on what it changed from here, so the
        // descent it inherited is not counted as its own doing, and the frozen-drag foil knows the
        // fraction to hold the closure at.
        let fork = model::ForkState {
            fraction: model::scalar0(burn.field(), "preserved_drag_fraction"),
            propellant: model::scalar0(burn.field(), "propellant"),
            dv_actual: model::scalar0(burn.field(), "dv_actual"),
            dv_frozen: model::scalar0(burn.field(), "dv_frozen"),
        };

        // ── The centerpiece: fork the marched, plume-coupled state. ────────────────────────────
        let committed_capture: RefCell<Option<model::BranchRow>> = RefCell::new(None);
        // The committed branch's measured rank, captured where the reports are still in hand.
        let bond_capture: RefCell<Option<usize>> = RefCell::new(None);
        let branches = CfdFlow::study("mid-burn throttle roster")
            .cases(model::throttle_roster())
            .fork(&burn)
            .branch(|case| model::branch_world(case, fork.fraction))
            .continue_for(constants::BRANCH_STEPS)
            .reduce(move |run| model::score_branch(run, fork))
            .inspect(|rows| {
                utils_print::print_branches(rows);
                // Commit the branch that shed the most velocity over the continuation — a trajectory
                // outcome, not an instantaneous force reading. `total_cmp` rather than an unwrapping
                // `partial_cmp`: the witnesses are checked finite where they are read, and a total
                // order removes the panic entirely rather than relying on that check holding.
                let committed = rows
                    .iter()
                    .max_by(|a, b| a.dv_actual.total_cmp(&b.dv_actual))
                    .cloned();
                *bond_capture.borrow_mut() =
                    committed.as_ref().map(|c| c.peak_bond).unwrap_or(None);
                *committed_capture.borrow_mut() = committed;
            })
            .record(model::branch_table_path())
            .gates(model::branch_gates())
            .verdict()?;

        let committed_bond = bond_capture.into_inner();
        // Read for its side condition: a roster that captured no branch is a study failure.
        let _committed = committed_capture.into_inner().ok_or_else(|| {
            leg_err("study: committed branch")(PhysicsError::CalculationError(
                "no branch captured".into(),
            ))
        })?;

        // ── Act 3b: the supersonic BURN continues from the fork point under the SRP envelope. ──
        //
        // The fork pauses at *ignition*, which is where the counterfactual must be taken — so the
        // burn itself still lies ahead. It runs under the supersonic-retropulsion axes (the C_T
        // stability cap, the jet-penetration throttle floor) until the vehicle drops out of that
        // regime, which is exactly where those axes stop describing the physics.
        let burn_out_world = model::burn_trunk_world(constants::BURN_OUT_STEPS)
            .map_err(leg_err("setup: burn-out world"))?;
        let burn_out = CfdFlow::march(&burn_out_world)
            .couple(world::powered_descent_coupling(1.0, 0, informed.margin_m))
            .trigger(utils::trigger())
            .kappa(utils::ft(0.0))
            .from(burn.state())
            .until(|field, _| {
                field
                    .scalar("flight_mach")
                    .and_then(|s| s.first().copied())
                    .is_some_and(|m| m < constants::SUBSONIC_HANDOVER_MACH)
            })
            .map_err(leg_err("act 3b: supersonic burn"))?;
        utils_print::print_act(
            "BURN — supersonic retropulsion under the SRP envelope",
            &burn_out,
        );

        // ── Act 4: TERMINAL. Cutoff at a leg boundary, subsonic re-seed, descent to touchdown. ──
        let terminal_world: CompressibleMarchConfig<FloatType> =
            model::terminal_world(constants::TERMINAL_STEPS)
                .map_err(leg_err("setup: terminal world"))?;
        let terminal = CfdFlow::march(&terminal_world)
            .march_with(MarchStop::Fixed(constants::TERMINAL_STEPS))
            .couple(world::powered_descent_coupling_with(
                1.0,
                0,
                informed.margin_m,
                true,
            ))
            .trigger(utils::trigger())
            .kappa(utils::ft(0.0))
            .from(burn_out.state())
            .until(|field, _| field.regime().map(|r| r.touchdown).unwrap_or(false))
            .map_err(leg_err("act 4: terminal descent"))?;
        utils_print::print_act("TERMINAL — cutoff, subsonic re-seed, touchdown", &terminal);

        // ── The witnesses the trajectory gates read, every one a typed accessor. ──────────────
        //
        // A captured step error is carried into the integrity gate rather than returned here. An
        // early return exits as a *setup* failure and the gates never run, so three of the four legs
        // used to be outside the one gate whose job is to notice them.
        let leg_errors: Vec<(String, String)> = [
            ("act 1: corridor", onset.error()),
            ("act 2-3: coast, commit, burn", burn.error()),
            ("act 3b: supersonic burn", burn_out.error()),
            ("act 4: terminal descent", terminal.error()),
        ]
        .into_iter()
        .filter_map(|(name, e)| e.map(|e| (name.to_string(), format!("{e}"))))
        .collect();

        let f = terminal.field();
        let commit_step = model::scalar0(f, IGNITION_COMMIT_STEP_FIELD);
        let legs = [model::LegSet {
            steps: terminal.step(),
            leg_errors,
            committed: model::scalar0(f, IGNITION_LATCH_FIELD) > 0.0,
            commit_step,
            commit_mach: model::scalar0(f, IGNITION_COMMIT_MACH_FIELD),
            commit_q: model::scalar0(f, IGNITION_COMMIT_Q_FIELD),
            commit_aided: model::scalar0(f, IGNITION_COMMIT_AIDED_FIELD) > 0.0,
            commit_sigma_m: model::scalar0(f, IGNITION_COMMIT_SIGMA_FIELD),
            commit_margin_m: informed.margin_m,
            altitude_km: model::scalar0(f, "flight_altitude") / 1000.0,
            descent_rate: model::scalar0(f, "descent_rate"),
            propellant: model::scalar0(f, "propellant"),
            touchdown: f.regime().map(|r| r.touchdown).unwrap_or(false),
            rebuilds: onset.rebuilds()
                + burn.rebuilds()
                + burn_out.rebuilds()
                + terminal.rebuilds(),
            re_seeds: terminal.re_seeds(),
            regime_transitions: terminal.regime_transitions(),
            // The onset is recorded as a step index; the table records seconds, so the compressed
            // flight step converts it.
            onset_s: model::scalar0(f, "wx_onset_step") * utils::ft(DT_FLIGHT),
            dwell_s: model::scalar0(f, "wx_dwell_s"),
            drift_denied_max_m: model::scalar0(f, "wx_drift_denied_max"),
            predicted_onset_s: informed.onset_s,
            predicted_dwell_s: informed.dwell_s,
            elapsed_s: utils::ft(clock.elapsed().as_secs_f64()),
            belief_separation_m,
            belief_clamped: informed.clamped,
            peak_bond: committed_bond,
        }];
        utils_print::print_provenance(terminal.field().log());

        Ok(branches.merge(model::leg_gates().check(&StudyView::of(&legs))))
    })();

    match outcome {
        Ok(verdict) => {
            println!("\n{verdict}");
            if verdict.passed() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("plasma-retropulsion descent failed: {e}");
            ExitCode::from(2)
        }
    }
}
