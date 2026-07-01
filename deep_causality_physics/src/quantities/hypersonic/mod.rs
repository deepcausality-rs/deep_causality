/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reacting / weakly-ionized-air quantity newtypes for the hypersonic Park-2T
//! blackout slice (Gap 2, Tier-A). These complement the existing MHD plasma
//! quantities (`PlasmaFrequency`, `DebyeLength`) in `quantities/mhd/`, which are
//! reused — not duplicated — by the hypersonic kernels.

use crate::PhysicsError;

/// Electron number density $n_e$. Unit: $m^{-3}$. Constraint: finite, $\geq 0$.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ElectronDensity<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for ElectronDensity<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> ElectronDensity<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Electron density must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Electron density cannot be negative".into(),
            ));
        }
        Ok(Self(val))
    }
    /// Creates a new `ElectronDensity` without validation.
    /// Use only if the value is guaranteed finite and non-negative.
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_num::RealField + Into<f64>> From<ElectronDensity<R>> for f64 {
    fn from(val: ElectronDensity<R>) -> Self {
        val.0.into()
    }
}

/// Ionization fraction $\alpha = n_e / n_{tot}$. Unit: dimensionless.
/// Constraint: finite, $\in [0, 1]$.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct IonizationFraction<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for IonizationFraction<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> IonizationFraction<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::NormalizationError(
                "Ionization fraction must be finite".into(),
            ));
        }
        if val < R::zero() || val > R::one() {
            return Err(PhysicsError::NormalizationError(
                "Ionization fraction must lie in [0, 1]".into(),
            ));
        }
        Ok(Self(val))
    }
    /// Creates a new `IonizationFraction` without validation.
    /// Use only if the value is guaranteed finite and in `[0, 1]`.
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_num::RealField + Into<f64>> From<IonizationFraction<R>> for f64 {
    fn from(val: IonizationFraction<R>) -> Self {
        val.0.into()
    }
}

/// Free-electron translational temperature $T_e$. Unit: K. Constraint: finite, $\geq 0$.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ElectronTemperature<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for ElectronTemperature<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> ElectronTemperature<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Electron temperature must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::ZeroKelvinViolation());
        }
        Ok(Self(val))
    }
    /// Creates a new `ElectronTemperature` without validation.
    /// Use only if the value is guaranteed finite and non-negative.
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_num::RealField + Into<f64>> From<ElectronTemperature<R>> for f64 {
    fn from(val: ElectronTemperature<R>) -> Self {
        val.0.into()
    }
}

/// Vibrational (vibrational–electronic) temperature $T_{ve}$. Unit: K.
/// Constraint: finite, $\geq 0$.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct VibrationalTemperature<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for VibrationalTemperature<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> VibrationalTemperature<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Vibrational temperature must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::ZeroKelvinViolation());
        }
        Ok(Self(val))
    }
    /// Creates a new `VibrationalTemperature` without validation.
    /// Use only if the value is guaranteed finite and non-negative.
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_num::RealField + Into<f64>> From<VibrationalTemperature<R>> for f64 {
    fn from(val: VibrationalTemperature<R>) -> Self {
        val.0.into()
    }
}

/// Species mass fraction $Y_s = \rho_s / \rho$. Unit: dimensionless.
/// Constraint: finite, $\in [0, 1]$.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MassFraction<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for MassFraction<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> MassFraction<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::NormalizationError(
                "Mass fraction must be finite".into(),
            ));
        }
        if val < R::zero() || val > R::one() {
            return Err(PhysicsError::NormalizationError(
                "Mass fraction must lie in [0, 1]".into(),
            ));
        }
        Ok(Self(val))
    }
    /// Creates a new `MassFraction` without validation.
    /// Use only if the value is guaranteed finite and in `[0, 1]`.
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_num::RealField + Into<f64>> From<MassFraction<R>> for f64 {
    fn from(val: MassFraction<R>) -> Self {
        val.0.into()
    }
}

/// Reaction rate (forward or backward rate coefficient evaluated at a
/// rate-controlling temperature). Unit: model-dependent (e.g. $m^3 mol^{-1} s^{-1}$).
/// Constraint: finite, $\geq 0$.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ReactionRate<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for ReactionRate<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> ReactionRate<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Reaction rate must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Reaction rate cannot be negative".into(),
            ));
        }
        Ok(Self(val))
    }
    /// Creates a new `ReactionRate` without validation.
    /// Use only if the value is guaranteed finite and non-negative.
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_num::RealField + Into<f64>> From<ReactionRate<R>> for f64 {
    fn from(val: ReactionRate<R>) -> Self {
        val.0.into()
    }
}
