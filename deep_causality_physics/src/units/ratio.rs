/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;

/// A generic dimensionless ratio.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Ratio(f64);

impl Ratio {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl From<Ratio> for f64 {
    fn from(val: Ratio) -> Self {
        val.0
    }
}
