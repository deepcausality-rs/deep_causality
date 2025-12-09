/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::{PhysicsError, PhysicsErrorEnum};

// Global dimensionless ratios and common types

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Ratio(f64);

impl Ratio {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<Ratio> for f64 {
    fn from(val: Ratio) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Efficiency(f64);

impl Efficiency {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !(0.0..=1.0).contains(&val) {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(
                    "Efficiency must be between 0 and 1".into(),
                ),
            ));
        }
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<Efficiency> for f64 {
    fn from(val: Efficiency) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct IndexOfRefraction(f64);

impl IndexOfRefraction {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<IndexOfRefraction> for f64 {
    fn from(val: IndexOfRefraction) -> Self {
        val.0
    }
}
