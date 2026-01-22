/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Trait implementations for `DoubleFloat` that belong to other modules.
//!
//! This file centralizes implementations of:
//! - `Zero`, `One` (identity traits)
//! - `Num` (numeric trait)
//! - `AbelianGroup`, `DivisionAlgebra`, `RealField` (algebra traits)
//! - Marker traits (`Associative`, `Commutative`, `Distributive`)

use crate::Float106;
use crate::{
    AbelianGroup, Associative, Commutative, Distributive, DivisionAlgebra, Float, Num, One,
    RealField, Zero,
};

// =============================================================================
// Identity Traits
// =============================================================================

impl Zero for Float106 {
    #[inline]
    fn zero() -> Self {
        Self::from_f64(0.0)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.hi() == 0.0 && self.lo() == 0.0
    }
}

impl One for Float106 {
    #[inline]
    fn one() -> Self {
        Self::from_f64(1.0)
    }

    #[inline]
    fn is_one(&self) -> bool {
        self.hi() == 1.0 && self.lo() == 0.0
    }
}

// =============================================================================
// Numeric Trait
// =============================================================================

impl Num for Float106 {}

// =============================================================================
// Marker Traits (algebra properties)
// =============================================================================

impl Associative for Float106 {}
impl Commutative for Float106 {}
impl Distributive for Float106 {}

// =============================================================================
// Abelian Group (additive)
// =============================================================================

impl AbelianGroup for Float106 {}

// =============================================================================
// Division Algebra
// =============================================================================

impl DivisionAlgebra<Float106> for Float106 {
    /// The conjugate of a real number is itself.
    #[inline]
    fn conjugate(&self) -> Self {
        *self
    }

    /// The squared norm of a DoubleFloat `x` is `x*x`.
    #[inline]
    fn norm_sqr(&self) -> Float106 {
        *self * *self
    }

    /// The inverse of a DoubleFloat `x` is `1/x`.
    #[inline]
    fn inverse(&self) -> Self {
        Self::from_f64(1.0) / *self
    }
}

// Note: Assignment operators (AddAssign, etc.) are implemented in ops_arithmetic.rs

// =============================================================================
// Real Field
// =============================================================================

impl RealField for Float106 {
    #[inline]
    fn nan() -> Self {
        <Self as Float>::nan()
    }

    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }

    #[inline]
    fn sqrt(self) -> Self {
        <Self as Float>::sqrt(self)
    }

    #[inline]
    fn abs(self) -> Self {
        <Self as Float>::abs(self)
    }

    #[inline]
    fn floor(self) -> Self {
        <Self as Float>::floor(self)
    }

    #[inline]
    fn ceil(self) -> Self {
        <Self as Float>::ceil(self)
    }

    #[inline]
    fn round(self) -> Self {
        <Self as Float>::round(self)
    }

    #[inline]
    fn exp(self) -> Self {
        <Self as Float>::exp(self)
    }

    #[inline]
    fn ln(self) -> Self {
        <Self as Float>::ln(self)
    }

    #[inline]
    fn log(self, base: Self) -> Self {
        <Self as Float>::log(self, base)
    }

    #[inline]
    fn powf(self, n: Self) -> Self {
        <Self as Float>::powf(self, n)
    }

    #[inline]
    fn sin(self) -> Self {
        <Self as Float>::sin(self)
    }

    #[inline]
    fn asin(self) -> Self {
        <Self as Float>::asin(self)
    }

    #[inline]
    fn cos(self) -> Self {
        <Self as Float>::cos(self)
    }

    #[inline]
    fn acos(self) -> Self {
        <Self as Float>::acos(self)
    }

    #[inline]
    fn tan(self) -> Self {
        <Self as Float>::tan(self)
    }

    #[inline]
    fn sinh(self) -> Self {
        <Self as Float>::sinh(self)
    }

    #[inline]
    fn cosh(self) -> Self {
        <Self as Float>::cosh(self)
    }

    #[inline]
    fn tanh(self) -> Self {
        <Self as Float>::tanh(self)
    }

    #[inline]
    fn atan(self) -> Self {
        <Self as Float>::atan(self)
    }

    #[inline]
    fn atan2(self, other: Self) -> Self {
        <Self as Float>::atan2(self, other)
    }

    #[inline]
    fn pi() -> Self {
        Self::PI
    }

    #[inline]
    fn e() -> Self {
        Self::E
    }

    #[inline]
    fn epsilon() -> Self {
        Self::EPSILON
    }
}
