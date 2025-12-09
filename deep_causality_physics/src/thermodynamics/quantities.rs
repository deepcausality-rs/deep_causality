/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::{PhysicsError, PhysicsErrorEnum};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct Temperature(f64);

impl Temperature {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(PhysicsErrorEnum::ZeroKelvinViolation));
        }
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<Temperature> for f64 {
    fn from(val: Temperature) -> Self {
        val.0
    }
}

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
