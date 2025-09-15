/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CausalTensor;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

// Implemented via `std::ops` traits for ergonomic use (`+`, `-`, `*`, `/`).

// --- ADDITION ---

// Implementation for `&CausalTensor<T> + T`
impl<T> Add<T> for &CausalTensor<T>
where
    T: Add<T, Output = T> + Copy,
{
    type Output = CausalTensor<T>;

    fn add(self, rhs: T) -> Self::Output {
        let mut new_data = self.data.clone();
        for item in &mut new_data {
            *item = (*item).add(rhs);
        }
        CausalTensor {
            data: new_data,
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }
}

// Implementation for `CausalTensor<T> + T` (consuming)
impl<T> Add<T> for CausalTensor<T>
where
    T: Add<T, Output = T> + Copy,
{
    type Output = CausalTensor<T>;
    fn add(self, rhs: T) -> Self::Output {
        (&self).add(rhs)
    }
}

// Implementation for `CausalTensor<T> += T` (in-place)
impl<T> AddAssign<T> for CausalTensor<T>
where
    T: AddAssign + Copy,
{
    fn add_assign(&mut self, rhs: T) {
        for item in &mut self.data {
            item.add_assign(rhs);
        }
    }
}

// --- SUBTRACTION ---

// Implementation for `&CausalTensor<T> - T`
impl<T> Sub<T> for &CausalTensor<T>
where
    T: Sub<T, Output = T> + Copy,
{
    type Output = CausalTensor<T>;

    fn sub(self, rhs: T) -> Self::Output {
        let mut new_data = self.data.clone();
        for item in &mut new_data {
            *item = (*item).sub(rhs);
        }
        CausalTensor {
            data: new_data,
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }
}

// Implementation for `CausalTensor<T> - T` (consuming)
impl<T> Sub<T> for CausalTensor<T>
where
    T: Sub<T, Output = T> + Copy,
{
    type Output = CausalTensor<T>;
    fn sub(self, rhs: T) -> Self::Output {
        (&self).sub(rhs)
    }
}

// Implementation for `CausalTensor<T> -= T` (in-place)
impl<T> SubAssign<T> for CausalTensor<T>
where
    T: SubAssign + Copy,
{
    fn sub_assign(&mut self, rhs: T) {
        for item in &mut self.data {
            item.sub_assign(rhs);
        }
    }
}

// --- MULTIPLICATION ---

// Implementation for `&CausalTensor<T> * T`
impl<T> Mul<T> for &CausalTensor<T>
where
    T: Mul<T, Output = T> + Copy,
{
    type Output = CausalTensor<T>;

    fn mul(self, rhs: T) -> Self::Output {
        let mut new_data = self.data.clone();
        for item in &mut new_data {
            *item = (*item).mul(rhs);
        }
        CausalTensor {
            data: new_data,
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }
}

// Implementation for `CausalTensor<T> * T` (consuming)
impl<T> Mul<T> for CausalTensor<T>
where
    T: Mul<T, Output = T> + Copy,
{
    type Output = CausalTensor<T>;
    fn mul(self, rhs: T) -> Self::Output {
        (&self).mul(rhs)
    }
}

// Implementation for `CausalTensor<T> *= T` (in-place)
impl<T> MulAssign<T> for CausalTensor<T>
where
    T: MulAssign + Copy,
{
    fn mul_assign(&mut self, rhs: T) {
        for item in &mut self.data {
            item.mul_assign(rhs);
        }
    }
}

// --- DIVISION ---

// Implementation for `&CausalTensor<T> / T`
impl<T> Div<T> for &CausalTensor<T>
where
    T: Div<T, Output = T> + Copy,
{
    type Output = CausalTensor<T>;

    fn div(self, rhs: T) -> Self::Output {
        let mut new_data = self.data.clone();
        for item in &mut new_data {
            *item = (*item).div(rhs);
        }
        CausalTensor {
            data: new_data,
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }
}

// Implementation for `CausalTensor<T> / T` (consuming)
impl<T> Div<T> for CausalTensor<T>
where
    T: Div<T, Output = T> + Copy,
{
    type Output = CausalTensor<T>;
    fn div(self, rhs: T) -> Self::Output {
        (&self).div(rhs)
    }
}

// Implementation for `CausalTensor<T> /= T` (in-place)
impl<T> DivAssign<T> for CausalTensor<T>
where
    T: DivAssign + Copy,
{
    fn div_assign(&mut self, rhs: T) {
        for item in &mut self.data {
            item.div_assign(rhs);
        }
    }
}
