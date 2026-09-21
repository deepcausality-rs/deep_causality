/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::AdjustmentError;
use crate::UncertainAdjustable;
use crate::types::context_node_types::data_uncertain::data_uncertain_bool::UncertainBoolData;
use deep_causality_uncertain::{RandScalar, UncertainBool};

impl<R: RandScalar> UncertainAdjustable for UncertainBoolData<R> {
    type Data = UncertainBool<R>;

    fn update(&mut self, uncertain: Self::Data) -> Result<(), AdjustmentError> {
        self.data = uncertain;

        Ok(())
    }

    fn adjust(&mut self, uncertain: Self::Data) -> Result<(), AdjustmentError> {
        self.data = uncertain;

        Ok(())
    }
}
