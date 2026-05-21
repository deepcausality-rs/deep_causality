/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::PhysicsError;
use deep_causality_num::RealField;

/// Phase Angle (Radians).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PhaseAngle<R: RealField>(R);

impl<R: RealField> Default for PhaseAngle<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> PhaseAngle<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "PhaseAngle must be finite".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: RealField + Into<f64>> From<PhaseAngle<R>> for f64 {
    fn from(val: PhaseAngle<R>) -> Self {
        val.0.into()
    }
}
