/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::CausalityError;
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num::RealField;

/// Electric Potential (Volts or J/C).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ElectricPotential<R: RealField>(R);

impl<R: RealField> Default for ElectricPotential<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> ElectricPotential<R> {
    pub fn new(val: R) -> Result<Self, CausalityError> {
        Ok(Self(val))
    }
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: RealField + Into<f64>> From<ElectricPotential<R>> for f64 {
    fn from(val: ElectricPotential<R>) -> Self {
        val.0.into()
    }
}

/// Magnetic Flux (Webers).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MagneticFlux<R: RealField>(R);

impl<R: RealField> Default for MagneticFlux<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> MagneticFlux<R> {
    pub fn new(val: R) -> Result<Self, CausalityError> {
        Ok(Self(val))
    }
    pub fn new_unchecked(val: R) -> Self {
        Self(val)
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: RealField + Into<f64>> From<MagneticFlux<R>> for f64 {
    fn from(val: MagneticFlux<R>) -> Self {
        val.0.into()
    }
}

/// Wrapper for CausalMultiVector representing a physical field (E, B, etc.).
/// Implements Default to return a zero vector.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalField(pub CausalMultiVector<f64>);

impl Default for PhysicalField {
    fn default() -> Self {
        // Default to a zero vector in 3D Euclidean space.
        // Size of 3D Euclidean multivector is 2^3 = 8.
        Self(CausalMultiVector::new(vec![0.0; 8], Metric::Euclidean(3)).unwrap())
    }
}

impl PhysicalField {
    pub fn new(val: CausalMultiVector<f64>) -> Self {
        Self(val)
    }
    pub fn inner(&self) -> &CausalMultiVector<f64> {
        &self.0
    }
    pub fn into_inner(self) -> CausalMultiVector<f64> {
        self.0
    }
}
