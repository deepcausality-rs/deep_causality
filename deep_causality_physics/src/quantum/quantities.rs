/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::{PhysicsError, PhysicsErrorEnum};
use alloc::format;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]

pub struct Probability(f64);

impl Default for Probability {
    fn default() -> Self {
        Self(0.0)
    }
}

impl Probability {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !(0.0..=1.0).contains(&val) {
            return Err(PhysicsError::new(PhysicsErrorEnum::NormalizationError(
                format!("Probability must be between 0 and 1, got {}", val),
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
impl From<Probability> for f64 {
    fn from(val: Probability) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Energy(f64);

impl Energy {
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
impl From<Energy> for f64 {
    fn from(val: Energy) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Time(f64);

impl Time {
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
impl From<Time> for f64 {
    fn from(val: Time) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct PhaseAngle(f64);

impl PhaseAngle {
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
impl From<PhaseAngle> for f64 {
    fn from(val: PhaseAngle) -> Self {
        val.0
    }
}
