/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Dimensionless scalars: quantities that carry no SI unit and are used
//! across multiple physics domains.

use crate::PhysicsError;
use deep_causality_num::RealField;

/// A generic dimensionless ratio (no physical unit).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Ratio<R: RealField>(R);

impl<R: RealField> Default for Ratio<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Ratio<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: RealField + Into<f64>> From<Ratio<R>> for f64 {
    fn from(val: Ratio<R>) -> Self {
        val.0.into()
    }
}

/// Phase angle (radians) — dimensionless angle used in wave and quantum physics.
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

/// Probability — a dimensionless scalar constrained to [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Probability<R: RealField>(R);

impl<R: RealField> Default for Probability<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Probability<R> {
    /// Returns `PhysicsError::NormalizationError` if `val` is not in `[0, 1]`.
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() || val < R::zero() || val > R::one() {
            return Err(PhysicsError::NormalizationError(
                "Probability must be between 0 and 1".into(),
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

impl<R: RealField + Into<f64>> From<Probability<R>> for f64 {
    fn from(val: Probability<R>) -> Self {
        val.0.into()
    }
}
