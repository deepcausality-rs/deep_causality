/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Lane-Keeping as a Corrective `intervene` Loop
//!
//! The corrective sense of intervention. A vehicle drifts laterally from
//! the lane centre under a deterministic wind-and-crown schedule. A
//! monitor watches the offset after every simulation tick. When the
//! drift exceeds the anomaly threshold, a P-controller produces the
//! corrected offset and feeds it back into the chain via
//! `.intervene(corrected)`. The next bind sees the corrected value, not
//! the drifted one. The loop continues from the corrected state.
//!
//! Two trajectories run side by side.
//!
//! * Open loop. No intervention. Drift accumulates monotonically and the
//!   vehicle leaves the lane within a handful of seconds.
//! * Closed loop. The monitor fires its correction whenever the offset
//!   crosses 0.30 m. The vehicle stays in the lane indefinitely. The
//!   `EffectLog` records every correction with timestamp and corrected
//!   value.
//!
//! Same chain, same simulation stage, same drift schedule. The only
//! difference is whether `.intervene` is wired into the loop. The
//! catastrophic outcome of the open-loop run is the failure the
//! corrective interventions prevent.

mod model;
pub mod model_types;
mod model_utils;

use crate::model_types::{FloatType, LaneProcess, N_TICKS, nominal_lane_config};
use causal_intervention_examples::print_utils;
use deep_causality_core::CausalFlow;

fn main() {
    println!("=== Lane-Keeping as a Corrective `intervene` Loop ===\n");

    let open = run_open_loop();
    let closed = run_closed_loop();

    model_utils::print_section("Open loop (no intervention)", &open);
    model_utils::print_section("Closed loop (monitor + corrective intervene)", &closed);

    println!("=== Summary ===");
    model_utils::summary_line("Open loop  ", &open);
    model_utils::summary_line("Closed loop", &closed);

    println!(
        "\nThe open-loop trajectory exceeds the lane half-width and is\n\
         marked off-road. The closed-loop trajectory uses the same drift\n\
         schedule, but each anomaly triggers an `.intervene(corrected)`\n\
         call that snaps most of the deviation away. The vehicle stays\n\
         inside the lane for the full run."
    );

    println!("\n--- Closed-loop EffectLog (every monitor tick and every intervention) ---");
    print_utils::print_effect_log(&closed.logs);
}

/// Open loop: each tick is just `simulate_step`, run `N_TICKS` times. No monitor, no correction.
fn run_open_loop() -> LaneProcess<FloatType> {
    CausalFlow::from(model::initial_process())
        .iterate_n(N_TICKS as usize, |tick| tick.bind(model::simulate_step))
        .into_process()
}

/// Closed loop: the same tick, but each one `branch`es on the monitor — when the offset crosses the
/// anomaly threshold the corrective arm records the override and `intervene`s the corrected offset.
/// The only difference from the open loop is the `branch`.
///
/// Both the trigger and the correction read the same `cfg`, so the threshold the monitor fires on
/// and the gain the correction applies can never desynchronize.
fn run_closed_loop() -> LaneProcess<FloatType> {
    let cfg = nominal_lane_config();
    CausalFlow::from(model::initial_process())
        .iterate_n(N_TICKS as usize, |tick| {
            tick.bind(model::simulate_step).branch(
                |offset| offset.abs() > cfg.anomaly_threshold,
                |hot| {
                    hot.update_state(|mut state, _offset| {
                        state.correction_count += 1;
                        state
                    })
                    .intervene_if(|_| true, |offset| model::correction(offset, &cfg))
                },
                |cold| cold,
            )
        })
        .into_process()
}
