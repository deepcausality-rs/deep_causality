/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use deep_causality_num::{FromPrimitive, RealField};

/// Energy (Joules or eV).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Energy<R: RealField>(R);

const JOULES_PER_EV: f64 = 1.602_176_634e-19;
const JOULES_PER_CALORIE: f64 = 4.184;
const JOULES_PER_KWH: f64 = 3.6e6;

impl<R: RealField> Default for Energy<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Energy<R> {
    /// Creates a new `Energy` instance (Joules).
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        // Energy can be negative (e.g., potential energy wells), so no strict check unless specified.
        Ok(Self(val))
    }

    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }

    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: RealField + FromPrimitive> Energy<R> {
    /// Electron Volts.
    pub fn from_electron_volts(ev: R) -> Result<Self, PhysicsError> {
        let k = R::from_f64(JOULES_PER_EV).ok_or_else(|| {
            PhysicsError::NumericalInstability("R::from_f64(JOULES_PER_EV) failed".into())
        })?;
        Self::new(ev * k)
    }

    /// Calories (thermochemical).
    pub fn from_calories(cal: R) -> Result<Self, PhysicsError> {
        let k = R::from_f64(JOULES_PER_CALORIE).ok_or_else(|| {
            PhysicsError::NumericalInstability("R::from_f64(JOULES_PER_CALORIE) failed".into())
        })?;
        Self::new(cal * k)
    }

    /// Kilowatt-hours.
    pub fn from_kilowatt_hours(kwh: R) -> Result<Self, PhysicsError> {
        let k = R::from_f64(JOULES_PER_KWH).ok_or_else(|| {
            PhysicsError::NumericalInstability("R::from_f64(JOULES_PER_KWH) failed".into())
        })?;
        Self::new(kwh * k)
    }

    /// As Electron Volts.
    pub fn as_electron_volts(&self) -> R {
        let k = R::from_f64(JOULES_PER_EV).expect("R::from_f64(JOULES_PER_EV) failed");
        self.0 / k
    }

    /// As Calories.
    pub fn as_calories(&self) -> R {
        let k = R::from_f64(JOULES_PER_CALORIE).expect("R::from_f64(JOULES_PER_CALORIE) failed");
        self.0 / k
    }

    /// As Kilowatt-hours.
    pub fn as_kilowatt_hours(&self) -> R {
        let k = R::from_f64(JOULES_PER_KWH).expect("R::from_f64(JOULES_PER_KWH) failed");
        self.0 / k
    }
}

impl<R: RealField + Into<f64>> From<Energy<R>> for f64 {
    fn from(val: Energy<R>) -> Self {
        val.0.into()
    }
}
