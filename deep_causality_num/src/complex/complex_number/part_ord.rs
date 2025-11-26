/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Complex, ComplexNumber, Float};

impl<F> PartialOrd for Complex<F>
where
    F: Float + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        match self.norm().partial_cmp(&other.norm()) {
            Some(core::cmp::Ordering::Equal) => match self.re.partial_cmp(&other.re) {
                Some(core::cmp::Ordering::Equal) => self.im.partial_cmp(&other.im),
                other => other,
            },
            other => other,
        }
    }
}
