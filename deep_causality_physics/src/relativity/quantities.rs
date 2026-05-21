/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::CausalityError;

/// Spacetime Interval ($s^2$).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SpacetimeInterval<R: deep_causality_num::RealField>(R);

impl<R: deep_causality_num::RealField> Default for SpacetimeInterval<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_num::RealField> SpacetimeInterval<R> {
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

impl<R: deep_causality_num::RealField + Into<f64>> From<SpacetimeInterval<R>> for f64 {
    fn from(val: SpacetimeInterval<R>) -> Self {
        val.0.into()
    }
}

use deep_causality_multivector::{CausalMultiVector, Metric};

/// Wrapper for CausalMultiVector representing a vector in Spacetime.
///
/// Implements Default using Minkowski metric.
#[derive(Debug, Clone, PartialEq)]
pub struct SpacetimeVector(pub CausalMultiVector<f64>);

impl Default for SpacetimeVector {
    fn default() -> Self {
        Self(CausalMultiVector::new(vec![0.0], Metric::Minkowski(0)).unwrap())
    }
}

impl SpacetimeVector {
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
