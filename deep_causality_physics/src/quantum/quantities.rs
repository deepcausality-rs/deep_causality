/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::{PhysicsError, PhysicsErrorEnum};
use alloc::format;

/// Probability value [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Probability(f64);

impl Default for Probability {
    fn default() -> Self {
        Self(0.0)
    }
}

impl Probability {
    /// Creates a new `Probability` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError::NormalizationError` if `val` is not in [0, 1].
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

/// Phase Angle (Radians).
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
