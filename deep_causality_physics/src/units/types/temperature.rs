/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::PhysicsError;
use deep_causality_num::{FromPrimitive, RealField};

/// Absolute Temperature (Kelvin).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Temperature<R: RealField>(R);

const ZERO_CELSIUS_IN_KELVIN: f64 = 273.15;

impl<R: RealField> Default for Temperature<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Temperature<R> {
    /// Creates a new `Temperature` instance from Kelvin.
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if val < R::zero() {
            return Err(PhysicsError::ZeroKelvinViolation());
        }
        Ok(Self(val))
    }

    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }

    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: RealField + FromPrimitive> Temperature<R> {
    /// Creates a new `Temperature` instance from Celsius.
    pub fn from_celsius(celsius: R) -> Result<Self, PhysicsError> {
        let k = R::from_f64(ZERO_CELSIUS_IN_KELVIN).ok_or_else(|| {
            PhysicsError::NumericalInstability("R::from_f64(ZERO_CELSIUS_IN_KELVIN) failed".into())
        })?;
        Self::new(celsius + k)
    }

    /// Creates a new `Temperature` instance from Fahrenheit.
    pub fn from_fahrenheit(fahrenheit: R) -> Result<Self, PhysicsError> {
        let thirty_two = R::from_f64(32.0)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(32) failed".into()))?;
        let five = R::from_f64(5.0)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(5) failed".into()))?;
        let nine = R::from_f64(9.0)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(9) failed".into()))?;
        let celsius = (fahrenheit - thirty_two) * (five / nine);
        Self::from_celsius(celsius)
    }

    /// Returns the temperature in Celsius.
    pub fn as_celsius(&self) -> R {
        let k = R::from_f64(ZERO_CELSIUS_IN_KELVIN)
            .expect("R::from_f64(ZERO_CELSIUS_IN_KELVIN) failed");
        self.0 - k
    }

    /// Returns the temperature in Fahrenheit.
    pub fn as_fahrenheit(&self) -> R {
        let nine = R::from_f64(9.0).expect("R::from_f64(9) failed");
        let five = R::from_f64(5.0).expect("R::from_f64(5) failed");
        let thirty_two = R::from_f64(32.0).expect("R::from_f64(32) failed");
        (self.as_celsius() * (nine / five)) + thirty_two
    }
}

impl<R: RealField + Into<f64>> From<Temperature<R>> for f64 {
    fn from(val: Temperature<R>) -> Self {
        val.0.into()
    }
}
