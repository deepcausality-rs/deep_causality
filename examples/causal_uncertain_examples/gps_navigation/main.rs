/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # GPS Navigation with Uncertainty as a `CausalFlow` Chain
//!
//! Reworks the original straight-line `Uncertain<f64>` example into a four-stage
//! stateless chain over `CausalFlow`. Each stage receives the previous stage's
//! `Uncertain<f64>` directly and returns the next uncertain quantity; the flow
//! supplies the plumbing, so no stage touches `EffectValue` or `PropagatingEffect`.
//!
//! Pipeline:
//!
//! 1. `distance_stage`  — propagate position noise into a distance estimate
//! 2. `time_stage`      — propagate distance and speed noise into a travel-time estimate
//! 3. `route_stage`     — compare against an alternative route under uncertainty
//! 4. `fuel_stage`      — propagate distance and efficiency noise into a fuel estimate
//!
//! The `Uncertain<f64>` API (sampling, comparisons, conditional, probability
//! exceedance) is unchanged; `CausalFlow::map` sequences the stages and the
//! terminal `run` reports completion or the rare short-circuit.

mod model;

use deep_causality_core::CausalFlow;
use deep_causality_uncertain::Uncertain;
use model::{Position, distance_stage, fuel_stage, route_stage, time_stage};

fn main() {
    println!("GPS Navigation with Uncertainty Analysis (CausalFlow chain)");
    println!("=====================================================================\n");

    let start = Position {
        lat: Uncertain::normal(37.7749, 0.0001), // San Francisco, ~10 m GPS noise
        lon: Uncertain::normal(-122.4194, 0.0001),
    };
    let destination = Position {
        lat: Uncertain::<f64>::point(37.7849),   // ~1 mile north
        lon: Uncertain::<f64>::point(-122.4094), // ~1 mile east
    };

    CausalFlow::value(start)
        .map(move |pos| distance_stage(pos, destination.clone()))
        .map(time_stage)
        .map(route_stage)
        .map(fuel_stage)
        .run(
            |_| println!("\n✅ Pipeline complete."),
            |err| println!("\n⚠️  Pipeline short-circuited.\n   error: {err:?}"),
        );
}
