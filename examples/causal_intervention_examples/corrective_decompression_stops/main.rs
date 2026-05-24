/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Decompression Stops as a Corrective `intervene` Loop
//!
//! A retrofit of the dive-computer example into the corrective
//! intervention pattern. A diver starts saturated at 30 m and ascends
//! toward the surface. A single mid-range tissue compartment tracks
//! N2 partial pressure across the ascent. The Bühlmann supersaturation
//! ratio is the monitored quantity: when tissue N2 climbs too high
//! relative to ambient pressure, decompression sickness becomes likely.
//!
//! The value channel carries the *ascent command* for each tick. By
//! default it is the planned ascent rate. When the monitor sees the
//! ratio cross the safety threshold, the next ascent command is
//! intervened to 0.0 metres. The simulation continues from a held
//! depth; the tissue offloads; the ratio drops; ascent resumes.
//!
//! Two trajectories run side by side.
//!
//! * Open loop. A continuous-ascent profile. Tissue cannot offload
//!   fast enough and the ratio crosses the DCS threshold partway up.
//! * Closed loop. The monitor inserts decompression stops whenever
//!   needed. The diver surfaces with the ratio inside the safe
//!   envelope and the `EffectLog` records every stop event.
//!
//! Same chain, same physics, same starting tissue saturation. The only
//! difference is whether the corrective `.intervene(0.0)` fires.

mod model;
pub mod model_types;
mod model_utils;

use crate::model_types::{DiveProcess, FloatType, N_TICKS};
use causal_intervention_examples::print_utils;
use deep_causality_core::Intervenable;

fn main() {
    println!("=== Decompression Stops as a Corrective `intervene` Loop ===\n");

    let open = run_open_loop();
    let closed = run_closed_loop();

    model_utils::print_section("Open loop (continuous ascent, no monitor)", &open);
    model_utils::print_section("Closed loop (monitor + corrective stops)", &closed);

    println!("=== Summary ===");
    model_utils::summary_line("Open loop  ", &open);
    model_utils::summary_line("Closed loop", &closed);

    println!(
        "\nThe open-loop ascent crosses the DCS ratio threshold partway\n\
         up. The closed-loop run uses the same physics, but each\n\
         supersaturation alarm triggers `.intervene(0.0)` on the ascent\n\
         command. The diver surfaces with the tissue ratio inside the\n\
         safe envelope."
    );

    println!("\n--- Closed-loop EffectLog (per-tick reading + every stop) ---");
    print_utils::print_effect_log(&closed.logs);
}

fn run_open_loop() -> DiveProcess<FloatType> {
    let mut process = model::initial_process();
    for _ in 0..N_TICKS {
        process = process.bind(model::simulate_step);
    }
    process
}

fn run_closed_loop() -> DiveProcess<FloatType> {
    let mut process = model::initial_process();
    for _ in 0..N_TICKS {
        process = process.bind(model::simulate_step);

        let cfg = process.context.clone().expect("DiveConfig present");
        if process.state.last_ratio > cfg.safety_ratio_threshold && process.state.depth_m > 0.0 {
            process.state.stop_count += 1;
            process = process.intervene(0.0);
        }
    }
    process
}
