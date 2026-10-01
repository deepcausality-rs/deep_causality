/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The simulated flight: atmosphere, sunlight, the gondola's heat balance and the actuator
//! registers the controller writes. The controller sees only [`Reading`].

use crate::FloatType;
use crate::constants::*;
use deep_causality_algebra::Real;
use deep_causality_num::{lift, lift_usize};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

// =============================================================================
// Actuator registers
// =============================================================================

/// Charge limits a register can hold, in rising order.
pub const STOP: u8 = 0;
pub const HALF: u8 = 1;
pub const FULL: u8 = 2;

pub static FAN: AtomicBool = AtomicBool::new(false);
pub static RADIATOR: AtomicBool = AtomicBool::new(false);
pub static HEATER: AtomicBool = AtomicBool::new(false);
/// Charge limit set by the cell's temperature window.
pub static CELL_LIMIT: AtomicU8 = AtomicU8::new(FULL);
/// Charge limit set by the heat budget.
pub static BUDGET_LIMIT: AtomicU8 = AtomicU8::new(FULL);

/// A snapshot of every actuator register.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Actuators {
    pub fan: bool,
    pub radiator: bool,
    pub heater: bool,
    pub cell_limit: u8,
    pub budget_limit: u8,
}

impl Actuators {
    pub fn read() -> Self {
        Self {
            fan: FAN.load(Ordering::Relaxed),
            radiator: RADIATOR.load(Ordering::Relaxed),
            heater: HEATER.load(Ordering::Relaxed),
            cell_limit: CELL_LIMIT.load(Ordering::Relaxed),
            budget_limit: BUDGET_LIMIT.load(Ordering::Relaxed),
        }
    }

    /// The charger obeys the stricter of its two limits.
    pub fn charge_limit(&self) -> u8 {
        self.cell_limit.min(self.budget_limit)
    }

    /// The registers as they act on the pack: below the charging threshold the panels deliver
    /// nothing, so both charge limits read stop.
    pub fn in_light(self, sun_w_m2: FloatType) -> Self {
        if sun_w_m2 >= MIN_CHARGING_SUN_W_M2 {
            return self;
        }
        Self {
            cell_limit: STOP,
            budget_limit: STOP,
            ..self
        }
    }
}

// =============================================================================
// What the controller measures
// =============================================================================

/// One minute of sensor data.
#[derive(Debug, Clone, Copy, Default)]
pub struct Reading {
    pub pressure_kpa: FloatType,
    pub air_c: FloatType,
    pub cell_c: FloatType,
    pub sun_w_m2: FloatType,
}

// =============================================================================
// Heat transfer, shared by the simulation and the controller's rules
// =============================================================================

/// Air density from pressure and temperature (ideal gas), in kg/m³.
pub fn air_density(pressure_kpa: FloatType, air_c: FloatType) -> FloatType {
    pressure_kpa * lift::<FloatType>(1000.0) / (AIR_GAS_CONSTANT * (air_c + ZERO_C_IN_K))
}

/// Heat the fan removes from a cell at `cell_c`, in W.
pub fn fan_heat_w(density: FloatType, cell_c: FloatType, air_c: FloatType) -> FloatType {
    FAN_W_K * Real::powf(density / SEA_LEVEL_DENSITY, FAN_DENSITY_EXPONENT) * (cell_c - air_c)
}

/// Heat the radiator panel radiates from a cell at `cell_c` to surroundings at `air_c`, in W.
pub fn radiator_heat_w(cell_c: FloatType, air_c: FloatType) -> FloatType {
    let cell_k = cell_c + ZERO_C_IN_K;
    let air_k = air_c + ZERO_C_IN_K;
    RADIATOR_AREA_M2
        * STEFAN_BOLTZMANN
        * (cell_k * cell_k * cell_k * cell_k - air_k * air_k * air_k * air_k)
}

/// Conductance from the cell to the outside air through the insulation, in W/K.
pub fn passive_w_k(density: FloatType) -> FloatType {
    let outer = OUTER_CONVECTION_W_K * Real::sqrt(density / SEA_LEVEL_DENSITY);
    let one: FloatType = lift::<FloatType>(1.0);
    one / (one / INSULATION_W_K + one / outer)
}

/// Heat entering the pack at `charge_a`, in W: electronics, absorbed sunlight and charging loss.
pub fn heat_in_w(sun_w_m2: FloatType, charge_a: FloatType) -> FloatType {
    ELECTRONICS_W + SOLAR_ABSORBING_AREA_M2 * sun_w_m2 + charge_a * charge_a * PACK_OHM
}

// =============================================================================
// The flight
// =============================================================================

/// The simulated balloon: where it is, and the state of its pack.
pub struct Flight {
    minute: usize,
    cell_c: FloatType,
    charge_wh: FloatType,
    lowest_charge_wh: FloatType,
    coldest_cell_c: FloatType,
    warmest_cell_c: FloatType,
}

impl Flight {
    pub fn launch() -> Self {
        Self {
            minute: 0,
            cell_c: LAUNCH_CELL_C,
            charge_wh: LAUNCH_CHARGE * PACK_WH,
            lowest_charge_wh: LAUNCH_CHARGE * PACK_WH,
            coldest_cell_c: LAUNCH_CELL_C,
            warmest_cell_c: LAUNCH_CELL_C,
        }
    }

    /// True once the descent has reached the ground.
    pub fn landed(&self) -> bool {
        self.minute > DESCENT_START_MIN && self.altitude_m() <= lift::<FloatType>(0.0)
    }

    /// Climb at the vertical speed, float, then descend from the planned start.
    pub fn altitude_m(&self) -> FloatType {
        let rate_per_min = VERTICAL_SPEED_M_S * lift::<FloatType>(60.0);
        let climbed = rate_per_min * lift_usize::<FloatType>(self.minute);
        if self.minute < DESCENT_START_MIN {
            return if climbed < FLOAT_ALTITUDE_M {
                climbed
            } else {
                FLOAT_ALTITUDE_M
            };
        }
        let descended = rate_per_min * lift_usize::<FloatType>(self.minute - DESCENT_START_MIN);
        let altitude = FLOAT_ALTITUDE_M - descended;
        if altitude > lift::<FloatType>(0.0) {
            altitude
        } else {
            lift::<FloatType>(0.0)
        }
    }

    /// Local solar time, in hours since midnight of launch day.
    pub fn local_hour(&self) -> FloatType {
        LAUNCH_HOUR + lift_usize::<FloatType>(self.minute) / lift::<FloatType>(60.0)
    }

    pub fn charge_fraction(&self) -> FloatType {
        self.charge_wh / PACK_WH
    }

    pub fn lowest_charge_fraction(&self) -> FloatType {
        self.lowest_charge_wh / PACK_WH
    }

    pub fn cell_range_c(&self) -> (FloatType, FloatType) {
        (self.coldest_cell_c, self.warmest_cell_c)
    }

    fn in_warm_air(&self) -> bool {
        (WARM_AIR_START_MIN..WARM_AIR_END_MIN).contains(&self.minute)
            && self.altitude_m() >= FLOAT_ALTITUDE_M
    }

    /// Standard-atmosphere pressure and the launch-day temperature profile.
    fn air(&self) -> (FloatType, FloatType) {
        let h = self.altitude_m();
        let tropopause_m: FloatType = lift::<FloatType>(11000.0);
        let pressure = if h <= tropopause_m {
            SEA_LEVEL_KPA
                * Real::powf(
                    lift::<FloatType>(1.0) - lift::<FloatType>(2.25577e-5) * h,
                    lift::<FloatType>(5.25588),
                )
        } else {
            lift::<FloatType>(22.632) * Real::exp(-(h - tropopause_m) / lift::<FloatType>(6341.6))
        };
        let lapse = GROUND_AIR_C - LAPSE_RATE_K_M * h;
        let mut air = if lapse > STRATOSPHERE_C {
            lapse
        } else {
            STRATOSPHERE_C
        };
        if self.in_warm_air() {
            air += WARM_AIR_EXCESS_K;
        }
        (pressure, air)
    }

    /// Sunlight on the gondola: the sun's elevation, the air above, and the cloud deck below.
    fn sun(&self, pressure_kpa: FloatType) -> FloatType {
        let hour = Real::floor(self.local_hour() / lift::<FloatType>(24.0));
        let hour = self.local_hour() - hour * lift::<FloatType>(24.0);
        if hour <= lift::<FloatType>(6.0) || hour >= lift::<FloatType>(18.0) {
            return lift::<FloatType>(0.0);
        }
        let elevation = Real::sin(
            <FloatType as Real>::pi() * (hour - lift::<FloatType>(6.0)) / lift::<FloatType>(12.0),
        );
        let transmitted = Real::exp(-OPTICAL_DEPTH * pressure_kpa / SEA_LEVEL_KPA);
        let reflected = if self.in_warm_air() {
            CLOUD_DECK_GAIN
        } else {
            lift::<FloatType>(1.0)
        };
        SOLAR_CONSTANT_W_M2 * transmitted * elevation * reflected
    }

    /// What the sensors report this minute.
    pub fn reading(&self) -> Reading {
        let (pressure_kpa, air_c) = self.air();
        Reading {
            pressure_kpa,
            air_c,
            cell_c: self.cell_c,
            sun_w_m2: self.sun(pressure_kpa),
        }
    }

    /// Advances one minute under the current actuator registers.
    pub fn step(&mut self) {
        let r = self.reading();
        let act = Actuators::read().in_light(r.sun_w_m2);
        let density = air_density(r.pressure_kpa, r.air_c);

        let charge_a = match act.charge_limit() {
            FULL => FULL_CHARGE_A,
            HALF => HALF_CHARGE_A,
            _ => lift::<FloatType>(0.0),
        };
        let heater_w = if act.heater {
            HEATER_W
        } else {
            lift::<FloatType>(0.0)
        };

        let mut heat_w = heat_in_w(r.sun_w_m2, charge_a) + heater_w
            - passive_w_k(density) * (self.cell_c - r.air_c);
        if act.fan {
            heat_w -= fan_heat_w(density, self.cell_c, r.air_c);
        }
        if act.radiator {
            heat_w -= radiator_heat_w(self.cell_c, r.air_c);
        }

        let seconds: FloatType = lift_usize::<FloatType>(STEP_MIN) * lift::<FloatType>(60.0);
        let hours = seconds / lift::<FloatType>(3600.0);
        self.cell_c += heat_w * seconds / HEAT_CAPACITY_J_K;

        let drawn_w = ELECTRONICS_W + heater_w;
        let charge = self.charge_wh + (PACK_V * charge_a - drawn_w) * hours;
        self.charge_wh = Real::clamp(charge, lift::<FloatType>(0.0), PACK_WH);

        if self.charge_wh < self.lowest_charge_wh {
            self.lowest_charge_wh = self.charge_wh;
        }
        if self.cell_c < self.coldest_cell_c {
            self.coldest_cell_c = self.cell_c;
        }
        if self.cell_c > self.warmest_cell_c {
            self.warmest_cell_c = self.cell_c;
        }
        self.minute += STEP_MIN;
    }
}
