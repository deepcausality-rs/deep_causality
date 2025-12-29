/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::causal_tensor::CpuTensor;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

// Implemented via `std::ops` traits for ergonomic use (`+`, `-`, `*`, `/`).

// --- ADDITION ---

// Implementation for `&CpuTensor<T> + T`
impl<T> Add<T> for &CpuTensor<T>
where
    T: Add<T, Output = T> + Clone,
{
    type Output = CpuTensor<T>;

    fn add(self, rhs: T) -> Self::Output {
        let mut new_data = self.data.clone();
        for item in &mut new_data {
            *item = item.clone().add(rhs.clone());
        }
        CpuTensor {
            data: new_data,
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }
}

// Implementation for `CpuTensor<T> + T` (consuming)
impl<T> Add<T> for CpuTensor<T>
where
    T: Add<T, Output = T> + Clone,
{
    type Output = CpuTensor<T>;
    fn add(self, rhs: T) -> Self::Output {
        (&self).add(rhs)
    }
}

// Implementation for `CpuTensor<T> += T` (in-place)
impl<T> AddAssign<T> for CpuTensor<T>
where
    T: AddAssign + Clone,
{
    fn add_assign(&mut self, rhs: T) {
        for item in &mut self.data {
            item.add_assign(rhs.clone());
        }
    }
}

// --- SUBTRACTION ---

// Implementation for `&CpuTensor<T> - T`
impl<T> Sub<T> for &CpuTensor<T>
where
    T: Sub<T, Output = T> + Clone,
{
    type Output = CpuTensor<T>;

    fn sub(self, rhs: T) -> Self::Output {
        let mut new_data = self.data.clone();
        for item in &mut new_data {
            *item = item.clone().sub(rhs.clone());
        }
        CpuTensor {
            data: new_data,
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }
}

// Implementation for `CpuTensor<T> - T` (consuming)
impl<T> Sub<T> for CpuTensor<T>
where
    T: Sub<T, Output = T> + Clone,
{
    type Output = CpuTensor<T>;
    fn sub(self, rhs: T) -> Self::Output {
        (&self).sub(rhs)
    }
}

// Implementation for `CpuTensor<T> -= T` (in-place)
impl<T> SubAssign<T> for CpuTensor<T>
where
    T: SubAssign + Clone,
{
    fn sub_assign(&mut self, rhs: T) {
        for item in &mut self.data {
            item.sub_assign(rhs.clone());
        }
    }
}

// --- MULTIPLICATION ---

// Implementation for `&CpuTensor<T> * T`
impl<T> Mul<T> for &CpuTensor<T>
where
    T: Mul<T, Output = T> + Clone,
{
    type Output = CpuTensor<T>;

    fn mul(self, rhs: T) -> Self::Output {
        let mut new_data = self.data.clone();
        for item in &mut new_data {
            *item = item.clone().mul(rhs.clone());
        }
        CpuTensor {
            data: new_data,
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }
}

// Implementation for `CpuTensor<T> * T` (consuming)
impl<T> Mul<T> for CpuTensor<T>
where
    T: Mul<T, Output = T> + Clone,
{
    type Output = CpuTensor<T>;
    fn mul(self, rhs: T) -> Self::Output {
        (&self).mul(rhs)
    }
}

// Implementation for `CpuTensor<T> *= T` (in-place)
impl<T> MulAssign<T> for CpuTensor<T>
where
    T: MulAssign + Clone,
{
    fn mul_assign(&mut self, rhs: T) {
        for item in &mut self.data {
            item.mul_assign(rhs.clone());
        }
    }
}

// --- DIVISION ---

// Implementation for `&CpuTensor<T> / T`
impl<T> Div<T> for &CpuTensor<T>
where
    T: Div<T, Output = T> + Clone,
{
    type Output = CpuTensor<T>;

    fn div(self, rhs: T) -> Self::Output {
        let mut new_data = self.data.clone();
        for item in &mut new_data {
            *item = item.clone().div(rhs.clone());
        }
        CpuTensor {
            data: new_data,
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }
}

// Implementation for `CpuTensor<T> / T` (consuming)
impl<T> Div<T> for CpuTensor<T>
where
    T: Div<T, Output = T> + Clone,
{
    type Output = CpuTensor<T>;
    fn div(self, rhs: T) -> Self::Output {
        (&self).div(rhs)
    }
}

// Implementation for `CpuTensor<T> /= T` (in-place)
impl<T> DivAssign<T> for CpuTensor<T>
where
    T: DivAssign + Clone,
{
    fn div_assign(&mut self, rhs: T) {
        for item in &mut self.data {
            item.div_assign(rhs.clone());
        }
    }
}
