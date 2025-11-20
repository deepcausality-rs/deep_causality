/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::complex::octonion_number::{Octonion, OctonionNumber};

use crate::float::Float;

impl<F> OctonionNumber<F> for Octonion<F>
where
    F: Float,
{
    fn conjugate(&self) -> Self {
        Self {
            s: self.s,
            e1: -self.e1,
            e2: -self.e2,
            e3: -self.e3,
            e4: -self.e4,
            e5: -self.e5,
            e6: -self.e6,
            e7: -self.e7,
        }
    }

    fn norm_sqr(&self) -> F {
        self.s * self.s
            + self.e1 * self.e1
            + self.e2 * self.e2
            + self.e3 * self.e3
            + self.e4 * self.e4
            + self.e5 * self.e5
            + self.e6 * self.e6
            + self.e7 * self.e7
    }

    fn norm(&self) -> F {
        self.norm_sqr().sqrt()
    }

    fn normalize(&self) -> Self {
        let n = self.norm();
        if n.is_zero() { *self } else { *self / n }
    }

    fn inverse(&self) -> Self {
        let n_sqr = self.norm_sqr();
        if n_sqr.is_zero() {
            let nan = F::nan();
            Self::new(nan, nan, nan, nan, nan, nan, nan, nan)
        } else {
            self.conjugate() / n_sqr
        }
    }

    fn dot(&self, other: &Self) -> F {
        self.s * other.s
            + self.e1 * other.e1
            + self.e2 * other.e2
            + self.e3 * other.e3
            + self.e4 * other.e4
            + self.e5 * other.e5
            + self.e6 * other.e6
            + self.e7 * other.e7
    }
}
