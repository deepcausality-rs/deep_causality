/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use alloc::format;

/// Mass quantity (kg).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Mass(f64);

impl Mass {
    /// Creates a new `Mass` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError::PhysicalInvariantBroken` if `val` is not finite or `val < 0.0`.
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Mass must be finite: {}",
                val
            )));
        }
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Mass cannot be negative: {}",
                val
            )));
        }
        Ok(Self(val))
    }
    /// Creates a new `Mass` instance without validation.
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<Mass> for f64 {
    fn from(val: Mass) -> Self {
        val.0
    }
}

/// Speed quantity (scalar magnitude of velocity) (m/s).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Speed(f64);

impl Speed {
    /// Creates a new `Speed` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError::PhysicalInvariantBroken` if `val` is not finite or `val < 0.0`.
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Speed must be finite: {}",
                val
            )));
        }
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Speed cannot be negative: {}",
                val
            )));
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
impl From<Speed> for f64 {
    fn from(val: Speed) -> Self {
        val.0
    }
}

/// Linear acceleration (m/s^2).
/// Can be negative to indicate direction in 1D context.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Acceleration(f64);

impl Acceleration {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Acceleration must be finite: {}",
                val
            )));
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
impl From<Acceleration> for f64 {
    fn from(val: Acceleration) -> Self {
        val.0
    }
}

/// Force (N).
/// Can be negative to indicate direction in 1D context.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Force(f64);

impl Force {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Force must be finite: {}",
                val
            )));
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
impl From<Force> for f64 {
    fn from(val: Force) -> Self {
        val.0
    }
}

/// Torque (N·m).
/// Can be negative to indicate direction (e.g. clockwise vs counter-clockwise).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Torque(f64);
impl Torque {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Torque must be finite: {}",
                val
            )));
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
impl From<Torque> for f64 {
    fn from(val: Torque) -> Self {
        val.0
    }
}

/// Length (m).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Length(f64);
impl Length {
    /// Creates a new `Length` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError::PhysicalInvariantBroken` if `val` is not finite or `val < 0.0`.
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Length must be finite: {}",
                val
            )));
        }
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Length cannot be negative: {}",
                val
            )));
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
impl From<Length> for f64 {
    fn from(val: Length) -> Self {
        val.0
    }
}

/// Area (m^2).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Area(f64);
impl Area {
    /// Creates a new `Area` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError::PhysicalInvariantBroken` if `val` is not finite or `val < 0.0`.
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Area must be finite: {}",
                val
            )));
        }
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Area cannot be negative: {}",
                val
            )));
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
impl From<Area> for f64 {
    fn from(val: Area) -> Self {
        val.0
    }
}

/// Volume (m^3).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Volume(f64);
impl Volume {
    /// Creates a new `Volume` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError::PhysicalInvariantBroken` if `val` is not finite or `val < 0.0`.
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Volume must be finite: {}",
                val
            )));
        }
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Volume cannot be negative: {}",
                val
            )));
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
impl From<Volume> for f64 {
    fn from(val: Volume) -> Self {
        val.0
    }
}

/// Moment of Inertia (kg·m^2).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct MomentOfInertia(f64);
impl MomentOfInertia {
    /// Creates a new `MomentOfInertia` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError::PhysicalInvariantBroken` if `val` is not finite or `val < 0.0`.
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "MomentOfInertia must be finite: {}",
                val
            )));
        }
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "MomentOfInertia cannot be negative: {}",
                val
            )));
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
impl From<MomentOfInertia> for f64 {
    fn from(val: MomentOfInertia) -> Self {
        val.0
    }
}

/// Frequency (Hz).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Frequency(f64);
impl Frequency {
    /// Creates a new `Frequency` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError::PhysicalInvariantBroken` if `val` is not finite or `val < 0.0`.
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Frequency must be finite: {}",
                val
            )));
        }
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Frequency cannot be negative: {}",
                val
            )));
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
impl From<Frequency> for f64 {
    fn from(val: Frequency) -> Self {
        val.0
    }
}
