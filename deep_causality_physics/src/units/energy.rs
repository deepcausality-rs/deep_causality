/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;

/// Energy (Joules or eV).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Energy(f64);

const JOULES_PER_EV: f64 = 1.602_176_634e-19;
const JOULES_PER_CALORIE: f64 = 4.184;
const JOULES_PER_KWH: f64 = 3.6e6;

impl Energy {
    /// Creates a new `Energy` instance (Joules).
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        // Energy can be negative (e.g., potential energy wells), so no strict check unless specified.
        Ok(Self(val))
    }

    /// Electron Volts.
    pub fn from_electron_volts(ev: f64) -> Result<Self, PhysicsError> {
        Self::new(ev * JOULES_PER_EV)
    }

    /// Calories (thermochemical).
    pub fn from_calories(cal: f64) -> Result<Self, PhysicsError> {
        Self::new(cal * JOULES_PER_CALORIE)
    }

    /// Kilowatt-hours.
    pub fn from_kilowatt_hours(kwh: f64) -> Result<Self, PhysicsError> {
        Self::new(kwh * JOULES_PER_KWH)
    }

    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    /// As Electron Volts.
    pub fn as_electron_volts(&self) -> f64 {
        self.0 / JOULES_PER_EV
    }

    /// As Calories.
    pub fn as_calories(&self) -> f64 {
        self.0 / JOULES_PER_CALORIE
    }

    /// As Kilowatt-hours.
    pub fn as_kilowatt_hours(&self) -> f64 {
        self.0 / JOULES_PER_KWH
    }
}

impl From<Energy> for f64 {
    fn from(val: Energy) -> Self {
        val.0
    }
}
