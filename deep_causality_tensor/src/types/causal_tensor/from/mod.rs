/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CpuTensor;

impl<T: Clone> From<T> for CpuTensor<T> {
    /// Creates a scalar tensor (0-dimensional) from a single value.
    fn from(item: T) -> Self {
        CpuTensor::from_vec_and_shape_unchecked(vec![item], &[])
    }
}

impl<'a, T: Clone> From<&'a T> for CpuTensor<T> {
    /// Creates a scalar tensor (0-dimensional) from a reference to a single value.
    fn from(item: &'a T) -> Self {
        CpuTensor::from_vec_and_shape_unchecked(vec![item.clone()], &[])
    }
}

impl<'a, T: Clone> From<&'a CpuTensor<T>> for CpuTensor<T> {
    /// Creates a new `CpuTensor` by cloning an existing `CpuTensor` reference.
    fn from(item: &'a CpuTensor<T>) -> Self {
        item.clone()
    }
}
