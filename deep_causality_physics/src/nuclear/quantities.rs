/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::PhysicsError;

/// Amount of Substance (Moles).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct AmountOfSubstance(f64);

impl AmountOfSubstance {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative AmountOfSubstance".into(),
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
impl From<AmountOfSubstance> for f64 {
    fn from(val: AmountOfSubstance) -> Self {
        val.0
    }
}

/// Half-Life (Seconds).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct HalfLife(f64);

impl HalfLife {
    /// Creates a new `HalfLife` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError` if `val <= 0.0`.
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "HalfLife must be positive (zero implies infinite decay rate)".into(),
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
impl From<HalfLife> for f64 {
    fn from(val: HalfLife) -> Self {
        val.0
    }
}

/// Radioactivity (Becquerels).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Activity(f64);

impl Activity {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative Activity".into(),
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
impl From<Activity> for f64 {
    fn from(val: Activity) -> Self {
        val.0
    }
}

/// Energy Density (Joules per cubic meter).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct EnergyDensity(f64);

impl EnergyDensity {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative EnergyDensity".into(),
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
impl From<EnergyDensity> for f64 {
    fn from(val: EnergyDensity) -> Self {
        val.0
    }
}
