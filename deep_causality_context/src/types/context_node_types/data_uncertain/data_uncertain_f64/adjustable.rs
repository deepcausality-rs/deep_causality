/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::AdjustmentError;
use crate::UncertainAdjustable;
use crate::types::context_node_types::data_uncertain::data_uncertain_f64::UncertainData;
use deep_causality_uncertain::RandScalar;
use deep_causality_uncertain::Uncertain;

/// Replacing the whole distribution is what adjusting an uncertain quantity means.
///
/// This file held only a licence header before: the Boolean node had an `UncertainAdjustable` impl
/// and the real one did not, an asymmetry with no reason behind it. Both channels adjust the same
/// way — the new distribution supersedes the old one, because there is no partial update of a
/// distribution that is not just a different distribution.
impl<R: RandScalar> UncertainAdjustable for UncertainData<R> {
    type Data = Uncertain<R>;

    fn update(&mut self, uncertain: Self::Data) -> Result<(), AdjustmentError> {
        self.data = uncertain;
        Ok(())
    }

    fn adjust(&mut self, uncertain: Self::Data) -> Result<(), AdjustmentError> {
        self.data = uncertain;
        Ok(())
    }
}
