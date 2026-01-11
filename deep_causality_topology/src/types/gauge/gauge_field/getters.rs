/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GaugeField, GaugeGroup, Manifold};
use deep_causality_metric::Metric;
use deep_causality_tensor::CausalTensor;

impl<G: GaugeGroup, T, A, F> GaugeField<G, T, A, F> {
    /// Returns a reference to the base manifold.
    #[inline]
    pub fn base(&self) -> &Manifold<T, T> {
        &self.base
    }

    /// Returns the spacetime metric signature.
    #[inline]
    pub fn metric(&self) -> Metric {
        self.metric
    }

    /// Returns a reference to the gauge connection (potential).
    #[inline]
    pub fn connection(&self) -> &CausalTensor<A> {
        &self.connection
    }

    /// Returns a reference to the field strength (curvature).
    #[inline]
    pub fn field_strength(&self) -> &CausalTensor<F> {
        &self.field_strength
    }

    /// Returns the human-readable name of the gauge group.
    #[inline]
    pub fn gauge_group_name(&self) -> &'static str {
        G::name()
    }

    /// Returns the dimension of the Lie algebra (number of generators).
    #[inline]
    pub fn lie_algebra_dim(&self) -> usize {
        G::LIE_ALGEBRA_DIM
    }

    /// Returns whether the gauge group is abelian.
    ///
    /// For abelian groups: F = dA
    /// For non-abelian groups: F = dA + A∧A
    #[inline]
    pub fn is_abelian(&self) -> bool {
        G::IS_ABELIAN
    }

    /// Returns the spacetime dimension.
    #[inline]
    pub fn spacetime_dim(&self) -> usize {
        G::SPACETIME_DIM
    }

    /// Checks if using East Coast convention (-+++).
    ///
    /// East Coast is standard in GR textbooks (MTW, Wald).
    #[inline]
    pub fn is_east_coast(&self) -> bool {
        self.metric.sign_of_sq(0) == -1
    }

    /// Checks if using West Coast convention (+---).
    ///
    /// West Coast is standard in particle physics (Weinberg, Peskin & Schroeder).
    #[inline]
    pub fn is_west_coast(&self) -> bool {
        self.metric.sign_of_sq(0) == 1
            && self.metric.dimension() > 1
            && self.metric.sign_of_sq(1) == -1
    }
}
