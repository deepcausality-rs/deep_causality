/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Datable;
use crate::types::context_node_types::data_uncertain::data_uncertain_f64::UncertainData;
use deep_causality_uncertain::{RandScalar, Uncertain};

impl<R: RandScalar> Datable for UncertainData<R> {
    type Data = Uncertain<R>;

    fn get_data(&self) -> Self::Data {
        self.data.clone()
    }

    fn set_data(&mut self, value: Self::Data) {
        self.data = value;
    }
}
