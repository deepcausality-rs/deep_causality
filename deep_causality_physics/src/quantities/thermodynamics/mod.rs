/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use deep_causality_algebra::RealField;

/// Entropy (J/K).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Entropy<R: RealField>(R);

impl<R: RealField> Default for Entropy<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Entropy<R> {
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

impl<R: RealField + Into<f64>> From<Entropy<R>> for f64 {
    fn from(val: Entropy<R>) -> Self {
        val.0.into()
    }
}

/// Thermodynamic efficiency (0.0 to 1.0).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Efficiency<R: RealField>(R);

impl<R: RealField> Default for Efficiency<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Efficiency<R> {
    /// Creates a new `Efficiency` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError::PhysicalInvariantBroken` if `val` is not in `[0, 1]`.
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        let one = R::one();
        if val < R::zero() || val > one {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Efficiency must be between 0 and 1".into(),
            ));
        }
        Ok(Self(val))
    }
}

impl<R: RealField> Efficiency<R> {
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: RealField + Into<f64>> From<Efficiency<R>> for f64 {
    fn from(val: Efficiency<R>) -> Self {
        val.0.into()
    }
}
