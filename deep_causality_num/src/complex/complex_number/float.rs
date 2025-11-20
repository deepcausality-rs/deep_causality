/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Complex, ComplexNumber, Float, FromPrimitive, One, Zero};
use std::num::FpCategory;

impl<F> Float for Complex<F>
where
    F: Float + FromPrimitive,
{
    #[inline]
    fn nan() -> Self {
        Self::new(F::nan(), F::nan())
    }

    #[inline]
    fn infinity() -> Self {
        Self::new(F::infinity(), F::zero())
    }

    #[inline]
    fn neg_infinity() -> Self {
        Self::new(F::neg_infinity(), F::zero())
    }

    #[inline]
    fn neg_zero() -> Self {
        Self::new(F::neg_zero(), F::neg_zero())
    }

    #[inline]
    fn min_value() -> Self {
        Self::new(F::min_value(), F::zero())
    }

    #[inline]
    fn min_positive_value() -> Self {
        Self::new(F::min_positive_value(), F::zero())
    }

    #[inline]
    fn epsilon() -> Self {
        Self::new(F::epsilon(), F::zero())
    }

    #[inline]
    fn max_value() -> Self {
        Self::new(F::max_value(), F::zero())
    }

    #[inline]
    fn is_nan(self) -> bool {
        self.re.is_nan() || self.im.is_nan()
    }

    #[inline]
    fn is_infinite(self) -> bool {
        self.re.is_infinite() || self.im.is_infinite()
    }

    #[inline]
    fn is_finite(self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }

    #[inline]
    fn is_normal(self) -> bool {
        self.re.is_normal() || self.im.is_normal()
    }

    #[inline]
    fn is_subnormal(self) -> bool {
        self.re.is_subnormal() || self.im.is_subnormal()
    }

    #[inline]
    fn classify(self) -> FpCategory {
        if self.is_nan() {
            FpCategory::Nan
        } else if self.is_infinite() {
            FpCategory::Infinite
        } else if self.is_subnormal() {
            FpCategory::Subnormal
        } else if self.is_normal() {
            FpCategory::Normal
        } else if self.is_zero() {
            FpCategory::Zero
        } else {
            FpCategory::Normal // Default or other cases
        }
    }

    #[inline]
    fn floor(self) -> Self {
        Self::new(self.re.floor(), self.im.floor())
    }

    #[inline]
    fn ceil(self) -> Self {
        Self::new(self.re.ceil(), self.im.ceil())
    }

    #[inline]
    fn round(self) -> Self {
        Self::new(self.re.round(), self.im.round())
    }

    #[inline]
    fn trunc(self) -> Self {
        Self::new(self.re.trunc(), self.im.trunc())
    }

    #[inline]
    fn fract(self) -> Self {
        Self::new(self.re.fract(), self.im.fract())
    }

    #[inline]
    fn abs(self) -> Self {
        Self::new(self.norm(), F::zero())
    }

    #[inline]
    fn signum(self) -> Self {
        if self.is_zero() {
            Self::nan()
        } else {
            self / self.norm()
        }
    }

    #[inline]
    fn is_sign_positive(self) -> bool {
        if !self.re.is_zero() {
            self.re.is_sign_positive()
        } else {
            self.im.is_sign_positive()
        }
    }

    #[inline]
    fn is_sign_negative(self) -> bool {
        if !self.re.is_zero() {
            self.re.is_sign_negative()
        } else {
            self.im.is_sign_negative()
        }
    }

    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self {
        self * a + b
    }

    #[inline]
    fn recip(self) -> Self {
        Self::one() / self
    }

    #[inline]
    fn powi(self, n: i32) -> Self {
        if n == 0 {
            return Self::one();
        }
        let mut res = Self::one();
        let mut base = self;
        let mut n_abs = n.abs();

        while n_abs > 0 {
            if n_abs % 2 == 1 {
                res *= base;
            }
            base = base * base;
            n_abs /= 2;
        }

        if n < 0 { Self::one() / res } else { res }
    }

    #[inline]
    fn powf(self, n: Self) -> Self {
        if self.is_zero() {
            return Self::zero();
        }
        // z^w = e^(w * ln(z))
        (n * self.ln()).exp()
    }

    #[inline]
    fn sqrt(self) -> Self {
        // Numerically stable principal sqrt for complex numbers
        // Reference: Higham, Accuracy and Stability of Numerical Algorithms
        if self.is_zero() {
            return Self::zero();
        }
        let re = self.re;
        let im = self.im;
        let norm = self.norm();
        let half = F::from(0.5).unwrap();
        let two = F::from(2.0).unwrap();

        if re.is_sign_positive() || re.is_zero() {
            // re >= 0
            let re_sqrt = ((norm + re) * half).sqrt();
            let im_sqrt = if re_sqrt.is_zero() {
                F::zero()
            } else {
                im / (two * re_sqrt)
            };
            Self::new(re_sqrt, im_sqrt)
        } else {
            // re < 0
            let im_sqrt = ((norm - re) * half).sqrt();
            let re_sqrt = if im_sqrt.is_zero() {
                F::zero()
            } else {
                im / (two * im_sqrt)
            };
            Self::new(re_sqrt, im_sqrt)
        }
    }

    #[inline]
    fn exp(self) -> Self {
        // e^(a + bi) = e^a * (cos(b) + i * sin(b))
        let exp_re = self.re.exp();
        Self::new(exp_re * self.im.cos(), exp_re * self.im.sin())
    }

    #[inline]
    fn exp2(self) -> Self {
        // 2^z = e^(z * ln(2))
        (self * F::from(2.0).unwrap().ln()).exp()
    }

    #[inline]
    fn ln(self) -> Self {
        // ln(z) = ln(norm) + i * arg
        Self::new(self.norm().ln(), self.arg())
    }

    #[inline]
    fn log(self, base: Self) -> Self {
        self.ln() / base.ln()
    }

    #[inline]
    fn log2(self) -> Self {
        self.ln() / F::from(2.0).unwrap().ln()
    }

    #[inline]
    fn log10(self) -> Self {
        self.ln() / F::from(10.0).unwrap().ln()
    }

    #[inline]
    fn to_degrees(self) -> Self {
        Self::new(self.re.to_degrees(), self.im.to_degrees())
    }

    #[inline]
    fn to_radians(self) -> Self {
        Self::new(self.re.to_radians(), self.im.to_radians())
    }

    #[inline]
    fn max(self, other: Self) -> Self {
        if self.norm() >= other.norm() {
            self
        } else {
            other
        }
    }

    #[inline]
    fn min(self, other: Self) -> Self {
        if self.norm() <= other.norm() {
            self
        } else {
            other
        }
    }

    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        if self.norm() < min.norm() {
            min
        } else if self.norm() > max.norm() {
            max
        } else {
            self
        }
    }

    #[inline]
    fn cbrt(self) -> Self {
        self.powf(Self::new(F::from(1.0 / 3.0).unwrap(), F::zero()))
    }

    #[inline]
    fn hypot(self, other: Self) -> Self {
        // For complex numbers, hypot is typically defined as the magnitude of the sum of squares
        // or the magnitude of the complex number itself. Here, we'll use the magnitude of the complex number.
        // If a more specific definition is needed, it should be clarified.
        // For now, we'll use the definition for real numbers applied to the magnitude.
        Self::new(self.norm().hypot(other.norm()), F::zero())
    }

    #[inline]
    fn sin(self) -> Self {
        Self::new(
            self.re.sin() * self.im.cosh(),
            self.re.cos() * self.im.sinh(),
        )
    }

    #[inline]
    fn cos(self) -> Self {
        Self::new(
            self.re.cos() * self.im.cosh(),
            -self.re.sin() * self.im.sinh(),
        )
    }

    #[inline]
    fn tan(self) -> Self {
        self.sin() / self.cos()
    }

    #[inline]
    fn asin(self) -> Self {
        // -i * ln(i * z + sqrt(1 - z^2))
        let i = Self::new(F::zero(), F::one());
        -i * (i * self + (Self::one() - self * self).sqrt()).ln()
    }

    #[inline]
    fn acos(self) -> Self {
        // -i * ln(z + i * sqrt(1 - z^2))
        let i = Self::new(F::zero(), F::one());
        -i * (self + i * (Self::one() - self * self).sqrt()).ln()
    }

    #[inline]
    fn atan(self) -> Self {
        // i/2 * (ln(1 - i*z) - ln(1 + i*z))
        let i = Self::new(F::zero(), F::one());
        let two = Self::new(F::from(2.0).unwrap(), F::zero());
        (i / two) * ((Self::one() - i * self).ln() - (Self::one() + i * self).ln())
    }

    #[inline]
    fn atan2(self, other: Self) -> Self {
        // atan2(y, x) for complex numbers is arg(x + iy)
        Self::new(
            (other + Self::new(F::zero(), F::one()) * self).arg(),
            F::zero(),
        )
    }

    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        (self.sin(), self.cos())
    }

    #[inline]
    fn exp_m1(self) -> Self {
        self.exp() - Self::one()
    }

    #[inline]
    fn ln_1p(self) -> Self {
        (Self::one() + self).ln()
    }

    #[inline]
    fn sinh(self) -> Self {
        Self::new(
            self.re.sinh() * self.im.cos(),
            self.re.cosh() * self.im.sin(),
        )
    }

    #[inline]
    fn cosh(self) -> Self {
        Self::new(
            self.re.cosh() * self.im.cos(),
            self.re.sinh() * self.im.sin(),
        )
    }

    #[inline]
    fn tanh(self) -> Self {
        self.sinh() / self.cosh()
    }

    #[inline]
    fn asinh(self) -> Self {
        // ln(z + sqrt(z^2 + 1))
        (self + (self * self + Self::one()).sqrt()).ln()
    }

    #[inline]
    fn acosh(self) -> Self {
        // ln(z + sqrt(z^2 - 1))
        (self + (self * self - Self::one()).sqrt()).ln()
    }

    #[inline]
    fn atanh(self) -> Self {
        // 0.5 * (ln(1 + z) - ln(1 - z))
        let half = Self::new(F::from(0.5).unwrap(), F::zero());
        half * ((Self::one() + self).ln() - (Self::one() - self).ln())
    }

    #[inline]
    fn integer_decode(self) -> (u64, i16, i8) {
        self.re.integer_decode()
    }

    #[inline]
    fn copysign(self, sign: Self) -> Self {
        Self::new(self.re.copysign(sign.re), self.im.copysign(sign.im))
    }
}
