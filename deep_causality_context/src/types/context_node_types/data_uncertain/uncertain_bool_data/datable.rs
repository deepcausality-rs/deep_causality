/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Datable;
use crate::types::context_node_types::data_uncertain::uncertain_bool_data::UncertainBoolData;
use deep_causality_uncertain::{RandScalar, UncertainBool};

impl<R: RandScalar> Datable for UncertainBoolData<R> {
    type Data = UncertainBool<R>;

    fn get_data(&self) -> Self::Data {
        self.data.clone()
    }

    fn set_data(&mut self, value: Self::Data) {
        self.data = value;
    }
}
