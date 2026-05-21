/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;

/// Pressure (Pascals).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Pressure(f64);

impl Pressure {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "Pressure must be finite: {}",
                val
            )));
        }
        if val < 0.0 {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative Pressure".into(),
            ));
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
impl From<Pressure> for f64 {
    fn from(val: Pressure) -> Self {
        val.0
    }
}

/// Density (kg/m^3).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Density<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Density<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Density<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Density must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative Density".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Density<R>> for f64 {
    fn from(val: Density<R>) -> Self {
        val.0.into()
    }
}

/// Dynamic Viscosity (Pa·s).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Viscosity<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Viscosity<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Viscosity<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Viscosity must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative Viscosity".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Viscosity<R>> for f64 {
    fn from(val: Viscosity<R>) -> Self {
        val.0.into()
    }
}
