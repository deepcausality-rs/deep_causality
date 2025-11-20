/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AsPrimitive, Complex, Float};

impl<F> AsPrimitive<F> for Complex<F>
where
    F: Float + AsPrimitive<F>,
{
    #[inline]
    fn as_(self) -> F {
        self.re.as_()
    }
}
