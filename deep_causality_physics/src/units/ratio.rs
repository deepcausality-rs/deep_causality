/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use deep_causality_num::RealField;

/// A generic dimensionless ratio.
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
