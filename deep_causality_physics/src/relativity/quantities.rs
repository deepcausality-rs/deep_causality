/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::CausalityError;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]

pub struct SpacetimeInterval(f64);

impl SpacetimeInterval {
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
impl From<SpacetimeInterval> for f64 {
    fn from(val: SpacetimeInterval) -> Self {
        val.0
    }
}

use deep_causality_multivector::{CausalMultiVector, Metric};

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
