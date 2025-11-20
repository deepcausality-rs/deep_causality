/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::complex::octonion_number::Octonion;
use crate::float::Float;
use crate::{NumCast, ToPrimitive};

// NumCast
impl<F: Float> NumCast for Octonion<F> {
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        F::from(n).map(|f| {
            Octonion::new(
                f,
                F::zero(),
                F::zero(),
                F::zero(),
                F::zero(),
                F::zero(),
                F::zero(),
                F::zero(),
            )
        })
    }
}
