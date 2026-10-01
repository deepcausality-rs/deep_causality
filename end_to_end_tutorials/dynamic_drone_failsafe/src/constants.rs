/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The world's constants: the terrain, the drone, its mission and the faults that end it.

use crate::FloatType;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int};

// =============================================================================
// Terrain: a valley cross-section, constant along the valley except where noted
// =============================================================================

/// The creek along the valley floor, from the floor up to this distance across the slope, in m.
pub const CREEK_EDGE_M: FloatType = const_scalar_from_int!(FloatType, 12);
/// The grass slope above the creek rises at this angle up to the road bench, in degrees.
pub const GRASS_SLOPE_DEG: FloatType = const_scalar_from_int!(FloatType, 30);
/// The flat access road cut into the slope, between these distances across, in m.
pub const ROAD_FROM_M: FloatType = const_scalar_from_int!(FloatType, 120);
pub const ROAD_TO_M: FloatType = const_scalar_from_int!(FloatType, 135);
/// The rock slope above the road rises at this angle, in degrees.
pub const ROCK_SLOPE_DEG: FloatType = const_scalar_from_int!(FloatType, 25);
/// Flat concrete pads under the line's towers: half their side, in m, and their centres.
pub const PAD_HALF_SIDE_M: FloatType = const_scalar_from_int!(FloatType, 6);
pub const PAD_CENTRES_M: [(FloatType, FloatType); 3] = [
    (
        const_scalar_from_int!(FloatType, 100),
        const_scalar_from_int!(FloatType, 150),
    ),
    (
        const_scalar_from_int!(FloatType, 100),
        const_scalar_from_int!(FloatType, 450),
    ),
    (
        const_scalar_from_int!(FloatType, 100),
        const_scalar_from_int!(FloatType, 750),
    ),
];
/// A ravine cut across the rock slope between these distances along the valley, in m, and its depth.
pub const RAVINE_FROM_M: FloatType = const_scalar_from_int!(FloatType, 560);
pub const RAVINE_TO_M: FloatType = const_scalar_from_int!(FloatType, 580);
pub const RAVINE_DEPTH_M: FloatType = const_scalar_from_int!(FloatType, 15);
/// A maintenance crew parked on the road for the night: where each person stands, in m.
pub const CREW_M: [(FloatType, FloatType); 4] = [
    (
        const_scalar_from_int!(FloatType, 124),
        const_scalar_from_int!(FloatType, 302),
    ),
    (
        const_scalar_from_int!(FloatType, 127),
        const_scalar_from_int!(FloatType, 309),
    ),
    (
        const_scalar_from_int!(FloatType, 131),
        const_scalar_from_int!(FloatType, 315),
    ),
    (
        const_scalar_from_int!(FloatType, 126),
        const_scalar_from_int!(FloatType, 324),
    ),
];

// =============================================================================
// The drone and its inspection mission
// =============================================================================

/// The inspection line runs along the valley this far across the slope, in m; the drone starts at
/// its beginning and flies along it.
pub const LINE_ACROSS_M: FloatType = const_scalar_from_int!(FloatType, 100);
pub const MISSION_START_ALONG_M: FloatType = const_scalar_from_int!(FloatType, 0);
/// Inspection height above the ground and ground speed along the line.
pub const INSPECTION_AGL_M: FloatType = const_scalar_from_int!(FloatType, 40);
pub const INSPECTION_SPEED_M_S: FloatType = const_scalar_from_int!(FloatType, 8);
/// Descent rates: an ordinary landing and an emergency landing on a failing battery, in m/s.
pub const LANDING_DESCENT_M_S: FloatType = const_scalar_from_float!(FloatType, 1.5);
pub const EMERGENCY_DESCENT_M_S: FloatType = const_scalar_from_int!(FloatType, 3);
/// The night-time wind drains down the slope toward the creek (a katabatic wind), in m/s across.
/// Without satellite positioning the drone cannot hold position against it and drifts with it.
pub const DOWNSLOPE_WIND_M_S: FloatType = const_scalar_from_int!(FloatType, -2);
/// A multirotor on skids tips over on ground steeper than this, in degrees.
pub const TIP_OVER_DEG: FloatType = const_scalar_from_int!(FloatType, 15);
/// A tipped drone tumbles down the slope until the ground flattens below this, in degrees.
pub const ROLL_STOP_DEG: FloatType = const_scalar_from_int!(FloatType, 5);
/// The distance reported when no person is on the ground at all, in m.
pub const NO_ONE_NEARBY_M: FloatType = const_scalar_from_int!(FloatType, 1_000_000);
/// Touching down closer than this to a person is a hazard to them, in m.
pub const PERSON_CLEARANCE_M: FloatType = const_scalar_from_int!(FloatType, 10);

// =============================================================================
// Telemetry and the fault timeline, in seconds of flight
// =============================================================================

/// Satellite positioning degrades near the tower and the ridge, then drops out.
pub const GNSS_DEGRADED_AT_S: usize = 50;
pub const GNSS_LOST_AT_S: usize = 55;
/// The command link drops behind the ridge.
pub const LINK_LOST_AT_S: usize = 65;
/// A cell fails and the pack voltage sags.
pub const BATTERY_FAULT_AT_S: usize = 85;
/// The flight ends this long after launch if the drone is still airborne.
pub const FLIGHT_LIMIT_S: usize = 600;

/// Telemetry values in each phase: satellites in view and horizontal dilution of precision.
pub const SATELLITES_NOMINAL: FloatType = const_scalar_from_int!(FloatType, 14);
pub const SATELLITES_DEGRADED: FloatType = const_scalar_from_int!(FloatType, 7);
pub const SATELLITES_LOST: FloatType = const_scalar_from_int!(FloatType, 3);
pub const HDOP_NOMINAL: FloatType = const_scalar_from_float!(FloatType, 0.8);
pub const HDOP_DEGRADED: FloatType = const_scalar_from_float!(FloatType, 3.5);
pub const HDOP_LOST: FloatType = const_scalar_from_int!(FloatType, 99);
/// Command-link packet loss, in percent.
pub const LINK_LOSS_NOMINAL_PCT: FloatType = const_scalar_from_int!(FloatType, 2);
pub const LINK_LOSS_LOST_PCT: FloatType = const_scalar_from_int!(FloatType, 95);
/// Lowest cell voltage: healthy at launch, falling slowly with use, and sagging after the fault.
pub const CELL_V_LAUNCH: FloatType = const_scalar_from_float!(FloatType, 4.05);
pub const CELL_V_DROP_PER_S: FloatType = const_scalar_from_float!(FloatType, 0.002);
pub const CELL_V_FAULT: FloatType = const_scalar_from_float!(FloatType, 3.25);

// =============================================================================
// Units and small numbers
// =============================================================================

pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const HALF: FloatType = const_scalar_from_float!(FloatType, 0.5);
pub const DEGREES_PER_HALF_TURN: FloatType = const_scalar_from_int!(FloatType, 180);
