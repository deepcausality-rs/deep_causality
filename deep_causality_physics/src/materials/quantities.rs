/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{PhysicsError, PhysicsErrorEnum};

// Scalar Stress/Stiffness if needed, though mostly Tensors are used.
// Defining them for completeness or future scalar ops.

/// Scalar stress (Pascals), used for simple 1D cases or invariants (Von Mises).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Stress(f64);

impl Stress {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
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

/// Scalar stiffness (Young's Modulus, etc.) (Pascals).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Stiffness(f64);

impl Stiffness {
    /// Creates a new `Stiffness` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError` if `val < 0.0`.
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Negative Stiffness (Scalar)".into()),
            ));
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
