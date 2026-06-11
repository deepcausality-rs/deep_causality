/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Add` and `Mul<R>` for [`VelocityOneForm`] — exactly the bounds the
//! `Rk4`/`Euler` arrows require of a marching state, and nothing more.

use core::ops::{Add, Mul};

use deep_causality_num::RealField;
use deep_causality_tensor::CausalTensor;

use super::VelocityOneForm;

impl<R: RealField> Add for VelocityOneForm<R> {
    type Output = Self;

    /// Element-wise sum of two velocity 1-forms.
    ///
    /// # Panics
    /// Panics when the operands carry different edge counts (fields from
    /// different lattices); validated construction makes this a programming
    /// error, not a runtime condition.
    fn add(self, rhs: Self) -> Self {
        assert_eq!(
            self.field.len(),
            rhs.field.len(),
            "VelocityOneForm + VelocityOneForm requires matching edge counts"
        );
        let data: alloc::vec::Vec<R> = self
            .field
            .as_slice()
            .iter()
            .zip(rhs.field.as_slice().iter())
            .map(|(a, b)| *a + *b)
            .collect();
        let len = data.len();
        Self {
            field: CausalTensor::new(data, alloc::vec![len])
                .expect("1-D tensor allocation cannot fail"),
        }
    }
}

impl<R: RealField> Mul<R> for VelocityOneForm<R> {
    type Output = Self;

    /// Scalar scaling of the whole field.
    fn mul(self, rhs: R) -> Self {
        let data: alloc::vec::Vec<R> = self.field.as_slice().iter().map(|a| *a * rhs).collect();
        let len = data.len();
        Self {
            field: CausalTensor::new(data, alloc::vec![len])
                .expect("1-D tensor allocation cannot fail"),
        }
    }
}
