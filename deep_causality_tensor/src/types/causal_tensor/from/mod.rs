/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalTensor;

impl<T: Clone> From<T> for CausalTensor<T> {
    /// Creates a scalar tensor (0-dimensional) from a single value.
    fn from(item: T) -> Self {
        CausalTensor::new(vec![item], vec![]).expect("Failed to create scalar CausalTensor from T")
    }
}

impl<'a, T: Clone> From<&'a T> for CausalTensor<T> {
    /// Creates a scalar tensor (0-dimensional) from a reference to a single value.
    fn from(item: &'a T) -> Self {
        CausalTensor::new(vec![item.clone()], vec![])
            .expect("Failed to create scalar CausalTensor from &T")
    }
}
