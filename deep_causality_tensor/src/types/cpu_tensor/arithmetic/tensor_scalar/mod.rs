/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::InternalCpuTensor;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

// Implemented via `std::ops` traits for ergonomic use (`+`, `-`, `*`, `/`).

// --- ADDITION ---

// Implementation for `&InternalCpuTensor<T> + T`
impl<T> Add<T> for &InternalCpuTensor<T>
where
    T: Add<T, Output = T> + Clone,
{
    type Output = InternalCpuTensor<T>;

    fn add(self, rhs: T) -> Self::Output {
        let mut new_data = self.data.clone();
        for item in &mut new_data {
            *item = item.clone().add(rhs.clone());
        }
        InternalCpuTensor {
            data: new_data,
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }
}

// Implementation for `InternalCpuTensor<T> + T` (consuming)
impl<T> Add<T> for InternalCpuTensor<T>
where
    T: Add<T, Output = T> + Clone,
{
    type Output = InternalCpuTensor<T>;
    fn add(self, rhs: T) -> Self::Output {
        (&self).add(rhs)
    }
}

// Implementation for `InternalCpuTensor<T> += T` (in-place)
impl<T> AddAssign<T> for InternalCpuTensor<T>
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

// Implementation for `&InternalCpuTensor<T> - T`
impl<T> Sub<T> for &InternalCpuTensor<T>
where
    T: Sub<T, Output = T> + Clone,
{
    type Output = InternalCpuTensor<T>;

    fn sub(self, rhs: T) -> Self::Output {
        let mut new_data = self.data.clone();
        for item in &mut new_data {
            *item = item.clone().sub(rhs.clone());
        }
        InternalCpuTensor {
            data: new_data,
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }
}

// Implementation for `InternalCpuTensor<T> - T` (consuming)
impl<T> Sub<T> for InternalCpuTensor<T>
where
    T: Sub<T, Output = T> + Clone,
{
    type Output = InternalCpuTensor<T>;
    fn sub(self, rhs: T) -> Self::Output {
        (&self).sub(rhs)
    }
}

// Implementation for `InternalCpuTensor<T> -= T` (in-place)
impl<T> SubAssign<T> for InternalCpuTensor<T>
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

// Implementation for `&InternalCpuTensor<T> * T`
impl<T> Mul<T> for &InternalCpuTensor<T>
where
    T: Mul<T, Output = T> + Clone,
{
    type Output = InternalCpuTensor<T>;

    fn mul(self, rhs: T) -> Self::Output {
        let mut new_data = self.data.clone();
        for item in &mut new_data {
            *item = item.clone().mul(rhs.clone());
        }
        InternalCpuTensor {
            data: new_data,
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }
}

// Implementation for `InternalCpuTensor<T> * T` (consuming)
impl<T> Mul<T> for InternalCpuTensor<T>
where
    T: Mul<T, Output = T> + Clone,
{
    type Output = InternalCpuTensor<T>;
    fn mul(self, rhs: T) -> Self::Output {
        (&self).mul(rhs)
    }
}

// Implementation for `InternalCpuTensor<T> *= T` (in-place)
impl<T> MulAssign<T> for InternalCpuTensor<T>
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

// Implementation for `&InternalCpuTensor<T> / T`
impl<T> Div<T> for &InternalCpuTensor<T>
where
    T: Div<T, Output = T> + Clone,
{
    type Output = InternalCpuTensor<T>;

    fn div(self, rhs: T) -> Self::Output {
        let mut new_data = self.data.clone();
        for item in &mut new_data {
            *item = item.clone().div(rhs.clone());
        }
        InternalCpuTensor {
            data: new_data,
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }
}

// Implementation for `InternalCpuTensor<T> / T` (consuming)
impl<T> Div<T> for InternalCpuTensor<T>
where
    T: Div<T, Output = T> + Clone,
{
    type Output = InternalCpuTensor<T>;
    fn div(self, rhs: T) -> Self::Output {
        (&self).div(rhs)
    }
}

// Implementation for `InternalCpuTensor<T> /= T` (in-place)
impl<T> DivAssign<T> for InternalCpuTensor<T>
where
    T: DivAssign + Clone,
{
    fn div_assign(&mut self, rhs: T) {
        for item in &mut self.data {
            item.div_assign(rhs.clone());
        }
    }
}
