/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The fault detectors' thresholds, how long a fault must persist before the controller acts, the
//! rules the controller reads the ground by, the fail-safe machine's states, and the norms of the
//! Effect Ethos with the facts they read.

use deep_causality_context::ContextoidId;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int};
use dynamic_drone_failsafe::FloatType;

/// Satellite positioning is degraded below this many satellites or above this dilution of
/// precision, and has no fix below this many satellites.
pub const DEGRADED_SATELLITES: FloatType = const_scalar_from_int!(FloatType, 8);
pub const DEGRADED_HDOP: FloatType = const_scalar_from_int!(FloatType, 2);
pub const NO_FIX_SATELLITES: FloatType = const_scalar_from_int!(FloatType, 5);
/// The command link is down above this packet loss, in percent.
pub const LINK_DOWN_LOSS_PCT: FloatType = const_scalar_from_int!(FloatType, 80);
/// The battery is critical below this lowest-cell voltage.
pub const CRITICAL_CELL_V: FloatType = const_scalar_from_float!(FloatType, 3.4);
/// A lost fix and a lost link count only once they have lasted this long, in seconds, so one
/// dropped packet or one bad epoch does not trigger a fail-safe.
pub const GNSS_CONFIRM_S: usize = 3;
pub const LINK_CONFIRM_S: usize = 5;

/// A drone on skids tips over on ground steeper than this, in degrees.
pub const TIP_OVER_DEG: FloatType = const_scalar_from_int!(FloatType, 15);
/// Below this fraction of LiDAR returns, the beam found no surface to reflect from.
pub const NO_RETURN_BELOW: FloatType = const_scalar_from_float!(FloatType, 0.5);
/// Water differs from the land around it by more than this, in degrees Celsius: warmer at night,
/// cooler by day.
pub const WATER_CONTRAST_C: FloatType = const_scalar_from_int!(FloatType, 2);
/// A hot spot this much warmer than the land, in degrees Celsius, or a bump this tall, in m,
/// marks a person.
pub const PERSON_EXCESS_C: FloatType = const_scalar_from_int!(FloatType, 15);
pub const PERSON_BUMP_M: FloatType = const_scalar_from_int!(FloatType, 1);
/// A patch is judged safe, or judged a hazard, only when that holds with probability above this.
pub const CONFIDENCE: FloatType = const_scalar_from_float!(FloatType, 0.95);
/// The sequential test that decides it: the half-width of its indifference band, its most draws,
/// and the seed it draws from.
pub const TEST_EPSILON: FloatType = const_scalar_from_float!(FloatType, 0.025);
pub const MAX_SAMPLES: usize = 400;
pub const JUDGE_SEED: u64 = 9090;

pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const ONE: FloatType = const_scalar_from_int!(FloatType, 1);

/// The context's identifier and the identifier of its clock node.
pub const GROUND_CONTEXT_ID: ContextoidId = 1;
pub const CLOCK_ID: ContextoidId = 1;
/// Night lasts from dusk to dawn, in hours of the local day.
pub const DUSK_HOUR: u64 = 19;
pub const DAWN_HOUR: u64 = 6;
pub const SECONDS_PER_HOUR: u64 = 3600;
pub const HOURS_PER_DAY: u64 = 24;

/// The fail-safe machine's states, in the order it evaluates them each second.
pub const CHOOSE_STATE: usize = 1;
pub const HOLD_STATE: usize = 2;
pub const HOME_STATE: usize = 3;
pub const LAND_STATE: usize = 4;
pub const LOOK_STATE: usize = 5;

/// The context nodes that hold the drone's own state, beside the clock: where the frames place it,
/// its height, how far that place may be off, how long its battery lasts, and the temperature of
/// the land below it.
pub const DRONE_ACROSS_ID: ContextoidId = 2;
pub const DRONE_ALONG_ID: ContextoidId = 3;
pub const DRONE_AGL_ID: ContextoidId = 4;
pub const POSITION_ERROR_ID: ContextoidId = 5;
pub const ENDURANCE_ID: ContextoidId = 6;
pub const LAND_TEMPERATURE_ID: ContextoidId = 7;

/// The side of one ground patch, in m.
pub const PATCH_SIDE_M: FloatType = const_scalar_from_int!(FloatType, 4);
/// The drone's own flight: approach speed, and descent rates for an ordinary landing and on a
/// failing battery, in m/s.
pub const APPROACH_SPEED_M_S: FloatType = const_scalar_from_int!(FloatType, 3);
pub const LANDING_DESCENT_M_S: FloatType = const_scalar_from_float!(FloatType, 1.5);
pub const EMERGENCY_DESCENT_M_S: FloatType = const_scalar_from_int!(FloatType, 3);
/// The lowest cell loses this much per second of flight, in V, and a critical battery still
/// flies this long, in s. A landing must leave this reserve, in s.
pub const CELL_DRAIN_V_PER_S: FloatType = const_scalar_from_float!(FloatType, 0.002);
pub const CRITICAL_ENDURANCE_S: FloatType = const_scalar_from_int!(FloatType, 20);
pub const RESERVE_S: FloatType = const_scalar_from_int!(FloatType, 5);

/// The drone must touch down at least this far from any person, in m. The clearance widens by
/// the drone's position error, which grows this much per second without a satellite fix, and by
/// how far a gust can push it in its last second of descent, in m.
pub const PERSON_CLEARANCE_M: FloatType = const_scalar_from_int!(FloatType, 10);
pub const POSITION_DRIFT_M_PER_S: FloatType = const_scalar_from_float!(FloatType, 0.1);
pub const GUST_MARGIN_M: FloatType = const_scalar_from_int!(FloatType, 2);
/// A person is ruled out on a patch only when no person is there with this probability, decided
/// by a sequential test with this indifference band and at most this many draws. The judge names
/// a person at 95 %; the Effect Ethos rules one out only at 99 %.
pub const PERSON_RULED_OUT: FloatType = const_scalar_from_float!(FloatType, 0.99);
pub const PERSON_TEST_EPSILON: FloatType = const_scalar_from_float!(FloatType, 0.005);
pub const PERSON_TEST_SAMPLES: usize = 2000;

/// The norms of the Effect Ethos, the tag that selects them, and the most proposals the machine
/// puts to it in one round.
pub const NORM_LANDING: u64 = 1;
pub const NORM_PERSON: u64 = 2;
pub const NORM_NOT_RULED_OUT: u64 = 3;
pub const NORM_BATTERY: u64 = 4;
pub const NORM_DRONE: u64 = 5;
pub const NORM_SACRIFICE: u64 = 6;
pub const LANDING_TAG: &str = "landing";
pub const MAX_PROPOSALS: usize = 16;

/// The ground map spans this many patches either side of the touchdown point, across the slope and
/// along the line.
pub const MAP_HALF_ACROSS: i64 = 12;
pub const MAP_HALF_ALONG: i64 = 5;
