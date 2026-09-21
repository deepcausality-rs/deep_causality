/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::AdjustmentError;
use crate::UncertainAdjustable;
use crate::types::context_node_types::data_uncertain::uncertain_data::UncertainData;
use deep_causality_uncertain::RandScalar;
use deep_causality_uncertain::Uncertain;

/// Both methods replace the whole distribution.
///
/// Adjusting an uncertain quantity means supplying a different distribution: there is no partial
/// update of a distribution that is not simply another distribution. `update` and `adjust`
/// therefore do the same thing, and neither can fail.
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
