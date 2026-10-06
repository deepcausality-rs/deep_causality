/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Route context and stage functions for the GPS-navigation `CausalFlow` chain.
//!
//! Each stage takes the previous stage's `Uncertain<f64>` directly, reads the route quantities it
//! needs from the route context, propagates uncertainty through one physical transformation, prints
//! the resulting statistics, and returns the next `Uncertain<f64>`. `CausalFlow` supplies the
//! chain's plumbing, so no stage touches `CausalEffect` or re-lifts with `PropagatingEffect::pure`.

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::CausalityError;
use deep_causality_uncertain::{Uncertain, UncertainError};

const SAMPLES: usize = 1000;

/// The route, vehicle and trip facts the stages read, one `Data` contextoid per quantity. An
/// uncertain quantity is held as the parameters of its distribution; the stage that reads it
/// builds the distribution. The destination is two of those quantities, latitude and longitude,
/// rather than a spatial node. The context holds no position, clock or event, so its spatial,
/// temporal and spacetime slots are empty.
pub type RouteContext = Context<Data<f64>, NoSpace<f64>, NoTime, NoSpaceTime<f64>>;

/// Node index: destination latitude in degrees.
pub const DESTINATION_LAT: usize = 0;
/// Node index: destination longitude in degrees.
pub const DESTINATION_LON: usize = 1;
/// Node index: mean of the normally distributed main-route base speed, in mph.
pub const BASE_SPEED_MEAN: usize = 2;
/// Node index: standard deviation of the main-route base speed (driver and traffic noise), in mph.
pub const BASE_SPEED_SD: usize = 3;
/// Node index: lower bound of the uniformly distributed congestion factor on the base speed.
pub const TRAFFIC_FACTOR_MIN: usize = 4;
/// Node index: upper bound of the uniformly distributed congestion factor on the base speed.
pub const TRAFFIC_FACTOR_MAX: usize = 5;
/// Node index: travel time beyond which the trip is late, in minutes.
pub const LATE_AFTER_MIN: usize = 6;
/// Node index: alternative-route distance in miles.
pub const ALT_DISTANCE: usize = 7;
/// Node index: mean of the normally distributed alternative-route speed, in mph.
pub const ALT_SPEED_MEAN: usize = 8;
/// Node index: standard deviation of the alternative-route speed, in mph.
pub const ALT_SPEED_SD: usize = 9;
/// Node index: planned-trip distance for the fuel estimate, in miles.
pub const TRIP_DISTANCE: usize = 10;
/// Node index: mean of the normally distributed fuel efficiency, in miles per gallon.
pub const FUEL_EFFICIENCY_MEAN: usize = 11;
/// Node index: standard deviation of the fuel efficiency, in miles per gallon.
pub const FUEL_EFFICIENCY_SD: usize = 12;
/// Node index: lower bound of the uniformly distributed fuel on hand, in gallons.
pub const FUEL_ON_HAND_MIN: usize = 13;
/// Node index: upper bound of the uniformly distributed fuel on hand, in gallons.
pub const FUEL_ON_HAND_MAX: usize = 14;
/// Node index: lower bound of the safe range for the fuel the trip needs, in gallons.
pub const SAFE_FUEL_MIN: usize = 15;
/// Node index: upper bound of the safe range for the fuel the trip needs, in gallons.
pub const SAFE_FUEL_MAX: usize = 16;
/// Node index: probability of having enough fuel above which no refuelling is advised.
pub const ENOUGH_FUEL_PROBABILITY: usize = 17;

/// The route, added in node-index order: node `i` holds contextoid id `i + 1`.
pub fn route_context() -> Result<RouteContext, ContextIndexError> {
    let mut context = Context::with_capacity(1, "route", 18);
    for (id, value) in [
        (1, 37.7849),   // DESTINATION_LAT: ~1 mile north
        (2, -122.4094), // DESTINATION_LON: ~1 mile east
        (3, 35.0),      // BASE_SPEED_MEAN
        (4, 8.0),       // BASE_SPEED_SD
        (5, 0.6),       // TRAFFIC_FACTOR_MIN: congestion drag
        (6, 1.0),       // TRAFFIC_FACTOR_MAX
        (7, 10.0),      // LATE_AFTER_MIN
        (8, 2.2),       // ALT_DISTANCE: slightly longer
        (9, 45.0),      // ALT_SPEED_MEAN: highway
        (10, 3.0),      // ALT_SPEED_SD: less variance
        (11, 2.0),      // TRIP_DISTANCE: ~2 mi planned trip
        (12, 28.0),     // FUEL_EFFICIENCY_MEAN
        (13, 4.0),      // FUEL_EFFICIENCY_SD
        (14, 0.8),      // FUEL_ON_HAND_MIN
        (15, 1.2),      // FUEL_ON_HAND_MAX
        (16, 0.5),      // SAFE_FUEL_MIN
        (17, 2.0),      // SAFE_FUEL_MAX
        (18, 0.8),      // ENOUGH_FUEL_PROBABILITY
    ] {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}

/// Read one `Data` contextoid's payload out of the route context.
pub fn read(context: &RouteContext, index: usize) -> Result<f64, CausalityError> {
    context
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(|data| data.get_data())
        .ok_or_else(|| {
            CausalityError::MissingParameter(format!(
                "the route context holds no Datoid at node index {index}"
            ))
        })
}

/// A sampling error, carried in the chain's error channel.
fn sampling_error(error: UncertainError) -> CausalityError {
    CausalityError::UncertainError(error.to_string())
}

/// Latitude/longitude pair carried as the chain's initial value.
#[derive(Debug, Clone, Default)]
pub struct Position {
    pub lat: Uncertain<f64>,
    pub lon: Uncertain<f64>,
}

/// Stage 1 — propagate position noise into distance (miles) to the destination in the route context.
pub fn distance_stage(
    start: Position,
    _state: &(),
    route: Option<&RouteContext>,
) -> Result<Uncertain<f64>, CausalityError> {
    let route = route.ok_or(CausalityError::MissingContext())?;
    let lat_diff = Uncertain::<f64>::point(read(route, DESTINATION_LAT)?) + (-start.lat);
    let lon_diff = Uncertain::<f64>::point(read(route, DESTINATION_LON)?) + (-start.lon);
    let distance_sq = lat_diff.clone() * lat_diff + lon_diff.clone() * lon_diff;
    // sqrt of the squared coordinate diff, times ~69 mi per degree at this latitude.
    let distance = distance_sq.map(|x| x.sqrt() * 69.0);

    println!("📍 [Stage 1] Distance");
    let mean = distance
        .expected_value_from_entropy(SAMPLES)
        .map_err(sampling_error)?;
    let std = distance
        .standard_deviation_from_entropy(SAMPLES)
        .map_err(sampling_error)?;
    println!("   mean: {mean:.3} mi, std: {std:.4} mi");
    println!(
        "   95% CI: {:.3} – {:.3} mi",
        mean - 1.96 * std,
        mean + 1.96 * std
    );

    Ok(distance)
}

/// Stage 2 — propagate distance and speed noise into a travel-time estimate (minutes).
pub fn time_stage(
    distance: Uncertain<f64>,
    _state: &(),
    route: Option<&RouteContext>,
) -> Result<Uncertain<f64>, CausalityError> {
    let route = route.ok_or(CausalityError::MissingContext())?;
    let base_speed = Uncertain::normal(read(route, BASE_SPEED_MEAN)?, read(route, BASE_SPEED_SD)?);
    let traffic_factor = Uncertain::uniform(
        read(route, TRAFFIC_FACTOR_MIN)?,
        read(route, TRAFFIC_FACTOR_MAX)?,
    );
    let late_after = read(route, LATE_AFTER_MIN)?;
    let actual_speed = base_speed * traffic_factor;
    let travel_hours = distance.clone() / actual_speed;
    let travel_minutes = travel_hours * Uncertain::<f64>::point(60.0);

    println!("\n⏱️  [Stage 2] Travel time");
    let mean = travel_minutes
        .expected_value_from_entropy(SAMPLES)
        .map_err(sampling_error)?;
    let std = travel_minutes
        .standard_deviation_from_entropy(SAMPLES)
        .map_err(sampling_error)?;
    let late = travel_minutes.greater_than(late_after);
    let p_late = late
        .estimate_probability_from_entropy(SAMPLES)
        .map_err(sampling_error)?
        * 100.0;
    println!("   mean: {mean:.1} min, std: {std:.1} min");
    println!("   P(>{late_after} min): {p_late:.1}%");

    // Carry travel time downstream — the route stage compares against it.
    Ok(travel_minutes)
}

/// Stage 3 — compare main-route time against a longer-but-steadier alternative.
pub fn route_stage(
    main_time: Uncertain<f64>,
    _state: &(),
    route: Option<&RouteContext>,
) -> Result<Uncertain<f64>, CausalityError> {
    let route = route.ok_or(CausalityError::MissingContext())?;
    let alt_distance = Uncertain::<f64>::point(read(route, ALT_DISTANCE)?);
    let alt_speed = Uncertain::normal(read(route, ALT_SPEED_MEAN)?, read(route, ALT_SPEED_SD)?);
    let alt_time = alt_distance / alt_speed * Uncertain::<f64>::point(60.0);

    let main_faster = main_time.lt_uncertain(&alt_time);
    let confidence = main_faster
        .estimate_probability_from_entropy(SAMPLES)
        .map_err(sampling_error)?
        * 100.0;

    println!("\n🛣️  [Stage 3] Route decision");
    println!("   P(main faster than alt): {confidence:.1}%");

    let chosen = Uncertain::conditional(main_faster.clone(), main_time.clone(), alt_time.clone());
    let chosen_mean = chosen
        .expected_value_from_entropy(SAMPLES)
        .map_err(sampling_error)?;
    println!("   chosen route expected time: {chosen_mean:.1} min");

    match main_faster.implicit_conditional_from_entropy() {
        Ok(true) => println!("   ✅ Recommend: main route"),
        Ok(false) => println!("   ✅ Recommend: alternative route"),
        Err(_) => println!("   ⚠️  Recommendation undecided"),
    }

    Ok(chosen)
}

/// Stage 4 — propagate distance into a fuel consumption estimate (gallons). The carried value is the
/// chosen route time, which fuel does not use; the stage reads the planned-trip distance, fuel
/// efficiency, fuel on hand, safe fuel range and required probability from the route context.
pub fn fuel_stage(
    _carried: Uncertain<f64>,
    _state: &(),
    route: Option<&RouteContext>,
) -> Result<Uncertain<f64>, CausalityError> {
    let route = route.ok_or(CausalityError::MissingContext())?;
    let distance = Uncertain::<f64>::point(read(route, TRIP_DISTANCE)?);
    let efficiency = Uncertain::normal(
        read(route, FUEL_EFFICIENCY_MEAN)?,
        read(route, FUEL_EFFICIENCY_SD)?,
    );
    let fuel = distance.clone() / efficiency;
    let mean_fuel = fuel
        .expected_value_from_entropy(SAMPLES)
        .map_err(sampling_error)?;
    println!("\n⛽ [Stage 4] Fuel");
    println!("   expected: {mean_fuel:.3} gal");

    let current_fuel = Uncertain::uniform(
        read(route, FUEL_ON_HAND_MIN)?,
        read(route, FUEL_ON_HAND_MAX)?,
    );
    let enough = current_fuel.gt_uncertain(&fuel);
    let p_enough = enough
        .estimate_probability_from_entropy(SAMPLES)
        .map_err(sampling_error)?
        * 100.0;
    println!("   P(have enough fuel): {p_enough:.1}%");

    let safe_min = read(route, SAFE_FUEL_MIN)?;
    let safe_max = read(route, SAFE_FUEL_MAX)?;
    let within_safe = fuel.within_range(safe_min, safe_max);
    let p_safe = within_safe
        .estimate_probability_from_entropy(SAMPLES)
        .map_err(sampling_error)?
        * 100.0;
    println!("   P(needed fuel in safe range {safe_min:.1}–{safe_max:.1} gal): {p_safe:.1}%");

    match enough.probability_exceeds_from_entropy(
        read(route, ENOUGH_FUEL_PROBABILITY)?,
        0.95,
        0.05,
        SAMPLES,
    ) {
        Ok(true) => println!("   ✅ Likely enough fuel for the trip"),
        Ok(false) => println!("   ⚠️  Consider refueling before the trip"),
        Err(_) => println!("   ⚠️  Fuel-sufficiency check inconclusive"),
    }

    Ok(fuel)
}
