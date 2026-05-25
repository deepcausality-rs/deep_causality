/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;

/// Pressure (Pascals).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Pressure<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for Pressure<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> Pressure<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Pressure must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative Pressure".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<Pressure<R>> for f64 {
    fn from(val: Pressure<R>) -> Self {
        val.0.into()
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

/// Kinematic Viscosity (m^2/s). Equals dynamic viscosity divided by density.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct KinematicViscosity<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for KinematicViscosity<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> KinematicViscosity<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "KinematicViscosity must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative KinematicViscosity".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<KinematicViscosity<R>> for f64 {
    fn from(val: KinematicViscosity<R>) -> Self {
        val.0.into()
    }
}

/// Specific Enthalpy (J/kg). Reference-state dependent; may be negative.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SpecificEnthalpy<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for SpecificEnthalpy<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> SpecificEnthalpy<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "SpecificEnthalpy must be finite".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<SpecificEnthalpy<R>> for f64 {
    fn from(val: SpecificEnthalpy<R>) -> Self {
        val.0.into()
    }
}

/// Wall Shear Stress magnitude (Pa). Stored as magnitude; sign convention is
/// carried by the calling context, not by this type.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct WallShearStress<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for WallShearStress<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> WallShearStress<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "WallShearStress must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative WallShearStress".into(),
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

impl<R: deep_causality_num::RealField + Into<f64>> From<WallShearStress<R>> for f64 {
    fn from(val: WallShearStress<R>) -> Self {
        val.0.into()
    }
}
