/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Field;
use core::cmp::PartialOrd;

/// A Real Field represents a Field that also supports analysis/calculus operations
/// and ordering.
///
/// It abstracts over f32, f64, f128, and potentially Dual numbers (for gradients).
pub trait RealField: Field + PartialOrd {
    fn sqrt(self) -> Self;
    fn abs(self) -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn exp(self) -> Self;
    fn ln(self) -> Self;
    fn log(self, base: Self) -> Self;
    fn powf(self, n: Self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn tanh(self) -> Self;
    fn atan2(self, other: Self) -> Self;
    // Constants
    fn pi() -> Self;
    fn e() -> Self;
}

impl RealField for f32 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }
    fn abs(self) -> Self {
        self.abs()
    }
    fn floor(self) -> Self {
        self.floor()
    }
    fn ceil(self) -> Self {
        self.ceil()
    }
    fn round(self) -> Self {
        self.round()
    }
    fn exp(self) -> Self {
        self.exp()
    }
    fn ln(self) -> Self {
        self.ln()
    }
    fn log(self, base: Self) -> Self {
        self.log(base)
    }
    fn powf(self, n: Self) -> Self {
        self.powf(n)
    }
    fn sin(self) -> Self {
        self.sin()
    }
    fn cos(self) -> Self {
        self.cos()
    }
    fn tan(self) -> Self {
        self.tan()
    }
    fn tanh(self) -> Self {
        self.tanh()
    }
    fn atan2(self, other: Self) -> Self {
        self.atan2(other)
    }
    fn pi() -> Self {
        std::f32::consts::PI
    }
    fn e() -> Self {
        std::f32::consts::E
    }
}

impl RealField for f64 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }
    fn abs(self) -> Self {
        self.abs()
    }
    fn floor(self) -> Self {
        self.floor()
    }
    fn ceil(self) -> Self {
        self.ceil()
    }
    fn round(self) -> Self {
        self.round()
    }
    fn exp(self) -> Self {
        self.exp()
    }
    fn ln(self) -> Self {
        self.ln()
    }
    fn log(self, base: Self) -> Self {
        self.log(base)
    }
    fn powf(self, n: Self) -> Self {
        self.powf(n)
    }
    fn sin(self) -> Self {
        self.sin()
    }
    fn cos(self) -> Self {
        self.cos()
    }
    fn tan(self) -> Self {
        self.tan()
    }
    fn tanh(self) -> Self {
        self.tanh()
    }
    fn atan2(self, other: Self) -> Self {
        self.atan2(other)
    }
    fn pi() -> Self {
        std::f64::consts::PI
    }
    fn e() -> Self {
        std::f64::consts::E
    }
}
