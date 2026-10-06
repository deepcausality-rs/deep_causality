/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # GPS Navigation with Uncertainty as a `CausalFlow` Chain
//!
//! A four-stage chain over `CausalFlow` with no state and the route as its
//! context. The start position is the chain's initial value; the destination,
//! the parameters of the speed, traffic, fuel-efficiency and fuel-on-hand
//! distributions, the distances, the lateness threshold and the fuel limits
//! are `Data<f64>` contextoids of the route context. Each stage receives the
//! previous stage's `Uncertain<f64>` directly, reads its route quantities from
//! the context, and returns the next uncertain quantity; the flow supplies the
//! plumbing, so no stage touches `CausalEffect` or `PropagatingEffect`.
//!
//! Pipeline:
//!
//! 1. `distance_stage`  — propagate position noise into a distance estimate
//! 2. `time_stage`      — propagate distance and speed noise into a travel-time estimate
//! 3. `route_stage`     — compare against an alternative route under uncertainty
//! 4. `fuel_stage`      — propagate distance and efficiency noise into a fuel estimate
//!
//! The `Uncertain<f64>` API (sampling, comparisons, conditional, probability
//! exceedance) does the numerical work; `CausalFlow::try_step_with` sequences the
//! stages and hands each one the route context, and the terminal `run` reports
//! completion or the rare short-circuit.

mod model;

use deep_causality_core::CausalFlow;
use deep_causality_uncertain::Uncertain;
use model::{Position, distance_stage, fuel_stage, route_context, route_stage, time_stage};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("GPS Navigation with Uncertainty Analysis (CausalFlow chain)");
    println!("=====================================================================\n");

    let start = Position {
        lat: Uncertain::normal(37.7749, 0.0001), // San Francisco, ~10 m GPS noise
        lon: Uncertain::normal(-122.4194, 0.0001),
    };

    CausalFlow::value(start)
        .context(route_context()?)
        .try_step_with(distance_stage)
        .try_step_with(time_stage)
        .try_step_with(route_stage)
        .try_step_with(fuel_stage)
        .run(
            |_| println!("\n✅ Pipeline complete."),
            |err| println!("\n⚠️  Pipeline short-circuited.\n   error: {err:?}"),
        );
    Ok(())
}
