/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Fundamental SI-base-and-derived scalars that span multiple physics domains.
//! These types are imported by fluid, photonics, thermodynamics, and other
//! domain kernels rather than living exclusively in `dynamics`.

use crate::PhysicsError;

/// Length (m).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Length<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Length<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Length<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Length must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Length cannot be negative".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Length<R>> for f64 {
    fn from(val: Length<R>) -> Self {
        val.0.into()
    }
}

/// Area (m²).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Area<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Area<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Area<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Area must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Area cannot be negative".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Area<R>> for f64 {
    fn from(val: Area<R>) -> Self {
        val.0.into()
    }
}

/// Volume (m³).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Volume<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Volume<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Volume<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Volume must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Volume cannot be negative".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Volume<R>> for f64 {
    fn from(val: Volume<R>) -> Self {
        val.0.into()
    }
}

/// Mass (kg).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Mass<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Mass<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Mass<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Mass must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Mass cannot be negative".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Mass<R>> for f64 {
    fn from(val: Mass<R>) -> Self {
        val.0.into()
    }
}

/// Speed — scalar magnitude of velocity (m/s).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Speed<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Speed<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Speed<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Speed must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Speed cannot be negative".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Speed<R>> for f64 {
    fn from(val: Speed<R>) -> Self {
        val.0.into()
    }
}

/// Frequency (Hz).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Frequency<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Frequency<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Frequency<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Frequency must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Frequency cannot be negative".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Frequency<R>> for f64 {
    fn from(val: Frequency<R>) -> Self {
        val.0.into()
    }
}
