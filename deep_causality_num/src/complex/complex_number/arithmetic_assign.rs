/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Complex, ComplexNumber, Float};
use std::ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign};

// Implement AddAssign trait
impl<F> AddAssign for Complex<F>
where
    F: Float + AddAssign,
{
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

// Implement AddAssign<F> for Complex<F>
impl<F> AddAssign<F> for Complex<F>
where
    F: Float + AddAssign,
{
    #[inline]
    fn add_assign(&mut self, rhs: F) {
        self.re += rhs;
    }
}

// Implement SubAssign trait
impl<F> SubAssign for Complex<F>
where
    F: Float + SubAssign,
{
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.re -= rhs.re;
        self.im -= rhs.im;
    }
}

// Implement SubAssign<F> for Complex<F>
impl<F> SubAssign<F> for Complex<F>
where
    F: Float + SubAssign,
{
    #[inline]
    fn sub_assign(&mut self, rhs: F) {
        self.re -= rhs;
    }
}

// Implement MulAssign trait
impl<F> MulAssign for Complex<F>
where
    F: Float,
{
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        let re = self.re * rhs.re - self.im * rhs.im;
        let im = self.re * rhs.im + self.im * rhs.re;
        self.re = re;
        self.im = im;
    }
}

// Implement MulAssign<F> for Complex<F>
impl<F> MulAssign<F> for Complex<F>
where
    F: Float + MulAssign,
{
    #[inline]
    fn mul_assign(&mut self, rhs: F) {
        self.re *= rhs;
        self.im *= rhs;
    }
}

// Implement DivAssign trait
impl<F> DivAssign for Complex<F>
where
    F: Float,
{
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        let denom = rhs.norm_sqr();
        if denom.is_zero() {
            self.re = F::nan();
            self.im = F::nan();
        } else {
            let re = (self.re * rhs.re + self.im * rhs.im) / denom;
            let im = (self.im * rhs.re - self.re * rhs.im) / denom;
            self.re = re;
            self.im = im;
        }
    }
}

// Implement DivAssign<F> for Complex<F>
impl<F> DivAssign<F> for Complex<F>
where
    F: Float + DivAssign,
{
    #[inline]
    fn div_assign(&mut self, rhs: F) {
        if rhs.is_zero() {
            self.re = F::nan();
            self.im = F::nan();
        } else {
            self.re /= rhs;
            self.im /= rhs;
        }
    }
}

// Implement RemAssign trait
impl<F> RemAssign for Complex<F>
where
    F: Float + RemAssign,
{
    #[inline]
    fn rem_assign(&mut self, rhs: Self) {
        self.re %= rhs.re;
        self.im %= rhs.im;
    }
}

// Implement RemAssign<F> for Complex<F>
impl<F> RemAssign<F> for Complex<F>
where
    F: Float + RemAssign,
{
    #[inline]
    fn rem_assign(&mut self, rhs: F) {
        self.re %= rhs;
        self.im %= rhs;
    }
}
