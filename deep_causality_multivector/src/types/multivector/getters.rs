/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalMultiVector, Metric};

impl<T> CausalMultiVector<T> {
    pub fn data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn metric(&self) -> Metric {
        self.metric
    }
}
