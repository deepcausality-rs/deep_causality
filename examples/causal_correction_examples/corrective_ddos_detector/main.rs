/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # DDoS Detection as a Corrective `intervene` Loop
//!
//! Network anomaly detection as a closed-loop control action. A virtual
//! interface carries traffic from a generator; a fixed-size, array-backed
//! sliding window holds the recent throughput as a rolling baseline. Each
//! tick the new measured throughput is scored against that baseline as a
//! z-score (how many standard deviations above the mean). When the score
//! crosses the parametric `sigma_threshold` for `trigger_slots` consecutive
//! seconds, a volumetric denial-of-service attack is declared and the loop
//! `.intervene(THROTTLE_ON)`s the value channel. The virtual NIC's regulator
//! reads that command and rate-limits the interface to the throttle ceiling,
//! mitigating the flood from the very next tick.
//!
//! The value channel carries the NIC's regulator command (`THROTTLE_OFF` /
//! `THROTTLE_ON`); state threads the sliding window plus the detection
//! accounting; context holds the read-only detector configuration. The whole
//! loop is one fluent `iterate_n` over the causal monad.
//!
//! ## Detecting a *sustained* surge
//!
//! The detector scores each incoming sample against the baseline and admits
//! only non-anomalous samples to the window. That withholding is deliberate.
//! The naive alternative — push every sample, then test whether the window
//! max exceeds the window's own mean + 3σ — self-masks: as flood samples
//! accumulate they inflate the mean and σ, so the z-score of the max collapses
//! back under the threshold within a few ticks and "3σ for 5 consecutive
//! seconds" never holds. Keeping the baseline clean lets the flood read as
//! anomalous for its full duration.
//!
//! Important! - Disabling the mitigation requires manual override by the operator
//! because there is no automatic disabling implemented. This is a deliberate decision.
//! If you have a use case where disabling can be automated safely, you need to implement
//! this based on the shown design pattern e.g. check that a mitigation is on, and simultaneously,
//! that twice the detection duration traffic was back to normal and only then it's safe to disable the mitigation.
//! Importantly, the detector must judge abatement on the raw inbound load, which a real scrubbing appliance still
//! sees while it rate-limits what it forwards. Also, you would add an extended observation phase on the raw inbound
//! before deciding whether disabling the mitigation is safe.
//!

mod model;
pub mod model_types;
mod model_utils;

use crate::model::{analyze_tick, initial_process};
use crate::model_types::{DetectorProcess, N_TICKS, THROTTLE_OFF, THROTTLE_ON, ThrottleState};
use causal_correction_examples::print_utils;
use deep_causality_core::CausalFlow;

fn main() {
    println!("=== DDoS Detection as a Corrective `intervene` Loop ===\n");

    // Closed loop: each tick advances the sliding-window analysis, then
    // `branch_with` engages the throttle the moment the surge has persisted
    // for `trigger_slots` consecutive anomalous seconds. The `throttle == OFF`
    // guard makes the mitigation fire exactly once.
    let result: DetectorProcess<ThrottleState> = CausalFlow::from(initial_process())
        .iterate_n(N_TICKS as usize, |tick| {
            tick.bind(analyze_tick).branch_with(
                |throttle, state, ctx| {
                    let cfg = ctx.expect("DetectorConfig present");
                    state.consecutive_anomalies >= cfg.trigger_slots && *throttle == THROTTLE_OFF
                },
                |anomaly| {
                    anomaly
                        .update_state(|mut state, _throttle| {
                            state.mitigation_count += 1;
                            if state.mitigated_at.is_none() {
                                state.mitigated_at = Some(state.tick);
                            }
                            state
                        })
                        .intervene(THROTTLE_ON)
                },
                // No detection. Business as usual.
                |normal| normal,
            )
        })
        .into_process();

    // Verbose details. Comment out to trim the output.
    model_utils::print_section("Closed loop", &result);

    println!("=== Summary ===");
    model_utils::summary_line("Closed loop", &result);
    print_utils::print_explenation();

    println!("\n--- EffectLog (per-tick analysis + mitigation event) ---");
    print_utils::print_effect_log(&result.logs);
}
