/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CpuTensor;
use std::ops::Neg;

/// Implements Neg trait for CpuTensor.
impl<T> Neg for CpuTensor<T>
where
    T: Neg<Output = T> + Copy + Clone,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        let data = self.data.iter().map(|x| -*x).collect();
        Self::from_vec_and_shape_unchecked(data, self.shape())
    }
}

impl<T> Neg for &CpuTensor<T>
where
    T: Neg<Output = T> + Copy + Clone,
{
    type Output = CpuTensor<T>;

    fn neg(self) -> Self::Output {
        let data = self.data.iter().map(|x| -*x).collect();
        CpuTensor::from_vec_and_shape_unchecked(data, self.shape())
    }
}
