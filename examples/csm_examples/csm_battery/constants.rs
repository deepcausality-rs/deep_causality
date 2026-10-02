/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The cell, the coolant, the charge session, and the two charge laws the controller can run.

use crate::FloatType;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int};

/// The cell's capacity, in ampere-hours, and the ambient temperature, in °C.
pub const CELL_CAPACITY_AH: FloatType = const_scalar_from_int!(FloatType, 50);
pub const AMBIENT_C: FloatType = const_scalar_from_int!(FloatType, 25);

/// Each minute the current heats the cell by `HEATING · I²` and the cell sheds
/// `cooling · (T − ambient)`: the pumped coolant's rate while the pump runs, the passive rate after.
pub const HEATING_C_PER_A2: FloatType = const_scalar_from_float!(FloatType, 0.00048);
pub const COOLING_PUMPED: FloatType = const_scalar_from_float!(FloatType, 0.08);
pub const COOLING_PASSIVE: FloatType = const_scalar_from_float!(FloatType, 0.01);

/// The coolant pump stops after this many minutes of charging.
pub const PUMP_FAILS_AFTER_MIN: usize = 10;

/// The session charges the cell from 20 % to 80 %, and gives up after two hours.
pub const START_CHARGE_PCT: FloatType = const_scalar_from_int!(FloatType, 20);
pub const TARGET_CHARGE_PCT: FloatType = const_scalar_from_int!(FloatType, 80);
pub const MAX_MINUTES: usize = 120;

/// Charging above this temperature ages a lithium-ion cell faster; the summary counts those
/// minutes.
pub const AGEING_ABOVE_C: FloatType = const_scalar_from_int!(FloatType, 45);

/// The charge law for a cooled cell: full current below 45 °C, half up to 55 °C, none above.
pub const COOLED_FULL_BELOW_C: FloatType = const_scalar_from_int!(FloatType, 45);
pub const COOLED_HALF_BELOW_C: FloatType = const_scalar_from_int!(FloatType, 55);
pub const FULL_A: u8 = 50;
pub const HALF_A: u8 = 25;

/// The derated law for a cell without its pump: 20 A below 40 °C, 10 A up to 45 °C, none above.
/// Without the pump, 20 A holds the cell near 44 °C at equilibrium.
pub const DERATED_FULL_BELOW_C: FloatType = const_scalar_from_int!(FloatType, 40);
pub const DERATED_HALF_BELOW_C: FloatType = const_scalar_from_int!(FloatType, 45);
pub const DERATED_A: u8 = 20;
pub const TRICKLE_A: u8 = 10;

pub const OFF_A: u8 = 0;

/// The cooling-loss detector fires when the cell rose this much more, in K, than the pumped
/// coolant allows for the current that flowed.
pub const COOLING_LOSS_MARGIN_C: FloatType = const_scalar_from_float!(FloatType, 0.3);

pub const MINUTES_PER_HOUR: FloatType = const_scalar_from_int!(FloatType, 60);
pub const HUNDRED: FloatType = const_scalar_from_int!(FloatType, 100);
