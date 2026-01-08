/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::{Uncertain, UncertainError};

/// GPS Navigation with Uncertainty-Aware Route Planning
///
/// This example demonstrates how uncertain GPS readings affect
/// arrival time predictions and route decisions, leveraging the
/// enhanced API of `deep_causality_uncertain`.
fn main() -> Result<(), UncertainError> {
    println!("🚗 GPS Navigation with Uncertainty Analysis");
    println!("===========================================\n");

    // GPS position has uncertainty due to satellite positioning errors
    let current_lat = Uncertain::normal(37.7749, 0.0001); // San Francisco
    let current_lon = Uncertain::normal(-122.4194, 0.0001);

    let destination_lat = Uncertain::<f64>::point(37.7849); // 1 mile north
    let destination_lon = Uncertain::<f64>::point(-122.4094); // 1 mile east

    // Calculate distance with uncertainty propagation
    // Using unary negation (-) and map for sqrt
    let lat_diff = destination_lat + (-current_lat);
    let lon_diff = destination_lon + (-current_lon);

    let distance_sq = lat_diff.clone() * lat_diff + lon_diff.clone() * lon_diff;
    let distance = distance_sq.map(|x| x.sqrt() * 69.0); // Convert to miles

    println!("📍 Distance Analysis:");
    let mean_distance = distance.expected_value(1000)?;
    let std_distance = distance.standard_deviation(1000)?;

    println!("   Mean distance: {:.3} miles", mean_distance);
    println!("   Std deviation: {:.4} miles", std_distance);
    println!(
        "   95% confidence: {:.3} - {:.3} miles",
        mean_distance - 1.96 * std_distance,
        mean_distance + 1.96 * std_distance
    );

    // Speed varies due to traffic, weather, driver behavior
    let base_speed = Uncertain::normal(35.0, 8.0); // mph, with uncertainty
    let traffic_factor = Uncertain::uniform(0.6, 1.0); // Traffic slows us down
    let actual_speed = base_speed * traffic_factor;

    // Calculate arrival time
    let travel_time_hours = distance.clone() / actual_speed;
    let travel_time_minutes = travel_time_hours * Uncertain::<f64>::point(60.0);

    println!("\n⏱️  Travel Time Analysis:");
    let mean_time = travel_time_minutes.expected_value(1000)?;
    let std_time = travel_time_minutes.standard_deviation(1000)?;

    println!("   Expected time: {:.1} minutes", mean_time);
    println!("   Std deviation: {:.1} minutes", std_time);

    // Probability analysis for arrival predictions
    let late_threshold = 10.0; // minutes
    let will_be_late = travel_time_minutes.greater_than(late_threshold);

    println!(
        "   Probability of taking >{}min: {:.1}%",
        late_threshold,
        will_be_late.estimate_probability(1000)? * 100.0
    );

    // Route decision with uncertainty
    println!("\n🛣️  Route Decision Analysis:");

    // Alternative route: longer but more predictable
    let alt_distance = Uncertain::<f64>::point(2.2); // Slightly longer
    let alt_speed = Uncertain::normal(45.0, 3.0); // Highway, more predictable
    let alt_time = alt_distance / alt_speed * Uncertain::<f64>::point(60.0);

    // Compare routes using evidence-based reasoning
    let main_faster = travel_time_minutes.lt_uncertain(&alt_time);
    let confidence_main_faster = main_faster.estimate_probability(1000)?;

    println!(
        "   Main route faster: {:.1}% confidence",
        confidence_main_faster * 100.0
    );

    // Conditional logic: Choose route based on confidence
    let chosen_route_time = Uncertain::conditional(
        main_faster.clone(),
        travel_time_minutes.clone(),
        alt_time.clone(),
    );

    println!(
        "\n   Chosen route time (conditional): {:.1} minutes",
        chosen_route_time.expected_value(1000)?
    );

    if main_faster.implicit_conditional()? {
        println!("   ✅ Recommendation: Take main route (more likely faster)");
    } else {
        println!("   ✅ Recommendation: Take alternative route (more likely faster)");
    }

    // Fuel consumption analysis
    println!("\n⛽ Fuel Consumption Analysis:");
    let fuel_efficiency = Uncertain::normal(28.0, 4.0); // mpg
    let fuel_needed = distance / fuel_efficiency;

    let mean_fuel = fuel_needed.expected_value(1000)?;

    println!("   Expected fuel: {:.3} gallons", mean_fuel);

    // Check if we have enough fuel using approx_eq and within_range
    let current_fuel = Uncertain::uniform(0.8, 1.2); // Uncertain fuel gauge reading
    let enough_fuel = current_fuel.gt_uncertain(&fuel_needed);

    println!(
        "   Confidence we have enough fuel: {:.1}%",
        enough_fuel.estimate_probability(1000)? * 100.0
    );

    let target_fuel = Uncertain::<f64>::point(1.5); // Target fuel in gallons
    let fuel_is_approx_target = fuel_needed.approx_eq(target_fuel.expected_value(1000)?, 0.1);
    println!(
        "   Fuel needed is approx target: {:.1}% confident",
        fuel_is_approx_target.estimate_probability(1000)? * 100.0
    );

    let fuel_within_safe_range = fuel_needed.within_range(0.5, 2.0);
    println!(
        "   Fuel needed within safe range (0.5-2.0 gal): {:.1}% confident",
        fuel_within_safe_range.estimate_probability(1000)? * 100.0
    );

    if enough_fuel.probability_exceeds(0.8, 0.95, 0.05, 1000)? {
        println!("   ✅ You likely have enough fuel for the trip!");
    } else {
        println!("   ⚠️  Consider refueling before the trip!");
    }

    Ok(())
}
