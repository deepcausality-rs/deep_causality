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
//! descent** on the compressible carrier, expressed in the CfdFlow grammar at both levels:
//!
//! * **The trunk** is trajectory level — `CfdFlow::march(&nominal).couple(..).from(..).until(..)` —
//!   marching the coupled descent to the *flow-resolved* blackout onset, an event the run finds.
//! * **The bank-command study** is campaign level — `CfdFlow::study(..).fork(&onset).branch(..)
//!   .continue_for(..).reduce_all(..).refine(..)` — a two-round event-fork counterfactual sweep
//!   from the shared onset: a coarse 6-candidate round, then a 0.5-deg refinement bracketing the
//!   coarse winner. Both rounds fork the same paused onset O(1) copy-on-write, and score against
//!   the same aim point.
//! * **The diagnostic legs** are trajectory level again — the committed world flies the peak
//!   passage, the flow-resolved exit, and the reacquisition segment via `.alternate(committed)`.
//! * **The verdict** is the campaign gates (steering beats ballistic, the refinement improves the
//!   coarse winner, tensor compression holds) `merge`d with the trajectory leg gates (the blackout
//!   window, the RAM-C II anchor, real INS drift and reacquisition, bounded rebuilds).
//!
//! Every simplification is labeled in `constants.rs`. The DSL never exits or prints: `main` maps
//! the merged `Verdict` to an exit code (0 all gates pass, 1 gate regression, 2 setup failure).
//!
//! ```bash
//! cargo run --release -p avionics_examples --example plasma_blackout_corridor
//! ```
mod constants;
mod model;
mod utils_print;

use avionics_examples::shared::{utils, world};
use deep_causality_cfd::{CfdFlow, MarchStop, PhysicsError, StudyError, StudyView, Verdict};
use deep_causality_core::AlternatableContext;
use std::cell::RefCell;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

/// The working precision. Switch between `f64` and `deep_causality_num::Float106`
/// (106-bit double-double); the specification constants stay `f64` literals, which either type
/// represents exactly, and every derived number is computed in this type, so this alias is the
/// only line that changes.
pub type FloatType = f64;

/// The whole corridor is one program: the trunk leg to the flow-resolved onset, the two-round
/// bank-command study, the committed diagnostic legs, and the merged verdict — resolved inside one
/// fallible closure so the trajectory legs and the campaign share the `?` short-circuit, then
/// mapped to an exit code (0 all gates pass, 1 gate regression, 2 setup failure).
fn main() -> ExitCode {
    const COARSE_TITLE: &str =
        "Counterfactual bank commands (coarse sweep, forked from the shared flow-resolved onset)";
    const FINE_TITLE: &str =
        "Fine sweep (0.5-deg candidates around the coarse winner, same onset fork)";

    // Where the fine-round branch table is recorded (the campaign's `record` seam).
    let table_path = || {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("cfd/plasma_blackout/corridor/corridor_branches.csv")
    };
    // Lift a leg's solver error into the study error channel, so the whole corridor resolves to
    // one `Result<Verdict, StudyError>` — the trajectory legs and the campaign share an error type.
    let leg_err = |stage: &'static str| move |e: PhysicsError| StudyError::in_stage(stage, e);

    let outcome: Result<Verdict, StudyError> = (|| {
        let clock = Instant::now();
        utils_print::print_intro();

        let nominal = model::descent_world("nominal_descent", 0.0)
            .map_err(leg_err("setup: nominal world"))?;

        // ── The trunk: trajectory level, march to the flow-resolved blackout onset. The predicate
        // is the classifier's denial flag — it fires when the evolved sheath crosses the GPS L1
        // cutoff.
        let onset = CfdFlow::march(&nominal)
            .couple(world::corridor_coupling(1.0, 0))
            .trigger(utils::trigger())
            .kappa(utils::ft(0.0))
            .from_field(world::initial_field())
            .until(|field, _| field.regime().map(|r| r.gnss_denied).unwrap_or(false))
            .map_err(leg_err("leg: descent to blackout onset"))?;
        let leg1 = model::snapshot("descent to blackout onset", &onset);
        utils_print::print_leg(&leg1);

        // ── The bank-command study: campaign level, event-fork counterfactuals, two rounds forking
        // the same paused onset. The fine-round `inspect` captures the committed branch for the
        // legs below.
        let committed_capture: RefCell<Option<model::BranchRow>> = RefCell::new(None);
        let corridor = CfdFlow::study("bank-angle corridor")
            .cases(model::coarse_commands())
            .fork(&onset)
            .branch(model::bank_world)
            .continue_for(constants::BRANCH_STEPS)
            .reduce_all(model::score_branches) // aim point from the ballistic branch first
            .inspect(|rows| utils_print::print_branches(COARSE_TITLE, rows))
            .refine(&onset, model::fine_candidates) // 0.5-deg bracket around the coarse winner
            .branch(model::bank_world)
            .continue_for(constants::BRANCH_STEPS)
            .reduce_all(model::score_branches) // same aim point: the rounds stay comparable
            .inspect(|rows| {
                utils_print::print_branches(FINE_TITLE, rows);
                *committed_capture.borrow_mut() = Some(rows[model::pick_committed(rows)].clone());
            })
            .record(table_path())
            .gates(model::corridor_gates())
            .verdict()?;

        let committed_row = committed_capture.into_inner().ok_or_else(|| {
            leg_err("study: committed branch")(PhysicsError::CalculationError(
                "no committed branch captured (the study errored before the fine round)".into(),
            ))
        })?;
        let committed_world =
            model::committed_world(&committed_row).map_err(leg_err("committed world"))?;

        // ── The diagnostic legs: trajectory level, flying the committed world. The world swap is a
        // loud pre-run context alternation; the carried field brings the navigation state, the
        // evolved projections, and the provenance log along.
        let coupling = || world::corridor_coupling(1.0, 0);
        let peak = CfdFlow::march(&nominal)
            .alternate_context(&committed_world)
            .couple(coupling())
            .trigger(utils::trigger())
            .kappa(utils::ft(0.0))
            .from(onset.state())
            .until(|field, _| {
                field
                    .scalar("flight_altitude")
                    .and_then(|a| a.first().copied())
                    .is_some_and(|a| a <= utils::ft(61_000.0))
            })
            .map_err(leg_err("leg: peak passage"))?;
        let leg2 = model::snapshot("peak passage 61 km (committed dwell)", &peak);
        utils_print::print_leg(&leg2);

        let exit_pause = CfdFlow::march(&nominal)
            .alternate_context(&committed_world)
            .couple(coupling())
            .trigger(utils::trigger())
            .kappa(utils::ft(0.0))
            .from(peak.state())
            .until(|field, _| field.regime().map(|r| !r.gnss_denied).unwrap_or(false))
            .map_err(leg_err("leg: flow-resolved exit"))?;
        let leg3 = model::snapshot("flow-resolved exit", &exit_pause);
        utils_print::print_leg(&leg3);

        let reacq = CfdFlow::march(&nominal)
            .alternate_context(&committed_world)
            .march_with(MarchStop::Fixed(constants::REACQ_STEPS))
            .couple(coupling())
            .trigger(utils::trigger())
            .kappa(utils::ft(0.0))
            .from(exit_pause.state())
            .until(|_, _| false)
            .map_err(leg_err("leg: reacquisition"))?;
        let leg4 = model::snapshot("reacquisition", &reacq);
        utils_print::print_leg(&leg4);

        // ── Provenance, then the leg witnesses the trajectory gates read.
        utils_print::print_provenance(reacq.field().log());
        let rendered_log = format!("{}", reacq.field().log());
        let legs = [model::LegSet {
            leg1,
            leg2,
            leg3,
            leg4,
            rebuilds: model::rebuild_count(&rendered_log),
            elapsed_s: utils::ft(clock.elapsed().as_secs_f64()),
            regime_log: rendered_log,
        }];

        // ── One report: the campaign verdict merged with the trajectory leg verdict, applied to
        // the leg witnesses through the same GateSeq machinery (trajectory-level `check`).
        Ok(corridor.merge(model::leg_gates().check(&StudyView::of(&legs))))
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
            eprintln!("plasma-blackout corridor failed: {e}");
            ExitCode::from(2)
        }
    }
}
