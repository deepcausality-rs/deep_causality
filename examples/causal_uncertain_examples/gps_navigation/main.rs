/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # GPS Navigation with Uncertainty as a `PropagatingEffect` Chain
//!
//! Reworks the original straight-line `Uncertain<f64>` example into a four-stage
//! stateless monadic chain over `PropagatingEffect`. Each stage receives the
//! previous stage's `Uncertain<f64>` carrier from `EffectValue::Value`, computes
//! the next uncertain quantity, and re-lifts it with `PropagatingEffect::pure`.
//!
//! Pipeline:
//!
//! 1. `distance_stage`  — propagate position noise into a distance estimate
//! 2. `time_stage`      — propagate distance and speed noise into a travel-time estimate
//! 3. `route_stage`     — compare against an alternative route under uncertainty
//! 4. `fuel_stage`      — propagate distance and efficiency noise into a fuel estimate
//!
//! The `Uncertain<f64>` API (sampling, comparisons, conditional, probability
//! exceedance) is unchanged; the monad supplies the chain's plumbing and the
//! short-circuit on the rare error case.

mod model;

use deep_causality_core::{EffectValue, PropagatingEffect};
use deep_causality_uncertain::Uncertain;
use model::{Position, distance_stage, fuel_stage, route_stage, time_stage};

fn main() {
    println!("🚗 GPS Navigation with Uncertainty Analysis (PropagatingEffect chain)");
    println!("=====================================================================\n");

    let start = Position {
        lat: Uncertain::normal(37.7749, 0.0001), // San Francisco, ~10 m GPS noise
        lon: Uncertain::normal(-122.4194, 0.0001),
    };
    let destination = Position {
        lat: Uncertain::<f64>::point(37.7849),   // ~1 mile north
        lon: Uncertain::<f64>::point(-122.4094), // ~1 mile east
    };

    let pipeline = PropagatingEffect::pure(start)
        .bind(move |value, _, _| distance_stage(value, destination.clone()))
        .bind(|value, _, _| time_stage(value))
        .bind(|value, _, _| route_stage(value))
        .bind(|value, _, _| fuel_stage(value));

    match pipeline.value {
        EffectValue::Value(_) => println!("\n✅ Pipeline complete."),
        EffectValue::None => println!("\n⚠️  Pipeline short-circuited (EffectValue::None)."),
        _ => println!("\n⚠️  Pipeline returned an unexpected EffectValue variant."),
    }

    if let Some(err) = pipeline.error {
        println!("   error: {err:?}");
    }
}
