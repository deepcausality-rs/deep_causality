/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;

// SI primitives shared across domains live in their own module.
pub use crate::quantities::si_primitives::{Area, Frequency, Length, Mass, Speed, Volume};

/// Linear acceleration (m/s²).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Acceleration<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Acceleration<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Acceleration<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Acceleration must be finite".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Acceleration<R>> for f64 {
    fn from(val: Acceleration<R>) -> Self {
        val.0.into()
    }
}

/// Force (N).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Force<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Force<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Force<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Force must be finite".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Force<R>> for f64 {
    fn from(val: Force<R>) -> Self {
        val.0.into()
    }
}

/// Torque (N·m).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Torque<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Torque<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Torque<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Torque must be finite".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Torque<R>> for f64 {
    fn from(val: Torque<R>) -> Self {
        val.0.into()
    }
}

/// Moment of Inertia (kg·m²).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MomentOfInertia<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for MomentOfInertia<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> MomentOfInertia<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "MomentOfInertia must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "MomentOfInertia cannot be negative".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<MomentOfInertia<R>> for f64 {
    fn from(val: MomentOfInertia<R>) -> Self {
        val.0.into()
    }
}
