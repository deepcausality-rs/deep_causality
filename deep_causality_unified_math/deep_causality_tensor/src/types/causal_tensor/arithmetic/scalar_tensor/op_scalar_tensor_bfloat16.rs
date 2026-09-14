/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `BFloat16` on the left of a scalar-tensor operation.
//!
//! `CausalTensor<T> op T` is generic in `T` (see the `tensor_scalar` module), but `T op
//! CausalTensor<T>` cannot be: a blanket `impl<T> Mul<CausalTensor<T>> for T` puts an uncovered
//! type parameter in the self position, which the orphan rule forbids. Each scalar therefore gets
//! its own impls, and this file is `BFloat16`'s.
//!
//! Without it, `BFloat16` is the odd one out among the shipped real fields: a program written against
//! a `FloatType` alias compiles with `scalar * tensor` at `f32` and `f64` and stops compiling the
//! day the alias names a software scalar. That is the failure precision-as-a-parameter exists to
//! remove.

use crate::types::causal_tensor::CausalTensor;
use core::ops::{Add, Div, Mul, Sub};
use deep_causality_num::BFloat16;

// ############################################################################
// ############################### ADDITION ###################################
// ############################################################################

/// `BFloat16 + &CausalTensor<BFloat16>`
impl<'a> Add<&'a CausalTensor<BFloat16>> for BFloat16 {
    type Output = CausalTensor<BFloat16>;

    fn add(self, rhs: &'a CausalTensor<BFloat16>) -> Self::Output {
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

/// `BFloat16 + CausalTensor<BFloat16>` (consuming)
impl Add<CausalTensor<BFloat16>> for BFloat16 {
    type Output = CausalTensor<BFloat16>;

    fn add(self, rhs: CausalTensor<BFloat16>) -> Self::Output {
        self.add(&rhs)
    }
}

// ############################################################################
// ############################### SUBTRACTION ###################################
// ############################################################################

/// `BFloat16 - &CausalTensor<BFloat16>`
impl<'a> Sub<&'a CausalTensor<BFloat16>> for BFloat16 {
    type Output = CausalTensor<BFloat16>;

    fn sub(self, rhs: &'a CausalTensor<BFloat16>) -> Self::Output {
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

/// `BFloat16 - CausalTensor<BFloat16>` (consuming)
impl Sub<CausalTensor<BFloat16>> for BFloat16 {
    type Output = CausalTensor<BFloat16>;

    fn sub(self, rhs: CausalTensor<BFloat16>) -> Self::Output {
        self.sub(&rhs)
    }
}

// ############################################################################
// ############################### MULTIPLICATION ###################################
// ############################################################################

/// `BFloat16 * &CausalTensor<BFloat16>`
impl<'a> Mul<&'a CausalTensor<BFloat16>> for BFloat16 {
    type Output = CausalTensor<BFloat16>;

    fn mul(self, rhs: &'a CausalTensor<BFloat16>) -> Self::Output {
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

/// `BFloat16 * CausalTensor<BFloat16>` (consuming)
impl Mul<CausalTensor<BFloat16>> for BFloat16 {
    type Output = CausalTensor<BFloat16>;

    fn mul(self, rhs: CausalTensor<BFloat16>) -> Self::Output {
        self.mul(&rhs)
    }
}

// ############################################################################
// ############################### DIVISION ###################################
// ############################################################################

/// `BFloat16 / &CausalTensor<BFloat16>`
impl<'a> Div<&'a CausalTensor<BFloat16>> for BFloat16 {
    type Output = CausalTensor<BFloat16>;

    fn div(self, rhs: &'a CausalTensor<BFloat16>) -> Self::Output {
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

/// `BFloat16 / CausalTensor<BFloat16>` (consuming)
impl Div<CausalTensor<BFloat16>> for BFloat16 {
    type Output = CausalTensor<BFloat16>;

    fn div(self, rhs: CausalTensor<BFloat16>) -> Self::Output {
        self.div(&rhs)
    }
}
