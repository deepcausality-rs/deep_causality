/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use deep_causality_num::RealField;

/// Index of refraction for a medium (ratio of c to phase velocity).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct IndexOfRefraction<R: RealField>(R);

impl<R: RealField> Default for IndexOfRefraction<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> IndexOfRefraction<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        // Technically can be negative in metamaterials, but typically positive.
        // We'll enforce non-zero for now to avoid division errors in calculations.
        if val == R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Index of Refraction cannot be zero".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: RealField + Into<f64>> From<IndexOfRefraction<R>> for f64 {
    fn from(val: IndexOfRefraction<R>) -> Self {
        val.0.into()
    }
}
