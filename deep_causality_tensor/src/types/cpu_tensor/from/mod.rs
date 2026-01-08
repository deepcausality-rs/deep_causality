/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::InternalCpuTensor;

impl<T: Clone> From<T> for InternalCpuTensor<T> {
    /// Creates a scalar tensor (0-dimensional) from a single value.
    fn from(item: T) -> Self {
        InternalCpuTensor::from_vec_and_shape_unchecked(vec![item], &[])
    }
}

impl<'a, T: Clone> From<&'a T> for InternalCpuTensor<T> {
    /// Creates a scalar tensor (0-dimensional) from a reference to a single value.
    fn from(item: &'a T) -> Self {
        InternalCpuTensor::from_vec_and_shape_unchecked(vec![item.clone()], &[])
    }
}

impl<'a, T: Clone> From<&'a InternalCpuTensor<T>> for InternalCpuTensor<T> {
    /// Creates a new `InternalCpuTensor` by cloning an existing reference.
    fn from(item: &'a InternalCpuTensor<T>) -> Self {
        item.clone()
    }
}
