/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::PhysicsError;
use deep_causality_num::RealField;

/// Probability value [0, 1].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Probability<R: RealField>(R);

impl<R: RealField> Default for Probability<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Probability<R> {
    /// Creates a new `Probability` instance.
    ///
    /// # Errors
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
