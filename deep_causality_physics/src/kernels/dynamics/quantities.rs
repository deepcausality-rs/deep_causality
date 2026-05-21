/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;

/// Mass quantity (kg).
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

/// Speed quantity (scalar magnitude of velocity) (m/s).
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

/// Linear acceleration (m/s^2).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Acceleration<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Acceleration<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Acceleration<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Acceleration must be finite".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Acceleration<R>> for f64 {
    fn from(val: Acceleration<R>) -> Self {
        val.0.into()
    }
}

/// Force (N).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Force<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Force<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Force<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Force must be finite".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Force<R>> for f64 {
    fn from(val: Force<R>) -> Self {
        val.0.into()
    }
}

/// Torque (N·m).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Torque<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Torque<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Torque<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Torque must be finite".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Torque<R>> for f64 {
    fn from(val: Torque<R>) -> Self {
        val.0.into()
    }
}

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

/// Area (m^2).
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

/// Volume (m^3).
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

/// Moment of Inertia (kg·m^2).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MomentOfInertia<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for MomentOfInertia<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> MomentOfInertia<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "MomentOfInertia must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "MomentOfInertia cannot be negative".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<MomentOfInertia<R>> for f64 {
    fn from(val: MomentOfInertia<R>) -> Self {
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
