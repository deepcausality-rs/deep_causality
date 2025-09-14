/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensor, CausalTensorError, Zero};
use std::ops::{Add, Div, Mul, Sub};

//
// Implement Add trait for CausalTensor
//
impl<T> Add for &CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Add<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn add(self, rhs: &CausalTensor<T>) -> Self::Output {
        self.broadcast_op(rhs, |a, b| Ok(a + b))
    }
}

// T + &T
impl<T> Add<&CausalTensor<T>> for CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Add<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn add(self, rhs: &CausalTensor<T>) -> Self::Output {
        (&self).add(rhs)
    }
}

// &T + T
impl<T> Add<CausalTensor<T>> for &CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Add<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn add(self, rhs: CausalTensor<T>) -> Self::Output {
        self.add(&rhs)
    }
}

impl<T> Add for CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Add<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn add(self, rhs: CausalTensor<T>) -> Self::Output {
        (&self).add(&rhs)
    }
}

//
// Implement Sub trait for CausalTensor
//
impl<T> Sub for &CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Sub<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn sub(self, rhs: &CausalTensor<T>) -> Self::Output {
        self.broadcast_op(rhs, |a, b| Ok(a - b))
    }
}

// T + &T
impl<T> Sub<&CausalTensor<T>> for CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Sub<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn sub(self, rhs: &CausalTensor<T>) -> Self::Output {
        (&self).sub(rhs)
    }
}

// &T + T
impl<T> Sub<CausalTensor<T>> for &CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Sub<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn sub(self, rhs: CausalTensor<T>) -> Self::Output {
        self.sub(&rhs)
    }
}

impl<T> Sub for CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Sub<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn sub(self, rhs: CausalTensor<T>) -> Self::Output {
        (&self).sub(&rhs)
    }
}

//
// Implement Mul trait for CausalTensor
//
impl<T> Mul for &CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Mul<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn mul(self, rhs: &CausalTensor<T>) -> Self::Output {
        self.broadcast_op(rhs, |a, b| Ok(a * b))
    }
}

// T + &T
impl<T> Mul<&CausalTensor<T>> for CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Mul<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn mul(self, rhs: &CausalTensor<T>) -> Self::Output {
        (&self).mul(rhs)
    }
}

// &T + T
impl<T> Mul<CausalTensor<T>> for &CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Mul<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn mul(self, rhs: CausalTensor<T>) -> Self::Output {
        self.mul(&rhs)
    }
}

impl<T> Mul for CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Mul<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn mul(self, rhs: CausalTensor<T>) -> Self::Output {
        (&self).mul(&rhs)
    }
}

//
// Implement Div trait for CausalTensor
//
impl<T> Div for &CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Zero + Div<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn div(self, rhs: &CausalTensor<T>) -> Self::Output {
        self.broadcast_op(rhs, |a, b| {
            if b == T::zero() {
                Err(CausalTensorError::InvalidOperation)
            } else {
                Ok(a / b)
            }
        })
    }
}

// T + &T
impl<T> Div<&CausalTensor<T>> for CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Zero + Div<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn div(self, rhs: &CausalTensor<T>) -> Self::Output {
        (&self).div(rhs)
    }
}

// &T + T
impl<T> Div<CausalTensor<T>> for &CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Zero + Div<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn div(self, rhs: CausalTensor<T>) -> Self::Output {
        self.div(&rhs)
    }
}

impl<T> Div for CausalTensor<T>
where
    T: Copy + Default + PartialOrd + Zero + Div<T, Output = T>,
{
    type Output = Result<CausalTensor<T>, CausalTensorError>;

    fn div(self, rhs: CausalTensor<T>) -> Self::Output {
        (&self).div(&rhs)
    }
}
