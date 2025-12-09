/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::{PhysicsError, PhysicsErrorEnum};
use alloc::format;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Mass(f64);

impl Mass {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                    "Mass cannot be negative: {}",
                    val
                )),
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
impl From<Mass> for f64 {
    fn from(val: Mass) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Speed(f64);

impl Speed {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                    "Speed cannot be negative: {}",
                    val
                )),
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
impl From<Speed> for f64 {
    fn from(val: Speed) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Acceleration(f64);

impl Acceleration {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Force(f64);

impl Force {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Torque(f64);
impl Torque {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Length(f64);
impl Length {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                    "Length cannot be negative: {}",
                    val
                )),
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
impl From<Length> for f64 {
    fn from(val: Length) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Area(f64);
impl Area {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                    "Area cannot be negative: {}",
                    val
                )),
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
impl From<Area> for f64 {
    fn from(val: Area) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Volume(f64);
impl Volume {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                    "Volume cannot be negative: {}",
                    val
                )),
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
impl From<Volume> for f64 {
    fn from(val: Volume) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct MomentOfInertia(f64);
impl MomentOfInertia {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                    "MomentOfInertia cannot be negative: {}",
                    val
                )),
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
impl From<MomentOfInertia> for f64 {
    fn from(val: MomentOfInertia) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Frequency(f64);
impl Frequency {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                    "Frequency cannot be negative: {}",
                    val
                )),
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
impl From<Frequency> for f64 {
    fn from(val: Frequency) -> Self {
        val.0
    }
}
