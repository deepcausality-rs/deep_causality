/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CausalTensorError;
use crate::types::causal_tensor::CpuTensor;
use deep_causality_num::Zero;
use std::ops::{Add, Div, Mul, Sub};

//
// Implement Add trait for CpuTensor
//
impl<T> Add for &CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Add<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn add(self, rhs: &CpuTensor<T>) -> Self::Output {
        self.broadcast_op(rhs, |a, b| Ok(a + b))
            .expect("Broadcast failed in Add")
    }
}

// T + &T
impl<T> Add<&CpuTensor<T>> for CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Add<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn add(self, rhs: &CpuTensor<T>) -> Self::Output {
        (&self).add(rhs)
    }
}

// &T + T
impl<T> Add<CpuTensor<T>> for &CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Add<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn add(self, rhs: CpuTensor<T>) -> Self::Output {
        self.add(&rhs)
    }
}

impl<T> Add for CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Add<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn add(self, rhs: CpuTensor<T>) -> Self::Output {
        (&self).add(&rhs)
    }
}

//
// Implement Sub trait for CpuTensor
//
impl<T> Sub for &CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Sub<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn sub(self, rhs: &CpuTensor<T>) -> Self::Output {
        self.broadcast_op(rhs, |a, b| Ok(a - b))
            .expect("Broadcast failed in Sub")
    }
}

// T + &T
impl<T> Sub<&CpuTensor<T>> for CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Sub<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn sub(self, rhs: &CpuTensor<T>) -> Self::Output {
        (&self).sub(rhs)
    }
}

// &T + T
impl<T> Sub<CpuTensor<T>> for &CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Sub<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn sub(self, rhs: CpuTensor<T>) -> Self::Output {
        self.sub(&rhs)
    }
}

impl<T> Sub for CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Sub<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn sub(self, rhs: CpuTensor<T>) -> Self::Output {
        (&self).sub(&rhs)
    }
}

//
// Implement Mul trait for CpuTensor
//
impl<T> Mul for &CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Mul<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn mul(self, rhs: &CpuTensor<T>) -> Self::Output {
        self.broadcast_op(rhs, |a, b| Ok(a * b))
            .expect("Broadcast failed in Mul")
    }
}

// T + &T
impl<T> Mul<&CpuTensor<T>> for CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Mul<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn mul(self, rhs: &CpuTensor<T>) -> Self::Output {
        (&self).mul(rhs)
    }
}

// &T + T
impl<T> Mul<CpuTensor<T>> for &CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Mul<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn mul(self, rhs: CpuTensor<T>) -> Self::Output {
        self.mul(&rhs)
    }
}

impl<T> Mul for CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Mul<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn mul(self, rhs: CpuTensor<T>) -> Self::Output {
        (&self).mul(&rhs)
    }
}

//
// Implement Div trait for CpuTensor
//
impl<T> Div for &CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Zero + Div<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn div(self, rhs: &CpuTensor<T>) -> Self::Output {
        self.broadcast_op(rhs, |a, b| {
            if b == T::zero() {
                Err(CausalTensorError::InvalidOperation)
            } else {
                Ok(a / b)
            }
        })
        .expect("Broadcast failed or division by zero in Div")
    }
}

// T + &T
impl<T> Div<&CpuTensor<T>> for CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Zero + Div<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn div(self, rhs: &CpuTensor<T>) -> Self::Output {
        (&self).div(rhs)
    }
}

// &T + T
impl<T> Div<CpuTensor<T>> for &CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Zero + Div<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn div(self, rhs: CpuTensor<T>) -> Self::Output {
        self.div(&rhs)
    }
}

impl<T> Div for CpuTensor<T>
where
    T: Clone + Default + PartialOrd + Zero + Div<T, Output = T>,
{
    type Output = CpuTensor<T>;

    fn div(self, rhs: CpuTensor<T>) -> Self::Output {
        (&self).div(&rhs)
    }
}
