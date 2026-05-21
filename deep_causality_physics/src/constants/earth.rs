/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::NEWTONIAN_CONSTANT_OF_GRAVITATION;

/// Gravity Mass Constant
pub const EARTH_GM: f64 = 3.986_004_418e14; // Gravitational Parameter (IERS 2010)

/// Gravitational acceleration
pub const EARTH_GRAVITY_ACCELERATION: f64 = 9.80665; // m s^-2 (exact)

// Derived from EARTH_GM / NEWTONIAN_CONSTANT_OF_GRAVITATION so the mass is
// self-consistent with the G used elsewhere in the crate. Recovering GM from
// observation and dividing by the same G must round-trip to this constant
// without a constants-mismatch bias (see chronometric_examples gm_recovery).
pub const EARTH_MASS_KG: f64 = EARTH_GM / NEWTONIAN_CONSTANT_OF_GRAVITATION;
pub const EARTH_RADIUS: f64 = 6_371_000.0; // Earth's mean radius in meters.
pub const EARTH_ROTATION_RATE: f64 = 7.292_115_146_706_979e-5; // IERS 2010 (rad/s)
pub const EARTH_ANGULAR_MOMENTUM: f64 = 5.86e33;

/// Earth's J2 Oblateness coefficient (JGM-3)
/// <https://www.sciencedirect.com/topics/engineering/oblateness>
pub const EARTH_J2: f64 = 1.082_63e-3;

/// Earth's Equatorial Radius (WGS-84)
pub const EARTH_RADIUS_EQUATORIAL: f64 = 6_378_137.0;
