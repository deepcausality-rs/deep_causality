/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::PhysicsError;

/// Absolute Temperature (Kelvin).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Temperature(f64);

const ZERO_CELSIUS_IN_KELVIN: f64 = 273.15;

impl Temperature {
    /// Creates a new `Temperature` instance from Kelvin.
    ///
    /// # Errors
    /// Returns `PhysicsError::ZeroKelvinViolation` if `val < 0.0`.
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::ZeroKelvinViolation());
        }
        Ok(Self(val))
    }

    /// Creates a new `Temperature` instance from Celsius.
    ///
    /// # Errors
    /// Returns `PhysicsError::ZeroKelvinViolation` if `val < -273.15`.
    pub fn from_celsius(celsius: f64) -> Result<Self, PhysicsError> {
        let kelvin = celsius + ZERO_CELSIUS_IN_KELVIN;
        Self::new(kelvin)
    }

    /// Creates a new `Temperature` instance from Fahrenheit.
    ///
    /// # Errors
    /// Returns `PhysicsError::ZeroKelvinViolation` if `val < -459.67`.
    pub fn from_fahrenheit(fahrenheit: f64) -> Result<Self, PhysicsError> {
        let celsius = (fahrenheit - 32.0) * (5.0 / 9.0);
        Self::from_celsius(celsius)
    }

    /// Creates a new `Temperature` instance without validation.
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }

    /// Returns the temperature in Kelvin.
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Returns the temperature in Celsius.
    pub fn as_celsius(&self) -> f64 {
        self.0 - ZERO_CELSIUS_IN_KELVIN
    }

    /// Returns the temperature in Fahrenheit.
    pub fn as_fahrenheit(&self) -> f64 {
        (self.as_celsius() * (9.0 / 5.0)) + 32.0
    }
}

impl From<Temperature> for f64 {
    fn from(val: Temperature) -> Self {
        val.0
    }
}
