/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;

/// Index of refraction for a medium (ratio of c to phase velocity).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct IndexOfRefraction(f64);

impl IndexOfRefraction {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        // Technically can be negative in metamaterials, but typically positive.
        // We'll enforce non-zero for now to avoid division errors in calculations.
        if val == 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Index of Refraction cannot be zero".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl From<IndexOfRefraction> for f64 {
    fn from(val: IndexOfRefraction) -> Self {
        val.0
    }
}
