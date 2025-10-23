/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Complex, Float, NumCast, ToPrimitive};

impl<F> NumCast for Complex<F>
where
    F: Float + NumCast,
{
    #[inline]
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        F::from(n).and_then(|re| {
            if re.is_nan() {
                None
            } else {
                Some(Self::new(re, F::zero()))
            }
        })
    }
}
