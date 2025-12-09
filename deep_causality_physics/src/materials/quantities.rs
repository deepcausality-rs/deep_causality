/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::{PhysicsError, PhysicsErrorEnum};
use deep_causality_core::CausalityError;

// Scalar Stress/Stiffness if needed, though mostly Tensors are used.
// Defining them for completeness or future scalar ops.

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Stress(f64);

impl Stress {
    pub fn new(val: f64) -> Result<Self, CausalityError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<Stress> for f64 {
    fn from(val: Stress) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Stiffness(f64);

impl Stiffness {
    pub fn new(val: f64) -> Result<Self, CausalityError> {
        if val < 0.0 {
            return Err(CausalityError::from(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Negative Stiffness (Scalar)".into()),
            )));
        }
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<Stiffness> for f64 {
    fn from(val: Stiffness) -> Self {
        val.0
    }
}
