/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::MaybeUncertain;
use std::ops::{Add, Div, Mul, Neg, Sub};

// Ops traits implementations
impl Add for MaybeUncertain<f64> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            is_present: self.is_present & rhs.is_present,
            value: self.value + rhs.value,
        }
    }
}

impl Sub for MaybeUncertain<f64> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            is_present: self.is_present & rhs.is_present,
            value: self.value - rhs.value,
        }
    }
}

impl Mul for MaybeUncertain<f64> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            is_present: self.is_present & rhs.is_present,
            value: self.value * rhs.value,
        }
    }
}

impl Div for MaybeUncertain<f64> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            is_present: self.is_present & rhs.is_present,
            value: self.value / rhs.value,
        }
    }
}

impl Neg for MaybeUncertain<f64> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            is_present: self.is_present,
            value: -self.value,
        }
    }
}
