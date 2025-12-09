/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::{PhysicsError, PhysicsErrorEnum};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct AmountOfSubstance(f64); // Moles

impl AmountOfSubstance {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Negative AmountOfSubstance".into()),
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct HalfLife(f64);

impl HalfLife {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Negative HalfLife".into()),
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Activity(f64);

impl Activity {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Negative Activity".into()),
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
