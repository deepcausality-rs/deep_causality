/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The campaign: how many scenarios, their seed, and the operational design domain each scenario
//! is drawn from. Every range is uniform.

use deep_causality_num::{const_scalar_from_float, const_scalar_from_int};
use dynamic_drone_failsafe::FloatType;

/// Scenarios in a campaign unless the command line names another count, and the campaign seed.
pub const SCENARIOS: usize = 1000;
pub const CAMPAIGN_SEED: u64 = 2026;

/// Local hour at launch: any time of day or night.
pub const START_HOUR: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, 0),
    const_scalar_from_int!(FloatType, 24),
);
/// Wind speed, in m/s, from any direction, in degrees.
pub const WIND_M_S: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, 0),
    const_scalar_from_int!(FloatType, 6),
);
pub const DIRECTION_DEG: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, 0),
    const_scalar_from_int!(FloatType, 360),
);
/// How far a gust moves the drone in its last second before touchdown, in m, in any direction.
pub const GUST_M: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, 0),
    const_scalar_from_int!(FloatType, 2),
);
/// The line's distance across the slope, in m: anywhere on the grass slope.
pub const LINE_ACROSS_M: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, 24),
    const_scalar_from_int!(FloatType, 108),
);
/// When the fix drops out, in s after launch; it degrades this long before.
pub const FIX_LOST_AT_S: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, 20),
    const_scalar_from_int!(FloatType, 90),
);
pub const DEGRADED_LEAD_S: usize = 5;
/// When the link drops, in s relative to the fix: before it, with it or after it.
pub const LINK_OFFSET_S: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, -20),
    const_scalar_from_int!(FloatType, 30),
);
/// A cell fails in this share of scenarios, and the fix and the link each come back in this share,
/// after this long, in s.
pub const CELL_FAILURE_SHARE: FloatType = const_scalar_from_float!(FloatType, 0.5);
pub const RECOVERY_SHARE: FloatType = const_scalar_from_float!(FloatType, 0.333);
pub const RECOVERY_AFTER_S: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, 10),
    const_scalar_from_int!(FloatType, 60),
);
/// When a cell fails, in s after the later of the two losses.
pub const BATTERY_OFFSET_S: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, 0),
    const_scalar_from_int!(FloatType, 60),
);
/// The terrace: where it starts across and along, in m, and its width and length.
pub const TERRACE_FROM_ACROSS_M: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, 14),
    const_scalar_from_int!(FloatType, 100),
);
pub const TERRACE_FROM_ALONG_M: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, 100),
    const_scalar_from_int!(FloatType, 700),
);
pub const TERRACE_SIZE_M: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, 12),
    const_scalar_from_int!(FloatType, 50),
);
/// A stand of trees near where the fix drops out: where it starts across, how far before that point
/// along the line it starts, and its width and length, in m.
pub const WOODLAND_FROM_ACROSS_M: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, 14),
    const_scalar_from_int!(FloatType, 100),
);
pub const WOODLAND_BEFORE_M: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, -40),
    const_scalar_from_int!(FloatType, 120),
);
pub const WOODLAND_WIDTH_M: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, 20),
    const_scalar_from_int!(FloatType, 40),
);
pub const WOODLAND_LENGTH_M: (FloatType, FloatType) = (
    const_scalar_from_int!(FloatType, 40),
    const_scalar_from_int!(FloatType, 100),
);
/// People on the ground: up to this many, each within this distance across and along of the point
/// where the fix drops out, and never in the creek.
pub const MAX_CREW: usize = 8;
pub const CREW_SPREAD_ACROSS_M: FloatType = const_scalar_from_int!(FloatType, 40);
pub const CREW_SPREAD_ALONG_M: FloatType = const_scalar_from_int!(FloatType, 60);
pub const CREEK_EDGE_M: FloatType = const_scalar_from_int!(FloatType, 12);
pub const ROAD_TO_M: FloatType = const_scalar_from_int!(FloatType, 135);
/// The drone flies the line at this speed until the fix drops out, in m/s.
pub const INSPECTION_SPEED_M_S: FloatType = const_scalar_from_int!(FloatType, 8);

/// The confidence of each upper bound on a rate.
pub const CONFIDENCE: FloatType = const_scalar_from_float!(FloatType, 0.95);
/// The non-safe outcomes of the full stack the report lists by scenario, at most.
pub const MAX_LISTED: usize = 20;

/// The distance recorded to the nearest person when the flight ended in the air.
pub const NO_ONE_NEARBY_M: FloatType = const_scalar_from_int!(FloatType, 1_000_000);
pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
pub const HUNDRED: FloatType = const_scalar_from_int!(FloatType, 100);
pub const DEGREES_PER_HALF_TURN: FloatType = const_scalar_from_int!(FloatType, 180);
