/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::CausalityError;
use deep_causality_multivector::{CausalMultiVector, Metric};

/// Electric Potential (Volts or J/C).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct ElectricPotential(f64);

impl ElectricPotential {
    pub fn new(val: f64) -> Result<Self, CausalityError> {
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<ElectricPotential> for f64 {
    fn from(val: ElectricPotential) -> Self {
        val.0
    }
}

/// Magnetic Flux (Webers).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct MagneticFlux(f64);

impl MagneticFlux {
    pub fn new(val: f64) -> Result<Self, CausalityError> {
        Ok(Self(val))
    }
    pub fn new_unchecked(val: f64) -> Self {
        Self(val)
    }
    pub fn value(&self) -> f64 {
        self.0
    }
}
impl From<MagneticFlux> for f64 {
    fn from(val: MagneticFlux) -> Self {
        val.0
    }
}

/// Wrapper for CausalMultiVector representing a physical field (E, B, etc.).
/// Implements Default to return a zero vector.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalField(pub CausalMultiVector<f64>);

impl Default for PhysicalField {
    fn default() -> Self {
        // Return a scalar 0 multivector with Euclidean metric
        Self(CausalMultiVector::new(vec![0.0], Metric::Euclidean(0)).unwrap())
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
