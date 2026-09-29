/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::context_node_types::data_uncertain::uncertain_data::UncertainData;
use core::fmt::{Debug, Display, Formatter};
use deep_causality_uncertain::RandScalar;

impl<R: RandScalar + Debug> Display for UncertainData<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "UncertainData: id: {} data: {:?}", self.id, self.data)
    }
}
