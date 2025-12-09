/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{PhysicsError, PhysicsErrorEnum};

/// Entropy (J/K).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Entropy(f64);

impl Entropy {
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
impl From<Entropy> for f64 {
    fn from(val: Entropy) -> Self {
        val.0
    }
}

/// Thermodynamic efficiency (0.0 to 1.0).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Efficiency(f64);

impl Efficiency {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !(0.0..=1.0).contains(&val) {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(
                    "Efficiency must be between 0 and 1".into(),
                ),
            ));
        }
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl From<Efficiency> for f64 {
    fn from(val: Efficiency) -> Self {
        val.0
    }
}
