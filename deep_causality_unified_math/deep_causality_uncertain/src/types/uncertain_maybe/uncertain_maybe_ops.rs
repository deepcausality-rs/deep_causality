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
use deep_causality_rand::RandScalar;
use std::ops::{Add, Div, Mul, Neg, Sub};

impl<R: RandScalar> Add for MaybeUncertain<R> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            is_present: self.is_present & rhs.is_present,
            value: self.value + rhs.value,
        }
    }
}

impl<R: RandScalar> Sub for MaybeUncertain<R> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            is_present: self.is_present & rhs.is_present,
            value: self.value - rhs.value,
        }
    }
}

impl<R: RandScalar> Mul for MaybeUncertain<R> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            is_present: self.is_present & rhs.is_present,
            value: self.value * rhs.value,
        }
    }
}

impl<R: RandScalar> Div for MaybeUncertain<R> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            is_present: self.is_present & rhs.is_present,
            value: self.value / rhs.value,
        }
    }
}

impl<R: RandScalar> Neg for MaybeUncertain<R> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            is_present: self.is_present,
            value: -self.value,
        }
    }
}
