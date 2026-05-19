/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Metric for cubical complexes (Stage B — unit-edge case only).
//!
//! `CubicalMetric<D>` is the cubical analogue of `ReggeGeometry<T>` and is wired through
//! the `ChainComplex::Metric` associated type. The Stage B / issue #487 scope is the
//! unit-edge case: every edge has length `1.0`. Non-unit / scaled / curved cubical
//! metrics are out of scope for this change set and tracked as a follow-up.

/// Unit-edge metric for a D-dimensional cubical complex.
///
/// Carries a single flag indicating whether the metric is the unit-edge case
/// (the only case supported by this change set). The struct is intentionally
/// minimal — a richer representation will replace it when non-uniform spacing
/// is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CubicalMetric<const D: usize> {
    unit_edge: bool,
}

impl<const D: usize> CubicalMetric<D> {
    /// Constructs the unit-edge cubical metric: every edge has length `1.0`.
    pub fn unit() -> Self {
        Self { unit_edge: true }
    }

    /// Returns `true` iff every edge length is `1.0`.
    pub fn is_unit_edge(&self) -> bool {
        self.unit_edge
    }
}
