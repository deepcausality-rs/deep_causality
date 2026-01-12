/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{GaugeGroup, LatticeCell, LatticeGaugeField, LinkVariable};
use deep_causality_num::{ComplexField, DivisionAlgebra, FromPrimitive, RealField, ToPrimitive};
use deep_causality_tensor::TensorData;
use std::fmt::Debug;

impl<
    G: GaugeGroup,
    const D: usize,
    M: TensorData + Debug + ComplexField<R> + DivisionAlgebra<R>,
    R: RealField + FromPrimitive + ToPrimitive,
> LatticeGaugeField<G, D, M, R>
{
    /// Get a link, returning identity if not found.
    pub(crate) fn get_link_or_identity(&self, edge: &LatticeCell<D>) -> LinkVariable<G, M, R> {
        self.links
            .get(edge)
            .cloned()
            .unwrap_or_else(LinkVariable::identity)
    }
}
