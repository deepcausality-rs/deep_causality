/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The fault detectors' thresholds and how long a fault must persist before the controller acts.

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
