/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Arithmetic that propagates absence.
//!
//! Presence is conjunctive: a result is present exactly when both operands are, so the presence
//! channels meet under `&` while the value channels do the arithmetic. Negation touches only the
//! value, because negating an absent quantity leaves it absent.
//!
//! These were on `MaybeUncertain<f64>` alone, which made the operators a property of one
//! precision. They are one impl each now.

use crate::MaybeUncertain;
use crate::UncertainScalar;
use std::ops::{Add, Div, Mul, Neg, Sub};

impl<R: UncertainScalar> Add for MaybeUncertain<R> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            is_present: self.is_present & rhs.is_present,
            value: self.value + rhs.value,
        }
    }
}

impl<R: UncertainScalar> Sub for MaybeUncertain<R> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            is_present: self.is_present & rhs.is_present,
            value: self.value - rhs.value,
        }
    }
}

impl<R: UncertainScalar> Mul for MaybeUncertain<R> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            is_present: self.is_present & rhs.is_present,
            value: self.value * rhs.value,
        }
    }
}

impl<R: UncertainScalar> Div for MaybeUncertain<R> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            is_present: self.is_present & rhs.is_present,
            value: self.value / rhs.value,
        }
    }
}

impl<R: UncertainScalar> Neg for MaybeUncertain<R> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            is_present: self.is_present,
            value: -self.value,
        }
    }
}
