/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{PhysicsError, PhysicsErrorEnum};

/// Alfven Speed ($v_A$). Characteristic speed of magnetic waves in plasma.
/// Unit: m/s. Constraint: >= 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct AlfvenSpeed(f64);

impl AlfvenSpeed {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Alfven Speed cannot be negative".into()),
            ));
        }
        Ok(Self(val))
    }
    /// Creates a new `AlfvenSpeed` without validation.
    /// Use only if the value is guaranteed to be non-negative.
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}

/// Plasma Beta ($\beta$). Ratio of thermal to magnetic pressure.
/// Unit: Dimensionless. Constraint: >= 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct PlasmaBeta(f64);

impl PlasmaBeta {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Plasma Beta cannot be negative".into()),
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

/// Magnetic Pressure ($P_B$). Energy density of the magnetic field.
/// Unit: Pascals (Pa). Constraint: >= 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct MagneticPressure(f64);

impl MagneticPressure {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(
                    "Magnetic Pressure cannot be negative".into(),
                ),
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

/// Larmor Radius ($r_L$). Gyroradius of a charged particle.
/// Unit: Meters (m). Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct LarmorRadius(f64);

impl LarmorRadius {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Larmor Radius must be positive".into()),
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

/// Debye Length ($\lambda_D$). Screening length in plasma.
/// Unit: Meters (m). Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct DebyeLength(f64);

impl DebyeLength {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Debye Length must be positive".into()),
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

/// Plasma Frequency ($\omega_{pe}$). Natural oscillation frequency.
/// Unit: Rad/s. Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct PlasmaFrequency(f64);

impl PlasmaFrequency {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(
                    "Plasma Frequency must be positive".into(),
                ),
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

/// Electrical Conductivity ($\sigma$).
/// Unit: Siemens/m (S/m). Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Conductivity(f64);

impl Conductivity {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Conductivity must be positive".into()),
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

/// Magnetic Diffusivity ($\eta$).
/// Unit: $m^2/s$. Constraint: >= 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Diffusivity(f64);

impl Diffusivity {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Diffusivity cannot be negative".into()),
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
