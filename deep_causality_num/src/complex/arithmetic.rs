/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Complex, ComplexNumber, Float, Num};
use std::iter::{Product, Sum};
use std::ops::{Add, Div, Mul, Rem, Sub};

impl<F> Num for Complex<F> where F: Float {}

// Implement Sum trait
impl<F> Sum for Complex<F>
where
    F: Float,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::new(F::zero(), F::zero()), |acc, x| acc + x)
    }
}

// Implement Product trait
impl<F> Product for Complex<F>
where
    F: Float,
{
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::new(F::one(), F::zero()), |acc, x| acc * x)
    }
}

// Implement Add trait
impl<F> Add for Complex<F>
where
    F: Float,
{
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.re + rhs.re, self.im + rhs.im)
    }
}

// Implement Add<F> for Complex<F>
impl<F> Add<F> for Complex<F>
where
    F: Float,
{
    type Output = Self;

    #[inline]
    fn add(self, rhs: F) -> Self::Output {
        Self::new(self.re + rhs, self.im)
    }
}

// Implement Sub trait
impl<F> Sub for Complex<F>
where
    F: Float,
{
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.re - rhs.re, self.im - rhs.im)
    }
}

// Implement Sub<F> for Complex<F>
impl<F> Sub<F> for Complex<F>
where
    F: Float,
{
    type Output = Self;

    #[inline]
    fn sub(self, rhs: F) -> Self::Output {
        Self::new(self.re - rhs, self.im)
    }
}

// Implement Mul trait
impl<F> Mul for Complex<F>
where
    F: Float,
{
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        // (a + bi) * (c + di) = (ac - bd) + (ad + bc)i
        Self::new(
            self.re * rhs.re - self.im * rhs.im,
            self.re * rhs.im + self.im * rhs.re,
        )
    }
}

// Implement Mul<F> for Complex<F>
impl<F> Mul<F> for Complex<F>
where
    F: Float,
{
    type Output = Self;

    #[inline]
    fn mul(self, rhs: F) -> Self::Output {
        Self::new(self.re * rhs, self.im * rhs)
    }
}

// Implement Div trait
impl<F> Div for Complex<F>
where
    F: Float,
{
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        // (a + bi) / (c + di) = [(ac + bd) / (c^2 + d^2)] + [(bc - ad) / (c^2 + d^2)]i
        let denom = rhs.norm_sqr();
        if denom.is_zero() {
            // Handle division by zero, return NaN complex number
            Self::new(F::nan(), F::nan())
        } else {
            Self::new(
                (self.re * rhs.re + self.im * rhs.im) / denom,
                (self.im * rhs.re - self.re * rhs.im) / denom,
            )
        }
    }
}

// Implement Div<F> for Complex<F>
impl<F> Div<F> for Complex<F>
where
    F: Float,
{
    type Output = Self;

    #[inline]
    fn div(self, rhs: F) -> Self::Output {
        if rhs.is_zero() {
            // Handle division by zero, return NaN complex number
            Self::new(F::nan(), F::nan())
        } else {
            Self::new(self.re / rhs, self.im / rhs)
        }
    }
}

// Implement Rem trait (modulo for complex numbers is not standard,
// but we can define it as component-wise modulo for consistency with NumOps)
impl<F> Rem for Complex<F>
where
    F: Float,
{
    type Output = Self;

    #[inline]
    fn rem(self, rhs: Self) -> Self::Output {
        let re = if rhs.re.is_zero() {
            F::nan()
        } else {
            self.re % rhs.re
        };
        let im = if rhs.im.is_zero() {
            F::nan()
        } else {
            self.im % rhs.im
        };
        Self::new(re, im)
    }
}

// Implement Rem<F> for Complex<F>
impl<F> Rem<F> for Complex<F>
where
    F: Float,
{
    type Output = Self;

    #[inline]
    fn rem(self, rhs: F) -> Self::Output {
        let re = if rhs.is_zero() {
            F::nan()
        } else {
            self.re % rhs
        };
        let im = if rhs.is_zero() {
            F::nan()
        } else {
            self.im % rhs
        };
        Self::new(re, im)
    }
}
