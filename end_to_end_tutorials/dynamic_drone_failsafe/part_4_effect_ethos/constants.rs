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
pub const LAND_STATE: usize = 3;
pub const LOOK_STATE: usize = 4;

/// In a contingency the drone holds and waits this long for the fix or the link to come back, in
/// s, before it lands as soon as practicable. With nothing permitted it proposes again after this
/// long, in s.
pub const RECOVERY_WINDOW_S: usize = 20;
pub const REASSESS_S: usize = 5;
/// The drone looks at a patch from this height, in m: low enough for sharp readings, high enough to
/// clear a canopy with margin.
pub const LOOK_HEIGHT_M: FloatType = const_scalar_from_int!(FloatType, 25);
/// A landing that waits on a look counts this much longer, in s, when the machine ranks it
/// against landings permitted already.
pub const LOOK_ALLOWANCE_S: FloatType = const_scalar_from_int!(FloatType, 10);
/// A look that has not ruled out every person near the patch this long after the drone arrives over
/// it, in s, sets the patch aside for the rest of the contingency.
pub const LOOK_LIMIT_S: usize = 20;

/// The context nodes that hold the drone's own state, beside the clock: where the frames place it,
/// its height, how far that place may be off, how long its battery lasts, and the temperature of
/// the land below it.
pub const DRONE_ACROSS_ID: ContextoidId = 2;
pub const DRONE_ALONG_ID: ContextoidId = 3;
pub const DRONE_AGL_ID: ContextoidId = 4;
pub const POSITION_ERROR_ID: ContextoidId = 5;
pub const ENDURANCE_ID: ContextoidId = 6;
pub const LAND_TEMPERATURE_ID: ContextoidId = 7;
pub const DESCENT_RATE_ID: ContextoidId = 8;

/// The side of one ground patch, in m.
pub const PATCH_SIDE_M: FloatType = const_scalar_from_int!(FloatType, 4);
/// The drone's own flight: approach speed, and descent rates for an ordinary landing and on a
/// failing battery, in m/s.
pub const APPROACH_SPEED_M_S: FloatType = const_scalar_from_int!(FloatType, 3);
pub const LANDING_DESCENT_M_S: FloatType = const_scalar_from_float!(FloatType, 1.5);
pub const EMERGENCY_DESCENT_M_S: FloatType = const_scalar_from_int!(FloatType, 3);
/// The lowest cell loses this much per second of flight, in V, and is empty at this voltage, in V.
/// A drop of this much in one second is a failed cell, in V, after which the battery still flies
/// this long, in s. With this little endurance left the situation is an emergency, in s. A landing
/// must leave this reserve, in s.
pub const CELL_DRAIN_V_PER_S: FloatType = const_scalar_from_float!(FloatType, 0.002);
pub const CELL_EMPTY_V: FloatType = const_scalar_from_int!(FloatType, 3);
pub const CELL_SAG_V: FloatType = const_scalar_from_float!(FloatType, 0.1);
pub const CRITICAL_ENDURANCE_S: FloatType = const_scalar_from_int!(FloatType, 20);
pub const EMERGENCY_ENDURANCE_S: FloatType = const_scalar_from_int!(FloatType, 60);
pub const RESERVE_S: FloatType = const_scalar_from_int!(FloatType, 5);

/// The drone must touch down at least this far from any person, in m. The clearance widens by how
/// far off the drone may land from the patch it tracks, in m, and by how far a gust can push it in
/// its last second of descent, in m. The drone sees the patch and the people near it in the same
/// frames, so their relative error does not grow while the drone drifts without a fix.
pub const PERSON_CLEARANCE_M: FloatType = const_scalar_from_int!(FloatType, 10);
pub const LANDING_ACCURACY_M: FloatType = const_scalar_from_int!(FloatType, 1);
pub const GUST_MARGIN_M: FloatType = const_scalar_from_int!(FloatType, 2);
/// A bump taller than a person is not one, in m, and a bump taller than this is canopy, in m. The
/// drone crosses trees only this far above the canopy, in m, and crosses unseen ground only above
/// this height, in m.
pub const PERSON_TALLEST_M: FloatType = const_scalar_from_float!(FloatType, 2.5);
pub const CANOPY_LOWEST_M: FloatType = const_scalar_from_int!(FloatType, 5);
pub const CANOPY_MARGIN_M: FloatType = const_scalar_from_int!(FloatType, 5);
pub const UNSEEN_CEILING_M: FloatType = const_scalar_from_int!(FloatType, 30);
/// The harm order as costs, each ten times the one before: losing the drone, a person not ruled
/// out near the patch, and a person more likely than not within 20, 10 and 5 m of it. A person not
/// ruled out is priced in an emergency and as a last resort; a likely person is priced only as a
/// last resort, and forbids the patch outright before it. Landing upright costs nothing.
pub const COST_DRONE_LOST: i64 = 1_000;
pub const COST_NOT_RULED_OUT: i64 = 10_000;
pub const COST_WITHIN_20_M: i64 = 100_000;
pub const COST_WITHIN_10_M: i64 = 1_000_000;
pub const COST_WITHIN_5_M: i64 = 10_000_000;
pub const BAND_20_M: FloatType = const_scalar_from_int!(FloatType, 20);
pub const BAND_10_M: FloatType = const_scalar_from_int!(FloatType, 10);
pub const BAND_5_M: FloatType = const_scalar_from_int!(FloatType, 5);
/// A person is ruled out on a patch only when no person is there with this probability, decided
/// by a sequential test with this indifference band and at most this many draws. The judge names
/// a person at 95 %; the Effect Ethos rules one out only at 99 %.
pub const PERSON_RULED_OUT: FloatType = const_scalar_from_float!(FloatType, 0.99);
pub const PERSON_TEST_EPSILON: FloatType = const_scalar_from_float!(FloatType, 0.005);
pub const PERSON_TEST_SAMPLES: usize = 2000;
/// The Effect Ethos counts a person as present near a patch once that is more likely than not.
pub const PERSON_LIKELY: FloatType = const_scalar_from_float!(FloatType, 0.5);

/// The norms of the Effect Ethos, the tags that select them, and the most landings and ditchings
/// the machine puts to it in one round.
pub const NORM_LANDING: u64 = 1;
pub const NORM_PERSON: u64 = 2;
pub const NORM_NOT_RULED_OUT: u64 = 3;
pub const NORM_BATTERY: u64 = 4;
pub const NORM_DRONE: u64 = 5;
pub const NORM_SACRIFICE: u64 = 6;
pub const NORM_EMERGENCY: u64 = 7;
pub const NORM_PATH: u64 = 8;
pub const NORM_LAST_RESORT: u64 = 9;
pub const NORM_WITHIN_20_M: u64 = 10;
pub const NORM_WITHIN_10_M: u64 = 11;
pub const NORM_WITHIN_5_M: u64 = 12;
pub const NORM_NEAR_UNKNOWN: u64 = 13;
pub const NORM_EDGE: u64 = 14;
pub const LANDING_TAG: &str = "landing";
/// The tag that adds the emergency norm, which applies once a cell has failed.
pub const EMERGENCY_TAG: &str = "emergency";
/// The tag that adds the last-resort norms: the person bans yield to banded costs.
pub const LAST_RESORT_TAG: &str = "last resort";
pub const MAX_LANDINGS: usize = 24;
pub const MAX_DITCHINGS: usize = 48;

/// The ground map spans this many patches either side of the touchdown point, across the slope and
/// along the line.
pub const MAP_HALF_ACROSS: i64 = 12;
pub const MAP_HALF_ALONG: i64 = 5;
