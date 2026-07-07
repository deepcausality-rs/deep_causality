/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;

/// Alfven Speed ($v_A$). Characteristic speed of magnetic waves in plasma.
/// Unit: m/s. Constraint: >= 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct AlfvenSpeed<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> Default for AlfvenSpeed<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_algebra::RealField> AlfvenSpeed<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Alfven Speed must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Alfven Speed cannot be negative".into(),
            ));
        }
        Ok(Self(val))
    }
    /// Creates a new `AlfvenSpeed` without validation.
    /// Use only if the value is guaranteed to be non-negative.
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: deep_causality_algebra::RealField + Into<f64>> From<AlfvenSpeed<R>> for f64 {
    fn from(val: AlfvenSpeed<R>) -> Self {
        val.0.into()
    }
}

/// Plasma Beta ($\beta$). Ratio of thermal to magnetic pressure.
/// Unit: Dimensionless. Constraint: >= 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PlasmaBeta<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> Default for PlasmaBeta<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_algebra::RealField> PlasmaBeta<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Plasma Beta must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Plasma Beta cannot be negative".into(),
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

impl<R: deep_causality_algebra::RealField + Into<f64>> From<PlasmaBeta<R>> for f64 {
    fn from(val: PlasmaBeta<R>) -> Self {
        val.0.into()
    }
}

/// Magnetic Pressure ($P_B$). Energy density of the magnetic field.
/// Unit: Pascals (Pa). Constraint: >= 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MagneticPressure<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> Default for MagneticPressure<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_algebra::RealField> MagneticPressure<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Magnetic Pressure must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Magnetic Pressure cannot be negative".into(),
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

impl<R: deep_causality_algebra::RealField + Into<f64>> From<MagneticPressure<R>> for f64 {
    fn from(val: MagneticPressure<R>) -> Self {
        val.0.into()
    }
}

/// Larmor Radius ($r_L$). Gyroradius of a charged particle.
/// Unit: Meters (m). Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct LarmorRadius<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> LarmorRadius<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Larmor Radius must be finite".into(),
            ));
        }
        if val <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Larmor Radius must be positive".into(),
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

impl<R: deep_causality_algebra::RealField> Default for LarmorRadius<R> {
    /// Returns machine epsilon as the smallest representable positive value
    /// that satisfies the > 0 constraint.
    fn default() -> Self {
        Self(R::epsilon())
    }
}

impl<R: deep_causality_algebra::RealField + Into<f64>> From<LarmorRadius<R>> for f64 {
    fn from(val: LarmorRadius<R>) -> Self {
        val.0.into()
    }
}

/// Debye Length ($\lambda_D$). Screening length in plasma.
/// Unit: Meters (m). Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct DebyeLength<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> DebyeLength<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Debye Length must be finite".into(),
            ));
        }
        if val <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Debye Length must be positive".into(),
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

impl<R: deep_causality_algebra::RealField> Default for DebyeLength<R> {
    /// Returns machine epsilon as the smallest representable positive value
    /// that satisfies the > 0 constraint.
    fn default() -> Self {
        Self(R::epsilon())
    }
}

impl<R: deep_causality_algebra::RealField + Into<f64>> From<DebyeLength<R>> for f64 {
    fn from(val: DebyeLength<R>) -> Self {
        val.0.into()
    }
}

/// Plasma Frequency ($\omega_{pe}$). Natural oscillation frequency.
/// Unit: Rad/s. Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PlasmaFrequency<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> PlasmaFrequency<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Plasma Frequency must be finite".into(),
            ));
        }
        if val <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Plasma Frequency must be positive".into(),
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

impl<R: deep_causality_algebra::RealField> Default for PlasmaFrequency<R> {
    /// Returns machine epsilon as the smallest representable positive value
    /// that satisfies the > 0 constraint.
    fn default() -> Self {
        Self(R::epsilon())
    }
}

impl<R: deep_causality_algebra::RealField + Into<f64>> From<PlasmaFrequency<R>> for f64 {
    fn from(val: PlasmaFrequency<R>) -> Self {
        val.0.into()
    }
}

/// Electrical Conductivity ($\sigma$).
/// Unit: Siemens/m (S/m). Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Conductivity<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> Conductivity<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Conductivity must be finite".into(),
            ));
        }
        if val <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Conductivity must be positive".into(),
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

impl<R: deep_causality_algebra::RealField> Default for Conductivity<R> {
    /// Returns machine epsilon as the smallest representable positive value
    /// that satisfies the > 0 constraint.
    fn default() -> Self {
        Self(R::epsilon())
    }
}

impl<R: deep_causality_algebra::RealField + Into<f64>> From<Conductivity<R>> for f64 {
    fn from(val: Conductivity<R>) -> Self {
        val.0.into()
    }
}

/// Magnetic Diffusivity ($\eta$).
/// Unit: $m^2/s$. Constraint: >= 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Diffusivity<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> Default for Diffusivity<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_algebra::RealField> Diffusivity<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Diffusivity must be finite".into(),
            ));
        }
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Diffusivity cannot be negative".into(),
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

impl<R: deep_causality_algebra::RealField + Into<f64>> From<Diffusivity<R>> for f64 {
    fn from(val: Diffusivity<R>) -> Self {
        val.0.into()
    }
}
