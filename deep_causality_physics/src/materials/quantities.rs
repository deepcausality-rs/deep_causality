/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use deep_causality_num::RealField;
use deep_causality_tensor::CausalTensor;

// =============================================================================
// Scalar quantities
// =============================================================================

/// Scalar stress (Pascals), used for simple 1D cases or invariants (Von Mises).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Stress<R: RealField>(R);

impl<R: RealField> Default for Stress<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Stress<R> {
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        Ok(Self(val))
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: RealField + Into<f64>> From<Stress<R>> for f64 {
    fn from(val: Stress<R>) -> Self {
        val.0.into()
    }
}

/// Scalar stiffness (Young's Modulus, etc.) (Pascals).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Stiffness<R: RealField>(R);

impl<R: RealField> Default for Stiffness<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: RealField> Stiffness<R> {
    /// Creates a new `Stiffness` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError` if `val < 0`.
    pub fn new(val: R) -> Result<Self, PhysicsError> {
        if val < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Negative Stiffness (Scalar)".into(),
            ));
        }
        Ok(Self(val))
    }
    pub fn value(&self) -> R {
        self.0
    }
}

impl<R: RealField + Into<f64>> From<Stiffness<R>> for f64 {
    fn from(val: Stiffness<R>) -> Self {
        val.0.into()
    }
}

// =============================================================================
// Tensor quantities
// =============================================================================

/// Strain tensor field $\boldsymbol{\epsilon}$ (Rank 2).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Strain<R: RealField>(CausalTensor<R>);

impl<R: RealField> Strain<R> {
    pub fn new(tensor: CausalTensor<R>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<R> {
        &self.0
    }
    pub fn into_inner(self) -> CausalTensor<R> {
        self.0
    }
}

/// Stiffness tensor $C_{ijkl}$ (Rank 4) used in generalized Hooke's law.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StiffnessTensor<R: RealField>(CausalTensor<R>);

impl<R: RealField> StiffnessTensor<R> {
    pub fn new(tensor: CausalTensor<R>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<R> {
        &self.0
    }
    pub fn into_inner(self) -> CausalTensor<R> {
        self.0
    }
}

/// Cauchy stress tensor $\boldsymbol{\sigma}$ (Rank 2).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StressTensor<R: RealField>(CausalTensor<R>);

impl<R: RealField> StressTensor<R> {
    pub fn new(tensor: CausalTensor<R>) -> Self {
        Self(tensor)
    }
    pub fn inner(&self) -> &CausalTensor<R> {
        &self.0
    }
    pub fn into_inner(self) -> CausalTensor<R> {
        self.0
    }
}
