/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::CausalityError;

/// Spacetime Interval ($s^2$).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SpacetimeInterval<R: deep_causality_algebra::RealField>(R);

impl<R: deep_causality_algebra::RealField> Default for SpacetimeInterval<R> {
    fn default() -> Self {
        Self(R::zero())
    }
}

impl<R: deep_causality_algebra::RealField> SpacetimeInterval<R> {
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

impl<R: deep_causality_algebra::RealField + Into<f64>> From<SpacetimeInterval<R>> for f64 {
    fn from(val: SpacetimeInterval<R>) -> Self {
        val.0.into()
    }
}

use deep_causality_algebra::RealField;
use deep_causality_multivector::{CausalMultiVector, Metric};

/// Wrapper for CausalMultiVector representing a vector in Spacetime.
///
/// Implements Default using Minkowski metric.
#[derive(Debug, Clone, PartialEq)]
pub struct SpacetimeVector<R: RealField>(pub CausalMultiVector<R>);

impl<R: RealField> Default for SpacetimeVector<R> {
    fn default() -> Self {
        // 4D Minkowski spacetime: 2^4 = 16 multivector components, matching the
        // convention used by every other spacetime vector constructed in this crate
        // (see relativity/spacetime.rs and the test fixtures). A 0-dimensional
        // Minkowski metric would degenerate to a single scalar and would not
        // represent a meaningful spacetime quantity.
        Self(CausalMultiVector::new(vec![R::zero(); 16], Metric::Minkowski(4)).unwrap())
    }
}

impl<R: RealField> SpacetimeVector<R> {
    pub fn new(val: CausalMultiVector<R>) -> Self {
        Self(val)
    }
    pub fn inner(&self) -> &CausalMultiVector<R> {
        &self.0
    }
    pub fn into_inner(self) -> CausalMultiVector<R> {
        self.0
    }
}
