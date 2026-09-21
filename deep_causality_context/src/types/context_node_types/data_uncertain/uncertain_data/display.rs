/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::context_node_types::data_uncertain::uncertain_data::UncertainData;
use deep_causality_uncertain::RandScalar;
use std::fmt::{Debug, Display, Formatter};

impl<R: RandScalar + Debug> Display for UncertainData<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "UncertainData: id: {} data: {:?}", self.id, self.data)
    }
}
