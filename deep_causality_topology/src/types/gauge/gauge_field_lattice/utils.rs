/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GaugeGroup, LatticeCell, LatticeGaugeField, LinkVariable};
use deep_causality_tensor::TensorData;

impl<G: GaugeGroup, const D: usize, T: TensorData + From<f64>> LatticeGaugeField<G, D, T> {
    /// Get a link, returning identity if not found.
    pub(crate) fn get_link_or_identity(&self, edge: &LatticeCell<D>) -> LinkVariable<G, T> {
        self.links
            .get(edge)
            .cloned()
            .unwrap_or_else(LinkVariable::identity)
    }
}
