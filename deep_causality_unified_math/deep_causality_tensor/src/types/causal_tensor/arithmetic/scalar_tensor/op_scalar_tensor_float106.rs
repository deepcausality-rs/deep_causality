/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Float106` on the left of a scalar-tensor operation.
//!
//! `CausalTensor<T> op T` is generic in `T` (see the `tensor_scalar` module), but `T op
//! CausalTensor<T>` cannot be: a blanket `impl<T> Mul<CausalTensor<T>> for T` puts an uncovered
//! type parameter in the self position, which the orphan rule forbids. Each scalar therefore gets
//! its own impls, and this file is `Float106`'s.
//!
//! Without it, `Float106` is the odd one out among the shipped real fields: a program written against
//! a `FloatType` alias compiles with `scalar * tensor` at `f32` and `f64` and stops compiling the
//! day the alias names a software scalar. That is the failure precision-as-a-parameter exists to
//! remove.

use crate::types::causal_tensor::CausalTensor;
use core::ops::{Add, Div, Mul, Sub};
use deep_causality_num::Float106;

// ############################################################################
// ############################### ADDITION ###################################
// ############################################################################

/// `Float106 + &CausalTensor<Float106>`
impl<'a> Add<&'a CausalTensor<Float106>> for Float106 {
    type Output = CausalTensor<Float106>;

    fn add(self, rhs: &'a CausalTensor<Float106>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = Add::add(self, *item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

/// `Float106 + CausalTensor<Float106>` (consuming)
impl Add<CausalTensor<Float106>> for Float106 {
    type Output = CausalTensor<Float106>;

    fn add(self, rhs: CausalTensor<Float106>) -> Self::Output {
        self.add(&rhs)
    }
}

// ############################################################################
// ############################### SUBTRACTION ###################################
// ############################################################################

/// `Float106 - &CausalTensor<Float106>`
impl<'a> Sub<&'a CausalTensor<Float106>> for Float106 {
    type Output = CausalTensor<Float106>;

    fn sub(self, rhs: &'a CausalTensor<Float106>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = Sub::sub(self, *item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

/// `Float106 - CausalTensor<Float106>` (consuming)
impl Sub<CausalTensor<Float106>> for Float106 {
    type Output = CausalTensor<Float106>;

    fn sub(self, rhs: CausalTensor<Float106>) -> Self::Output {
        self.sub(&rhs)
    }
}

// ############################################################################
// ############################### MULTIPLICATION ###################################
// ############################################################################

/// `Float106 * &CausalTensor<Float106>`
impl<'a> Mul<&'a CausalTensor<Float106>> for Float106 {
    type Output = CausalTensor<Float106>;

    fn mul(self, rhs: &'a CausalTensor<Float106>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = Mul::mul(self, *item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

/// `Float106 * CausalTensor<Float106>` (consuming)
impl Mul<CausalTensor<Float106>> for Float106 {
    type Output = CausalTensor<Float106>;

    fn mul(self, rhs: CausalTensor<Float106>) -> Self::Output {
        self.mul(&rhs)
    }
}

// ############################################################################
// ############################### DIVISION ###################################
// ############################################################################

/// `Float106 / &CausalTensor<Float106>`
impl<'a> Div<&'a CausalTensor<Float106>> for Float106 {
    type Output = CausalTensor<Float106>;

    fn div(self, rhs: &'a CausalTensor<Float106>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = Div::div(self, *item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

/// `Float106 / CausalTensor<Float106>` (consuming)
impl Div<CausalTensor<Float106>> for Float106 {
    type Output = CausalTensor<Float106>;

    fn div(self, rhs: CausalTensor<Float106>) -> Self::Output {
        self.div(&rhs)
    }
}
