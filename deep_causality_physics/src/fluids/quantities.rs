/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{PhysicsError, PhysicsErrorEnum};

/// Pressure (Pascals).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Pressure(f64);

impl Pressure {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Negative Pressure".into()),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<Pressure> for f64 {
    fn from(val: Pressure) -> Self {
        val.0
    }
}

/// Density (kg/m^3).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Density(f64);

impl Density {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Negative Density".into()),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<Density> for f64 {
    fn from(val: Density) -> Self {
        val.0
    }
}

/// Dynamic Viscosity (Pa·s).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Viscosity(f64);

impl Viscosity {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Negative Viscosity".into()),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<Viscosity> for f64 {
    fn from(val: Viscosity) -> Self {
        val.0
    }
}
