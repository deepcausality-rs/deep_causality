/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage functions for the GPS-navigation `CausalFlow` chain.
//!
//! Each stage takes the previous stage's `Uncertain<f64>` directly, propagates uncertainty through
//! one physical transformation, prints the resulting statistics, and returns the next
//! `Uncertain<f64>`. `CausalFlow` supplies the chain's plumbing, so no stage touches `EffectValue`
//! or re-lifts with `PropagatingEffect::pure`; the stages read as plain transforms.

use deep_causality_uncertain::Uncertain;

const SAMPLES: usize = 1000;

/// Latitude/longitude pair carried as the chain's initial value.
#[derive(Debug, Clone, Default)]
pub struct Position {
    pub lat: Uncertain<f64>,
    pub lon: Uncertain<f64>,
}

/// Stage 1 — propagate position noise into distance (miles).
pub fn distance_stage(start: Position, destination: Position) -> Uncertain<f64> {
    let lat_diff = destination.lat + (-start.lat);
    let lon_diff = destination.lon + (-start.lon);
    let distance_sq = lat_diff.clone() * lat_diff + lon_diff.clone() * lon_diff;
    // sqrt of the squared coordinate diff, times ~69 mi per degree at this latitude.
    let distance = distance_sq.map(|x| x.sqrt() * 69.0);

    println!("📍 [Stage 1] Distance");
    let mean = distance.expected_value(SAMPLES).unwrap_or(f64::NAN);
    let std = distance.standard_deviation(SAMPLES).unwrap_or(f64::NAN);
    println!("   mean: {mean:.3} mi, std: {std:.4} mi");
    println!(
        "   95% CI: {:.3} – {:.3} mi",
        mean - 1.96 * std,
        mean + 1.96 * std
    );

    distance
}

/// Stage 2 — propagate distance and speed noise into a travel-time estimate (minutes).
pub fn time_stage(distance: Uncertain<f64>) -> Uncertain<f64> {
    let base_speed = Uncertain::normal(35.0, 8.0); // mph with driver/traffic noise
    let traffic_factor = Uncertain::uniform(0.6, 1.0); // congestion drag
    let actual_speed = base_speed * traffic_factor;
    let travel_hours = distance.clone() / actual_speed;
    let travel_minutes = travel_hours * Uncertain::<f64>::point(60.0);

    println!("\n⏱️  [Stage 2] Travel time");
    let mean = travel_minutes.expected_value(SAMPLES).unwrap_or(f64::NAN);
    let std = travel_minutes
        .standard_deviation(SAMPLES)
        .unwrap_or(f64::NAN);
    let late = travel_minutes.greater_than(10.0);
    let p_late = late.estimate_probability(SAMPLES).unwrap_or(f64::NAN) * 100.0;
    println!("   mean: {mean:.1} min, std: {std:.1} min");
    println!("   P(>10 min): {p_late:.1}%");

    // Carry travel time downstream — the route stage compares against it.
    travel_minutes
}

/// Stage 3 — compare main-route time against a longer-but-steadier alternative.
pub fn route_stage(main_time: Uncertain<f64>) -> Uncertain<f64> {
    let alt_distance = Uncertain::<f64>::point(2.2); // slightly longer (mi)
    let alt_speed = Uncertain::normal(45.0, 3.0); // highway, less variance
    let alt_time = alt_distance / alt_speed * Uncertain::<f64>::point(60.0);

    let main_faster = main_time.lt_uncertain(&alt_time);
    let confidence = main_faster
        .estimate_probability(SAMPLES)
        .unwrap_or(f64::NAN)
        * 100.0;

    println!("\n🛣️  [Stage 3] Route decision");
    println!("   P(main faster than alt): {confidence:.1}%");

    let chosen = Uncertain::conditional(main_faster.clone(), main_time.clone(), alt_time.clone());
    let chosen_mean = chosen.expected_value(SAMPLES).unwrap_or(f64::NAN);
    println!("   chosen route expected time: {chosen_mean:.1} min");

    match main_faster.implicit_conditional() {
        Ok(true) => println!("   ✅ Recommend: main route"),
        Ok(false) => println!("   ✅ Recommend: alternative route"),
        Err(_) => println!("   ⚠️  Recommendation undecided"),
    }

    chosen
}

/// Stage 4 — propagate distance into a fuel consumption estimate (gallons). The carried value is the
/// chosen route time, which fuel does not use; it models a fresh planned-trip distance, matching the
/// original example's stage-local computation.
pub fn fuel_stage(_carried: Uncertain<f64>) -> Uncertain<f64> {
    let distance = Uncertain::<f64>::point(2.0); // ~2 mi planned trip
    let efficiency = Uncertain::normal(28.0, 4.0); // mpg
    let fuel = distance.clone() / efficiency;
    let mean_fuel = fuel.expected_value(SAMPLES).unwrap_or(f64::NAN);
    println!("\n⛽ [Stage 4] Fuel");
    println!("   expected: {mean_fuel:.3} gal");

    let current_fuel = Uncertain::uniform(0.8, 1.2);
    let enough = current_fuel.gt_uncertain(&fuel);
    let p_enough = enough.estimate_probability(SAMPLES).unwrap_or(f64::NAN) * 100.0;
    println!("   P(have enough fuel): {p_enough:.1}%");

    let within_safe = fuel.within_range(0.5, 2.0);
    let p_safe = within_safe
        .estimate_probability(SAMPLES)
        .unwrap_or(f64::NAN)
        * 100.0;
    println!("   P(needed fuel in safe range 0.5–2.0 gal): {p_safe:.1}%");

    match enough.probability_exceeds(0.8, 0.95, 0.05, SAMPLES) {
        Ok(true) => println!("   ✅ Likely enough fuel for the trip"),
        Ok(false) => println!("   ⚠️  Consider refueling before the trip"),
        Err(_) => println!("   ⚠️  Fuel-sufficiency check inconclusive"),
    }

    fuel
}
