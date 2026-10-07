/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Route context and stage functions for the GPS-navigation `CausalFlow` chain.
//!
//! Each stage takes the previous stage's `Uncertain<FloatType>` directly, reads the route
//! quantities it needs from the route context, propagates uncertainty through one physical
//! transformation, prints the resulting statistics, and returns the next `Uncertain<FloatType>`.
//! `CausalFlow` supplies the chain's plumbing, so no stage touches `CausalEffect` or re-lifts with
//! `PropagatingEffect::pure`.

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::CausalityError;
use deep_causality_num::lift;
use deep_causality_uncertain::{Uncertain, UncertainError};

const SAMPLES: usize = 1000;

/// The working precision of the example: every route quantity, distribution parameter and
/// uncertain value is a `FloatType`.
pub type FloatType = f64;

/// The route, vehicle and trip facts the stages read, one `Data` contextoid per quantity. An
/// uncertain quantity is held as the parameters of its distribution; the stage that reads it
/// builds the distribution. The destination is two of those quantities, latitude and longitude,
/// rather than a spatial node. The context holds no position, clock or event, so its spatial,
/// temporal and spacetime slots are empty.
pub type RouteContext =
    Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Contextoid id: destination latitude in degrees.
pub const DESTINATION_LAT: ContextoidId = 1;
/// Contextoid id: destination longitude in degrees.
pub const DESTINATION_LON: ContextoidId = 2;
/// Contextoid id: mean of the normally distributed main-route base speed, in mph.
pub const BASE_SPEED_MEAN: ContextoidId = 3;
/// Contextoid id: standard deviation of the main-route base speed (driver and traffic noise), in
/// mph.
pub const BASE_SPEED_SD: ContextoidId = 4;
/// Contextoid id: lower bound of the uniformly distributed congestion factor on the base speed.
pub const TRAFFIC_FACTOR_MIN: ContextoidId = 5;
/// Contextoid id: upper bound of the uniformly distributed congestion factor on the base speed.
pub const TRAFFIC_FACTOR_MAX: ContextoidId = 6;
/// Contextoid id: travel time beyond which the trip is late, in minutes.
pub const LATE_AFTER_MIN: ContextoidId = 7;
/// Contextoid id: alternative-route distance in miles.
pub const ALT_DISTANCE: ContextoidId = 8;
/// Contextoid id: mean of the normally distributed alternative-route speed, in mph.
pub const ALT_SPEED_MEAN: ContextoidId = 9;
/// Contextoid id: standard deviation of the alternative-route speed, in mph.
pub const ALT_SPEED_SD: ContextoidId = 10;
/// Contextoid id: planned-trip distance for the fuel estimate, in miles.
pub const TRIP_DISTANCE: ContextoidId = 11;
/// Contextoid id: mean of the normally distributed fuel efficiency, in miles per gallon.
pub const FUEL_EFFICIENCY_MEAN: ContextoidId = 12;
/// Contextoid id: standard deviation of the fuel efficiency, in miles per gallon.
pub const FUEL_EFFICIENCY_SD: ContextoidId = 13;
/// Contextoid id: lower bound of the uniformly distributed fuel on hand, in gallons.
pub const FUEL_ON_HAND_MIN: ContextoidId = 14;
/// Contextoid id: upper bound of the uniformly distributed fuel on hand, in gallons.
pub const FUEL_ON_HAND_MAX: ContextoidId = 15;
/// Contextoid id: lower bound of the safe range for the fuel the trip needs, in gallons.
pub const SAFE_FUEL_MIN: ContextoidId = 16;
/// Contextoid id: upper bound of the safe range for the fuel the trip needs, in gallons.
pub const SAFE_FUEL_MAX: ContextoidId = 17;
/// Contextoid id: probability of having enough fuel above which no refuelling is advised.
pub const ENOUGH_FUEL_PROBABILITY: ContextoidId = 18;

/// The route: one `Data` contextoid per quantity, keyed by its contextoid id.
pub fn route_context() -> Result<RouteContext, ContextIndexError> {
    let facts = [
        (DESTINATION_LAT, lift(37.7849)),   // ~1 mile north
        (DESTINATION_LON, lift(-122.4094)), // ~1 mile east
        (BASE_SPEED_MEAN, lift(35.0)),
        (BASE_SPEED_SD, lift(8.0)),
        (TRAFFIC_FACTOR_MIN, lift(0.6)), // congestion drag
        (TRAFFIC_FACTOR_MAX, lift(1.0)),
        (LATE_AFTER_MIN, lift(10.0)),
        (ALT_DISTANCE, lift(2.2)),    // slightly longer
        (ALT_SPEED_MEAN, lift(45.0)), // highway
        (ALT_SPEED_SD, lift(3.0)),    // less variance
        (TRIP_DISTANCE, lift(2.0)),   // ~2 mi planned trip
        (FUEL_EFFICIENCY_MEAN, lift(28.0)),
        (FUEL_EFFICIENCY_SD, lift(4.0)),
        (FUEL_ON_HAND_MIN, lift(0.8)),
        (FUEL_ON_HAND_MAX, lift(1.2)),
        (SAFE_FUEL_MIN, lift(0.5)),
        (SAFE_FUEL_MAX, lift(2.0)),
        (ENOUGH_FUEL_PROBABILITY, lift(0.8)),
    ];
    let mut context = Context::with_capacity(1, "route", facts.len());
    for (id, value) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}

/// Read the payload of the `Data` contextoid with contextoid id `id` out of the route context.
pub fn read(context: &RouteContext, id: ContextoidId) -> Result<FloatType, CausalityError> {
    context.get_data_by_id(id).ok_or_else(|| {
        CausalityError::MissingParameter(format!(
            "the route context holds no Datoid with contextoid id {id}"
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
    pub lat: Uncertain<FloatType>,
    pub lon: Uncertain<FloatType>,
}

/// Stage 1 — propagate position noise into distance (miles) to the destination in the route context.
pub fn distance_stage(
    start: Position,
    _state: &(),
    route: Option<&RouteContext>,
) -> Result<Uncertain<FloatType>, CausalityError> {
    let route = route.ok_or(CausalityError::MissingContext())?;
    let lat_diff = Uncertain::<FloatType>::point(read(route, DESTINATION_LAT)?) + (-start.lat);
    let lon_diff = Uncertain::<FloatType>::point(read(route, DESTINATION_LON)?) + (-start.lon);
    let distance_sq = lat_diff.clone() * lat_diff + lon_diff.clone() * lon_diff;
    // sqrt of the squared coordinate diff, times ~69 mi per degree at this latitude.
    let distance = distance_sq.map(|x| x.sqrt() * lift::<FloatType>(69.0));

    println!("📍 [Stage 1] Distance");
    let mean = distance
        .expected_value_from_entropy(SAMPLES)
        .map_err(sampling_error)?;
    let std = distance
        .standard_deviation_from_entropy(SAMPLES)
        .map_err(sampling_error)?;
    println!("   mean: {mean:.3} mi, std: {std:.4} mi");
    let half_width = lift::<FloatType>(1.96) * std;
    println!(
        "   95% CI: {:.3} – {:.3} mi",
        mean - half_width,
        mean + half_width
    );

    Ok(distance)
}

/// Stage 2 — propagate distance and speed noise into a travel-time estimate (minutes).
pub fn time_stage(
    distance: Uncertain<FloatType>,
    _state: &(),
    route: Option<&RouteContext>,
) -> Result<Uncertain<FloatType>, CausalityError> {
    let route = route.ok_or(CausalityError::MissingContext())?;
    let base_speed = Uncertain::normal(read(route, BASE_SPEED_MEAN)?, read(route, BASE_SPEED_SD)?);
    let traffic_factor = Uncertain::uniform(
        read(route, TRAFFIC_FACTOR_MIN)?,
        read(route, TRAFFIC_FACTOR_MAX)?,
    );
    let late_after = read(route, LATE_AFTER_MIN)?;
    let actual_speed = base_speed * traffic_factor;
    let travel_hours = distance.clone() / actual_speed;
    let travel_minutes = travel_hours * Uncertain::<FloatType>::point(lift(60.0));

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
        * lift::<FloatType>(100.0);
    println!("   mean: {mean:.1} min, std: {std:.1} min");
    println!("   P(>{late_after} min): {p_late:.1}%");

    // Carry travel time downstream — the route stage compares against it.
    Ok(travel_minutes)
}

/// Stage 3 — compare main-route time against a longer-but-steadier alternative.
pub fn route_stage(
    main_time: Uncertain<FloatType>,
    _state: &(),
    route: Option<&RouteContext>,
) -> Result<Uncertain<FloatType>, CausalityError> {
    let route = route.ok_or(CausalityError::MissingContext())?;
    let alt_distance = Uncertain::<FloatType>::point(read(route, ALT_DISTANCE)?);
    let alt_speed = Uncertain::normal(read(route, ALT_SPEED_MEAN)?, read(route, ALT_SPEED_SD)?);
    let alt_time = alt_distance / alt_speed * Uncertain::<FloatType>::point(lift(60.0));

    let main_faster = main_time.lt_uncertain(&alt_time);
    let confidence = main_faster
        .estimate_probability_from_entropy(SAMPLES)
        .map_err(sampling_error)?
        * lift::<FloatType>(100.0);

    println!("\n🛣️  [Stage 3] Route decision");
    println!("   P(main faster than alt): {confidence:.1}%");

    let chosen = Uncertain::conditional(main_faster.clone(), main_time.clone(), alt_time.clone());
    let chosen_mean = chosen
        .expected_value_from_entropy(SAMPLES)
        .map_err(sampling_error)?;
    println!("   chosen route expected time: {chosen_mean:.1} min");

    if main_faster
        .implicit_conditional_from_entropy()
        .map_err(sampling_error)?
    {
        println!("   ✅ Recommend: main route");
    } else {
        println!("   ✅ Recommend: alternative route");
    }

    Ok(chosen)
}

/// Stage 4 — propagate distance into a fuel consumption estimate (gallons). The carried value is the
/// chosen route time, which fuel does not use; the stage reads the planned-trip distance, fuel
/// efficiency, fuel on hand, safe fuel range and required probability from the route context.
pub fn fuel_stage(
    _carried: Uncertain<FloatType>,
    _state: &(),
    route: Option<&RouteContext>,
) -> Result<Uncertain<FloatType>, CausalityError> {
    let route = route.ok_or(CausalityError::MissingContext())?;
    let distance = Uncertain::<FloatType>::point(read(route, TRIP_DISTANCE)?);
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
        * lift::<FloatType>(100.0);
    println!("   P(have enough fuel): {p_enough:.1}%");

    let safe_min = read(route, SAFE_FUEL_MIN)?;
    let safe_max = read(route, SAFE_FUEL_MAX)?;
    let within_safe = fuel.within_range(safe_min, safe_max);
    let p_safe = within_safe
        .estimate_probability_from_entropy(SAMPLES)
        .map_err(sampling_error)?
        * lift::<FloatType>(100.0);
    println!("   P(needed fuel in safe range {safe_min:.1}–{safe_max:.1} gal): {p_safe:.1}%");

    if enough
        .probability_exceeds_from_entropy(
            read(route, ENOUGH_FUEL_PROBABILITY)?,
            lift(0.95),
            lift(0.05),
            SAMPLES,
        )
        .map_err(sampling_error)?
    {
        println!("   ✅ Likely enough fuel for the trip");
    } else {
        println!("   ⚠️  Consider refueling before the trip");
    }

    Ok(fuel)
}
