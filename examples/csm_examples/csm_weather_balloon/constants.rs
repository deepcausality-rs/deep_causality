/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Flight plan, gondola and controller constants for the weather balloon example.

use crate::FloatType;
use deep_causality_num::const_scalar_from_float;

// =============================================================================
// Flight plan
// =============================================================================

/// Simulation step and controller period, in minutes.
pub const STEP_MIN: usize = 1;
/// Local solar time at launch, in hours.
pub const LAUNCH_HOUR: FloatType = const_scalar_from_float!(FloatType, 10.0);
/// Ascent and descent rate of a typical sounding balloon, in m/s.
pub const VERTICAL_SPEED_M_S: FloatType = const_scalar_from_float!(FloatType, 5.0);
/// Float altitude of the expedition, in metres.
pub const FLOAT_ALTITUDE_M: FloatType = const_scalar_from_float!(FloatType, 20000.0);
/// Planned start of the descent: 07:00 local time on the next morning, in minutes after launch.
pub const DESCENT_START_MIN: usize = 21 * 60;

// =============================================================================
// The simulated atmosphere (the controller never reads these)
// =============================================================================

/// Air temperature at the launch site on a summer morning, in °C.
pub const GROUND_AIR_C: FloatType = const_scalar_from_float!(FloatType, 32.0);
/// Standard tropospheric lapse rate, in K/m.
pub const LAPSE_RATE_K_M: FloatType = const_scalar_from_float!(FloatType, 0.0065);
/// Standard-atmosphere temperature from the tropopause to 20 km, in °C.
pub const STRATOSPHERE_C: FloatType = const_scalar_from_float!(FloatType, -56.5);
/// Sea-level pressure of the standard atmosphere, in kPa.
pub const SEA_LEVEL_KPA: FloatType = const_scalar_from_float!(FloatType, 101.325);
/// Top-of-atmosphere solar irradiance, in W/m².
pub const SOLAR_CONSTANT_W_M2: FloatType = const_scalar_from_float!(FloatType, 1361.0);
/// Clear-sky optical depth at sea level: 75 % of sunlight reaches the ground.
pub const OPTICAL_DEPTH: FloatType = const_scalar_from_float!(FloatType, 0.288);
/// Start and end of the warm air mass the balloon drifts into at float, in minutes after launch
/// (13:30 to 16:00 local time).
pub const WARM_AIR_START_MIN: usize = 210;
pub const WARM_AIR_END_MIN: usize = 360;
/// How much warmer than the standard atmosphere that air mass is, in K.
pub const WARM_AIR_EXCESS_K: FloatType = const_scalar_from_float!(FloatType, 25.0);
/// A bright cloud deck below reflects sunlight onto the gondola while the air mass passes.
pub const CLOUD_DECK_GAIN: FloatType = const_scalar_from_float!(FloatType, 1.5);

// =============================================================================
// Gondola and battery pack
// =============================================================================

/// Heat capacity of the pack and the electronics around it, in J/K.
pub const HEAT_CAPACITY_J_K: FloatType = const_scalar_from_float!(FloatType, 2500.0);
/// Heat the sensors and the radio dissipate inside the gondola, in W. They also draw this from
/// the pack.
pub const ELECTRONICS_W: FloatType = const_scalar_from_float!(FloatType, 3.0);
/// Heater power, in W.
pub const HEATER_W: FloatType = const_scalar_from_float!(FloatType, 8.0);
/// Absorptance times exposed area of the gondola, in m².
pub const SOLAR_ABSORBING_AREA_M2: FloatType = const_scalar_from_float!(FloatType, 0.012);
/// Conductance of the foam insulation, in W/K.
pub const INSULATION_W_K: FloatType = const_scalar_from_float!(FloatType, 0.15);
/// Conductance of natural convection outside the gondola at sea-level density, in W/K.
pub const OUTER_CONVECTION_W_K: FloatType = const_scalar_from_float!(FloatType, 3.0);
/// Conductance of the fan at sea-level density, in W/K. It falls with density to the power 0.8,
/// the Reynolds-number scaling of turbulent forced convection.
pub const FAN_W_K: FloatType = const_scalar_from_float!(FloatType, 1.0);
/// Exponent of that scaling.
pub const FAN_DENSITY_EXPONENT: FloatType = const_scalar_from_float!(FloatType, 0.8);
/// Emissivity times area of the radiator panel, in m².
pub const RADIATOR_AREA_M2: FloatType = const_scalar_from_float!(FloatType, 0.0425);
/// Pack voltage, in V.
pub const PACK_V: FloatType = const_scalar_from_float!(FloatType, 14.4);
/// Pack capacity, in Wh.
pub const PACK_WH: FloatType = const_scalar_from_float!(FloatType, 144.0);
/// Charge at launch, as a fraction of capacity.
pub const LAUNCH_CHARGE: FloatType = const_scalar_from_float!(FloatType, 0.8);
/// Pack internal resistance, in Ω.
pub const PACK_OHM: FloatType = const_scalar_from_float!(FloatType, 0.15);
/// Full and half charge current, in A.
pub const FULL_CHARGE_A: FloatType = const_scalar_from_float!(FloatType, 2.0);
pub const HALF_CHARGE_A: FloatType = const_scalar_from_float!(FloatType, 1.0);
/// Sunlight below which the panels cannot charge, in W/m².
pub const MIN_CHARGING_SUN_W_M2: FloatType = const_scalar_from_float!(FloatType, 150.0);
/// Cell temperature after an hour on the launch pad in the sun, in °C.
pub const LAUNCH_CELL_C: FloatType = const_scalar_from_float!(FloatType, 38.0);

// =============================================================================
// Controller: the envelope and its set points
// =============================================================================

/// Lithium-ion charge window, in °C: no charging outside it.
pub const CHARGE_MIN_C: FloatType = const_scalar_from_float!(FloatType, 0.0);
pub const CHARGE_MAX_C: FloatType = const_scalar_from_float!(FloatType, 45.0);
/// Inside the window, half current below this cell temperature, in °C.
pub const COLD_DERATE_C: FloatType = const_scalar_from_float!(FloatType, 10.0);
/// Inside the window, half current above this cell temperature, in °C.
pub const HOT_DERATE_C: FloatType = const_scalar_from_float!(FloatType, 40.0);
/// The heat budget derates charging when the heat coming in exceeds this share of what the
/// cooling can remove at the top of the charge window.
pub const BUDGET_MARGIN: FloatType = const_scalar_from_float!(FloatType, 0.8);
/// Cooling switches on at this cell temperature and off at the next, in °C.
pub const COOL_ON_C: FloatType = const_scalar_from_float!(FloatType, 35.0);
pub const COOL_OFF_C: FloatType = const_scalar_from_float!(FloatType, 30.0);
/// The heater switches on at this cell temperature and off at the next, in °C.
pub const HEAT_ON_C: FloatType = const_scalar_from_float!(FloatType, 5.0);
pub const HEAT_OFF_C: FloatType = const_scalar_from_float!(FloatType, 15.0);
/// The cooling regime changes when the other mechanism removes this much more heat at the
/// cooling set point. The 10 % band keeps the regime from chattering at the crossover.
pub const REGIME_HYSTERESIS: FloatType = const_scalar_from_float!(FloatType, 1.1);

// =============================================================================
// Physical constants
// =============================================================================

/// Stefan–Boltzmann constant, in W/(m² K⁴).
pub const STEFAN_BOLTZMANN: FloatType = const_scalar_from_float!(FloatType, 5.670374419e-8);
/// Specific gas constant of dry air, in J/(kg K).
pub const AIR_GAS_CONSTANT: FloatType = const_scalar_from_float!(FloatType, 287.05);
/// Air density of the standard atmosphere at sea level, in kg/m³.
pub const SEA_LEVEL_DENSITY: FloatType = const_scalar_from_float!(FloatType, 1.225);
/// Offset from °C to K.
pub const ZERO_C_IN_K: FloatType = const_scalar_from_float!(FloatType, 273.15);
