/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CausalTensorError;
use crate::InternalCpuTensor;
use deep_causality_num::Zero;
use std::ops::{Add, Div, Mul, Sub};

//
// Implement Add trait for InternalCpuTensor
//
impl<T> Add for &InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Add<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn add(self, rhs: &InternalCpuTensor<T>) -> Self::Output {
        self.broadcast_op(rhs, |a, b| Ok(a + b))
            .expect("Broadcast failed in Add")
    }
}

// T + &T
impl<T> Add<&InternalCpuTensor<T>> for InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Add<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn add(self, rhs: &InternalCpuTensor<T>) -> Self::Output {
        (&self).add(rhs)
    }
}

// &T + T
impl<T> Add<InternalCpuTensor<T>> for &InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Add<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn add(self, rhs: InternalCpuTensor<T>) -> Self::Output {
        self.add(&rhs)
    }
}

impl<T> Add for InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Add<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn add(self, rhs: InternalCpuTensor<T>) -> Self::Output {
        (&self).add(&rhs)
    }
}

//
// Implement Sub trait for InternalCpuTensor
//
impl<T> Sub for &InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Sub<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn sub(self, rhs: &InternalCpuTensor<T>) -> Self::Output {
        self.broadcast_op(rhs, |a, b| Ok(a - b))
            .expect("Broadcast failed in Sub")
    }
}

// T + &T
impl<T> Sub<&InternalCpuTensor<T>> for InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Sub<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn sub(self, rhs: &InternalCpuTensor<T>) -> Self::Output {
        (&self).sub(rhs)
    }
}

// &T + T
impl<T> Sub<InternalCpuTensor<T>> for &InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Sub<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn sub(self, rhs: InternalCpuTensor<T>) -> Self::Output {
        self.sub(&rhs)
    }
}

impl<T> Sub for InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Sub<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn sub(self, rhs: InternalCpuTensor<T>) -> Self::Output {
        (&self).sub(&rhs)
    }
}

//
// Implement Mul trait for InternalCpuTensor
//
impl<T> Mul for &InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Mul<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn mul(self, rhs: &InternalCpuTensor<T>) -> Self::Output {
        self.broadcast_op(rhs, |a, b| Ok(a * b))
            .expect("Broadcast failed in Mul")
    }
}

// T + &T
impl<T> Mul<&InternalCpuTensor<T>> for InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Mul<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn mul(self, rhs: &InternalCpuTensor<T>) -> Self::Output {
        (&self).mul(rhs)
    }
}

// &T + T
impl<T> Mul<InternalCpuTensor<T>> for &InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Mul<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn mul(self, rhs: InternalCpuTensor<T>) -> Self::Output {
        self.mul(&rhs)
    }
}

impl<T> Mul for InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Mul<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn mul(self, rhs: InternalCpuTensor<T>) -> Self::Output {
        (&self).mul(&rhs)
    }
}

//
// Implement Div trait for InternalCpuTensor
//
impl<T> Div for &InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Zero + Div<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn div(self, rhs: &InternalCpuTensor<T>) -> Self::Output {
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
impl<T> Div<&InternalCpuTensor<T>> for InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Zero + Div<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn div(self, rhs: &InternalCpuTensor<T>) -> Self::Output {
        (&self).div(rhs)
    }
}

// &T + T
impl<T> Div<InternalCpuTensor<T>> for &InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Zero + Div<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn div(self, rhs: InternalCpuTensor<T>) -> Self::Output {
        self.div(&rhs)
    }
}

impl<T> Div for InternalCpuTensor<T>
where
    T: Clone + Default + PartialOrd + Zero + Div<T, Output = T>,
{
    type Output = InternalCpuTensor<T>;

    fn div(self, rhs: InternalCpuTensor<T>) -> Self::Output {
        (&self).div(&rhs)
    }
}
