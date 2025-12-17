/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::PhysicsError;
/// Phase Angle (Radians).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct PhaseAngle(f64);

impl PhaseAngle {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::new(
                crate::PhysicsErrorEnum::NumericalInstability(format!(
                    "PhaseAngle must be finite, got {}",
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
impl From<PhaseAngle> for f64 {
    fn from(val: PhaseAngle) -> Self {
        val.0
    }
}
