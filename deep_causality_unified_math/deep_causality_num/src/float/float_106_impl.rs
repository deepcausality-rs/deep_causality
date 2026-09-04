/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Float, Float106};
use core::num::FpCategory;

/// Whether a value carries a negative sign, negative zero included.
///
/// `hi < 0.0` is false for `-0.0`, so every sign test written that way silently turned a
/// negative zero into a positive one. The high word carries the sign of every canonical
/// value — `Float106::new` normalises `lo` to zero whenever the sum is zero, so a signed
/// zero is `(±0.0, ±0.0)` — and `is_sign_negative` reads it. The low word is consulted
/// only for the non-canonical pairs `Float106::from_raw` admits, where `hi` may be `+0.0`
/// while `lo` holds the value; the previous tests already carried that clause.
#[inline]
fn is_negative(x: Float106) -> bool {
    x.hi.is_sign_negative() || (x.hi == 0.0 && x.lo < 0.0)
}

impl Float for Float106 {
    #[inline]
    fn nan() -> Self {
        Self {
            hi: f64::NAN,
            lo: f64::NAN,
        }
    }

    #[inline]
    fn infinity() -> Self {
        Self {
            hi: f64::INFINITY,
            lo: f64::INFINITY,
        }
    }

    #[inline]
    fn neg_infinity() -> Self {
        Self {
            hi: f64::NEG_INFINITY,
            lo: f64::NEG_INFINITY,
        }
    }

    #[inline]
    fn neg_zero() -> Self {
        Self { hi: -0.0, lo: -0.0 }
    }

    #[inline]
    fn min_value() -> Self {
        Self {
            hi: f64::MIN,
            lo: 0.0,
        }
    }

    #[inline]
    fn min_positive_value() -> Self {
        Self {
            hi: f64::MIN_POSITIVE,
            lo: 0.0,
        }
    }

    #[inline]
    fn epsilon() -> Self {
        Self::EPSILON
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
    fn max_value() -> Self {
        Self {
            hi: f64::MAX,
            lo: 0.0,
        }
    }

    #[inline]
    fn is_nan(self) -> bool {
        self.hi.is_nan()
    }

    #[inline]
    fn is_infinite(self) -> bool {
        self.hi.is_infinite()
    }

    #[inline]
    fn is_finite(self) -> bool {
        self.hi.is_finite()
    }

    #[inline]
    fn is_normal(self) -> bool {
        self.hi.is_normal()
    }

    #[inline]
    fn is_subnormal(self) -> bool {
        self.hi.classify() == FpCategory::Subnormal
    }

    #[inline]
    fn classify(self) -> FpCategory {
        self.hi.classify()
    }

    /// Rounding keeps both halves. When the high half is already integral the low half decides,
    /// and the result is renormalised rather than summed into one `f64`, so an integer past 2⁵³
    /// survives; when the high half is not integral its distance to the nearest integer exceeds
    /// the low half, so the high half alone decides.
    #[inline]
    fn floor(self) -> Self {
        let hi_floor = self.hi.floor();
        if hi_floor == self.hi {
            Self::new(self.hi, self.lo.floor())
        } else {
            Self::from(hi_floor)
        }
    }

    #[inline]
    fn ceil(self) -> Self {
        let hi_ceil = self.hi.ceil();
        if hi_ceil == self.hi {
            Self::new(self.hi, self.lo.ceil())
        } else {
            Self::from(hi_ceil)
        }
    }

    #[inline]
    fn round(self) -> Self {
        let hi_round = self.hi.round();
        if hi_round == self.hi {
            Self::new(self.hi, self.lo.round())
        } else if (self.hi - self.hi.trunc()).abs() == 0.5 && self.lo != 0.0 {
            // The high half sits on a half; the low half says which side the value is on.
            let rounded = if self.lo > 0.0 {
                self.hi + 0.5
            } else {
                self.hi - 0.5
            };
            if rounded == 0.0 && self.hi.is_sign_negative() {
                Self::neg_zero()
            } else {
                Self::from(rounded)
            }
        } else {
            Self::from(hi_round)
        }
    }

    /// Toward zero: the floor of a non-negative value and the ceiling of a negative one.
    #[inline]
    fn trunc(self) -> Self {
        if self.hi.is_sign_negative() {
            self.ceil()
        } else {
            self.floor()
        }
    }

    #[inline]
    fn fract(self) -> Self {
        self - self.trunc()
    }

    /// `abs(-0.0)` is `+0.0`: taking a magnitude clears the sign bit, it does not keep it.
    #[inline]
    fn abs(self) -> Self {
        if is_negative(self) { -self } else { self }
    }

    /// `1.0` for a positive value, `+0.0` included, and `-1.0` for a negative one, `-0.0`
    /// included, as the trait documents and as `f32`, `f64` and `BFloat16` return. The
    /// branch this replaced answered `0.0` for both zeros, so neither sign of zero got the
    /// documented answer.
    #[inline]
    fn signum(self) -> Self {
        if self.is_nan() {
            Self::nan()
        } else if is_negative(self) {
            Self::from(-1.0)
        } else {
            Self::from(1.0)
        }
    }

    #[inline]
    fn is_sign_positive(self) -> bool {
        self.hi.is_sign_positive()
    }

    #[inline]
    fn is_sign_negative(self) -> bool {
        self.hi.is_sign_negative()
    }

    fn mul_add(self, a: Self, b: Self) -> Self {
        // FMA: self * a + b with higher precision
        self * a + b
    }

    fn recip(self) -> Self {
        Self::from(1.0) / self
    }

    fn powi(self, n: i32) -> Self {
        if n == 0 {
            return Self::from(1.0);
        }

        let mut result = Self::from(1.0);
        let mut base = if n < 0 { self.recip() } else { self };
        let mut exp = n.unsigned_abs();

        while exp > 0 {
            if exp & 1 == 1 {
                result *= base;
            }
            base *= base;
            exp >>= 1;
        }

        result
    }

    fn powf(self, n: Self) -> Self {
        // x^n = exp(n * ln(x))
        if self.hi <= 0.0 {
            if self.hi == 0.0 {
                if n.hi > 0.0 {
                    return Self::from(0.0);
                } else {
                    return Self::infinity();
                }
            }
            return Self::nan();
        }
        (n * self.ln()).exp()
    }

    fn sqrt(self) -> Self {
        // sqrt(±0) = ±0. The zero test comes first because `-0.0 < 0.0` is false, so a
        // negative zero is not the NaN case; returning `Self::from(0.0)` dropped its sign.
        if self.hi == 0.0 {
            return self;
        }
        if self.hi < 0.0 {
            return Self::nan();
        }

        // Newton-Raphson iteration: x_{n+1} = 0.5 * (x_n + a/x_n)
        let x0 = self.hi.sqrt();
        let mut x = Self::from(x0);

        // Two iterations for full precision
        x = (x + self / x) * Self::from(0.5);
        x = (x + self / x) * Self::from(0.5);

        x
    }

    fn exp(self) -> Self {
        // Range reduction: e^x = 2^k * e^r where r = x - k*ln(2)
        if self.hi == 0.0 && self.lo == 0.0 {
            return Self::from(1.0);
        }
        if self.is_nan() {
            return Self::nan();
        }
        if self.hi > 709.0 {
            return Self::infinity();
        }
        if self.hi < -709.0 {
            return Self::from(0.0);
        }

        // k = round(x / ln(2))
        let inv_ln2 = Self::from(core::f64::consts::LOG2_E);
        let k_f = (self * inv_ln2).hi.round();
        let k = k_f as i32;

        // r = x - k * ln(2)
        let r = self - Self::LN_2 * Self::from(k_f);

        // Taylor series for e^r
        let mut sum = Self::from(1.0);
        let mut term = r;
        sum += term;

        for i in 2..60 {
            term = term * r / Self::from(i as f64);
            sum += term;
            if term.abs().hi < 1e-32 {
                break;
            }
        }

        // Multiply by 2^k
        sum * Self::from(2.0_f64.powi(k))
    }

    fn exp2(self) -> Self {
        (self * Self::LN_2).exp()
    }

    fn ln(self) -> Self {
        if self.hi <= 0.0 {
            if self.hi == 0.0 {
                return Self::neg_infinity();
            }
            return Self::nan();
        }
        if self.hi.is_infinite() {
            // ln(+inf) = +inf. The iteration below would evaluate inf + inf/inf - 1 = NaN.
            return Self::infinity();
        }
        if self.hi == 1.0 && self.lo == 0.0 {
            return Self::from(0.0);
        }

        // Newton-Raphson: x_{n+1} = x_n + (a - e^{x_n}) / e^{x_n}
        //                        = x_n + a * e^{-x_n} - 1
        let x0 = self.hi.ln();
        let mut x = Self::from(x0);

        // Two iterations
        let exp_x = x.exp();
        x = x + self / exp_x - Self::from(1.0);
        let exp_x = x.exp();
        x = x + self / exp_x - Self::from(1.0);

        x
    }

    fn log(self, base: Self) -> Self {
        self.ln() / base.ln()
    }

    fn log2(self) -> Self {
        self.ln() / Self::LN_2
    }

    fn log10(self) -> Self {
        self.ln() / Self::LN_10
    }

    fn to_degrees(self) -> Self {
        // Scaling ±0 gives ±0. The double-double product renormalises through
        // `quick_two_sum`, and `-0.0 + 0.0` is `+0.0`, so the sign has to be short-circuited.
        if self.hi == 0.0 && self.lo == 0.0 {
            return self;
        }
        self * Self::from(180.0) / Self::PI
    }

    fn to_radians(self) -> Self {
        if self.hi == 0.0 && self.lo == 0.0 {
            return self;
        }
        self * Self::PI / Self::from(180.0)
    }

    fn max(self, other: Self) -> Self {
        if self.is_nan() {
            return other;
        }
        if other.is_nan() {
            return self;
        }
        if self > other { self } else { other }
    }

    fn min(self, other: Self) -> Self {
        if self.is_nan() {
            return other;
        }
        if other.is_nan() {
            return self;
        }
        if self < other { self } else { other }
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        debug_assert!(min <= max);
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }

    fn cbrt(self) -> Self {
        // cbrt(±0) = ±0; the cube root is odd, so the sign of a zero survives it.
        if self.hi == 0.0 {
            return self;
        }

        let sign = if is_negative(self) {
            Self::from(-1.0)
        } else {
            Self::from(1.0)
        };
        let abs_self = self.abs();

        // Newton-Raphson: x_{n+1} = (2*x_n + a/x_n^2) / 3
        let x0 = abs_self.hi.cbrt();
        let mut x = Self::from(x0);

        // 1/3 is not representable, so `Self::from(1.0 / 3.0)` would round it to f64 first and
        // widen a value whose low word is zero, capping the iteration below at f64 accuracy.
        // Dividing in double-double keeps the extra bits.
        let third = Self::from(1.0) / Self::from(3.0);
        x = (x * Self::from(2.0) + abs_self / (x * x)) * third;
        x = (x * Self::from(2.0) + abs_self / (x * x)) * third;

        sign * x
    }

    fn hypot(self, other: Self) -> Self {
        (self * self + other * other).sqrt()
    }

    fn sin(self) -> Self {
        self.sin_cos().0
    }

    fn cos(self) -> Self {
        self.sin_cos().1
    }

    fn tan(self) -> Self {
        // tan(±0) = ±0. The quotient ±0/1 renormalises to +0, so the sign is taken directly.
        if self.hi == 0.0 && self.lo == 0.0 {
            return self;
        }
        self.sin() / self.cos()
    }

    fn asin(self) -> Self {
        if self.hi.abs() > 1.0 {
            return Self::nan();
        }
        if self.hi == 1.0 {
            return Self::FRAC_PI_2;
        }
        if self.hi == -1.0 {
            return -Self::FRAC_PI_2;
        }

        // asin(x) = atan(x / sqrt(1 - x^2))
        let one_minus_x2 = Self::from(1.0) - self * self;
        self.atan2(one_minus_x2.sqrt())
    }

    fn acos(self) -> Self {
        if self.hi.abs() > 1.0 {
            return Self::nan();
        }

        // acos(x) = π/2 - asin(x)
        Self::FRAC_PI_2 - self.asin()
    }

    fn atan(self) -> Self {
        if self.is_nan() {
            return self;
        }
        // atan(±0) = ±0. The series below sums `-0.0 + 0.0`, which is `+0.0`.
        if self.hi == 0.0 && self.lo == 0.0 {
            return self;
        }
        let one = Self::from(1.0);
        if self.hi.is_infinite() {
            return if self.hi > 0.0 {
                Self::FRAC_PI_2
            } else {
                -Self::FRAC_PI_2
            };
        }

        // |x| > 1 goes through the reciprocal: atan(x) = ±π/2 − atan(1/x). This brings every
        // argument into [−1, 1] before the series, and keeps `x * x` below from overflowing.
        if self.abs().hi > 1.0 {
            let inner = (one / self).atan();
            return if self.hi > 0.0 {
                Self::FRAC_PI_2 - inner
            } else {
                -Self::FRAC_PI_2 - inner
            };
        }

        // Halve until the series argument is small: atan(x) = 2·atan(x / (1 + √(1 + x²))).
        // The series converges as y², so a single halving is not enough — an argument left
        // near 1 needs thousands of terms, far more than the loop below runs, and the sum is
        // then truncated rather than converged. Four halvings take |y| under 1/16, where
        // fifteen terms reach the type's precision.
        let mut y = self;
        let mut doublings = 0u32;
        while y.abs().hi > 0.0625 {
            y = y / (one + (one + y * y).sqrt());
            doublings += 1;
        }

        // atan(y) = y − y³/3 + y⁵/5 − …
        let y2 = y * y;
        let mut sum = y;
        let mut term = y;
        for i in 1..80 {
            let n = 2 * i + 1;
            term = -term * y2;
            let contribution = term / Self::from(n as f64);
            sum += contribution;
            if contribution.abs().hi < 1e-35 {
                break;
            }
        }

        for _ in 0..doublings {
            sum *= Self::from(2.0);
        }
        sum
    }

    fn atan2(self, other: Self) -> Self {
        // Handle special cases
        if other.hi == 0.0 && other.lo == 0.0 {
            if self.hi > 0.0 || (self.hi == 0.0 && self.lo > 0.0) {
                return Self::FRAC_PI_2;
            } else if self.hi < 0.0 || (self.hi == 0.0 && self.lo < 0.0) {
                return -Self::FRAC_PI_2;
            } else {
                return Self::nan();
            }
        }

        // atan2(±0, y) with y non-zero and not NaN: ±0 for a positive y, ±π for a negative
        // one. The quotient below is a signed zero whose sign the division normalisation
        // drops, so the sign is read off the numerator here instead.
        if self.hi == 0.0 && self.lo == 0.0 && !other.hi.is_nan() {
            return if other.hi > 0.0 {
                self
            } else if is_negative(self) {
                -Self::PI
            } else {
                Self::PI
            };
        }

        let ratio = self / other;
        let atan_ratio = ratio.atan();

        // Adjust based on quadrant
        if other.hi >= 0.0 {
            atan_ratio
        } else if self.hi >= 0.0 {
            atan_ratio + Self::PI
        } else {
            atan_ratio - Self::PI
        }
    }

    fn sin_cos(self) -> (Self, Self) {
        // sin(±0) = ±0 and cos(±0) = 1. The reduction below runs ±0 through a remainder and
        // a subtraction, and `-0.0 + 0.0` is `+0.0`, so the sign is returned directly.
        if self.hi == 0.0 && self.lo == 0.0 {
            return (self, Self::from(1.0));
        }

        // Range reduction to [-π, π]
        let reduced = self % Self::TWO_PI;
        let x = if reduced.hi > Self::PI.hi {
            reduced - Self::TWO_PI
        } else if reduced.hi < -Self::PI.hi {
            reduced + Self::TWO_PI
        } else {
            reduced
        };

        // Table-based reduction x = k·π/16 + s with |s| ≤ π/32, so the
        // Taylor pair below converges in ~9 terms instead of the ~40+ a
        // full-range argument needs.
        let kf = (x.hi * (16.0 / core::f64::consts::PI)).round();
        let s = x - Self::from(kf) * Self::FRAC_PI_16;
        let k = kf as i32;

        // Taylor pair on the reduced argument, sharing s².
        let s2 = s * s;

        let mut sin_s = s;
        let mut term = s;
        for i in 1..10 {
            let n = 2 * i;
            term = -term * s2 / Self::from((n * (n + 1)) as f64);
            sin_s += term;
            if term.abs().hi < 1e-33 {
                break;
            }
        }

        let mut cos_s = Self::from(1.0);
        let mut term = Self::from(1.0);
        for i in 1..10 {
            let n = 2 * i;
            term = -term * s2 / Self::from((n * (n - 1)) as f64);
            cos_s += term;
            if term.abs().hi < 1e-33 {
                break;
            }
        }

        // Recombine via angle addition with the precomputed sin/cos(k·π/16);
        // sin is odd and cos is even in k. NaN input casts k to 0 and
        // propagates through s.
        let idx = (k.unsigned_abs() as usize).min(16);
        let sin_k = if k < 0 {
            -Self::SIN_K_PI_16[idx]
        } else {
            Self::SIN_K_PI_16[idx]
        };
        let cos_k = Self::COS_K_PI_16[idx];

        (sin_k * cos_s + cos_k * sin_s, cos_k * cos_s - sin_k * sin_s)
    }

    fn exp_m1(self) -> Self {
        // exp_m1(±0) = ±0; the series adds `-0.0 + 0.0` and loses the sign.
        if self.hi == 0.0 && self.lo == 0.0 {
            return self;
        }
        // For small x, use Taylor series directly
        if self.abs().hi < 0.5 {
            let mut sum = self;
            let mut term = self;
            for i in 2..60 {
                term = term * self / Self::from(i as f64);
                sum += term;
                if term.abs().hi < 1e-33 {
                    break;
                }
            }
            sum
        } else {
            self.exp() - Self::from(1.0)
        }
    }

    fn ln_1p(self) -> Self {
        // ln_1p(±0) = ±0; the series adds `-0.0 + 0.0` and loses the sign.
        if self.hi == 0.0 && self.lo == 0.0 {
            return self;
        }
        // For small x, use Taylor series directly
        if self.abs().hi < 0.5 {
            let mut sum = self;
            let mut term = self;
            for i in 2..80 {
                term = -term * self * Self::from((i - 1) as f64) / Self::from(i as f64);
                sum += term;
                if term.abs().hi < 1e-33 {
                    break;
                }
            }
            sum
        } else {
            (Self::from(1.0) + self).ln()
        }
    }

    fn sinh(self) -> Self {
        // sinh(x) = (e^x - e^{-x}) / 2
        if !self.hi.is_finite() || (self.hi == 0.0 && self.lo == 0.0) {
            // sinh(±inf) = ±inf, sinh(NaN) = NaN, sinh(±0) = ±0. Without the first the
            // subtraction below meets a NaN from the opposite-signed exponential and loses
            // the sign; without the second it evaluates `(1 - 1) / 2`, a positive zero.
            return self;
        }
        let exp_x = self.exp();
        let exp_neg_x = (-self).exp();
        (exp_x - exp_neg_x) * Self::from(0.5)
    }

    fn cosh(self) -> Self {
        // cosh(x) = (e^x + e^{-x}) / 2
        if self.hi.is_nan() {
            return self;
        }
        if self.hi.is_infinite() {
            // cosh is even, so both infinities give +inf.
            return Self::infinity();
        }
        let exp_x = self.exp();
        let exp_neg_x = (-self).exp();
        (exp_x + exp_neg_x) * Self::from(0.5)
    }

    fn tanh(self) -> Self {
        // tanh(x) = (e^{2x} - 1) / (e^{2x} + 1), evaluated on |x| and signed afterwards.
        //
        // Taken directly on a signed argument the quotient is neither exactly odd — the two
        // sign branches round differently in the last bits — nor total: for x above about 355
        // the numerator and denominator both overflow and the quotient is NaN, where the true
        // value has already saturated at 1. Both are avoided by reducing to the positive half.
        if self.hi.is_nan() {
            return self;
        }
        let one = Self::from(1.0);
        // `self.hi < 0.0` is false for `-0.0`, so tanh(-0.0) came back as `+0.0`; tanh is odd
        // and IEEE 754 requires tanh(-0.0) = -0.0.
        let negative = is_negative(self);
        let magnitude = self.abs();
        let result = if magnitude.hi.is_infinite() {
            one
        } else {
            let exp_2a = (magnitude * Self::from(2.0)).exp();
            if exp_2a.hi.is_infinite() {
                one
            } else {
                (exp_2a - one) / (exp_2a + one)
            }
        };
        if negative { -result } else { result }
    }

    fn asinh(self) -> Self {
        // asinh(±0) = ±0; the formula below evaluates ln(±0 + 1) = +0 and loses the sign.
        if self.hi == 0.0 && self.lo == 0.0 {
            return self;
        }
        // asinh(x) = ln(x + sqrt(x^2 + 1))
        (self + (self * self + Self::from(1.0)).sqrt()).ln()
    }

    fn acosh(self) -> Self {
        // acosh(x) = ln(x + sqrt(x^2 - 1)) for x >= 1
        if self.hi < 1.0 {
            return Self::nan();
        }
        (self + (self * self - Self::from(1.0)).sqrt()).ln()
    }

    fn atanh(self) -> Self {
        // atanh(±0) = ±0; the formula below evaluates 0.5·ln(1) = +0 and loses the sign.
        if self.hi == 0.0 && self.lo == 0.0 {
            return self;
        }
        // atanh(x) = 0.5 * ln((1+x)/(1-x)) for |x| < 1
        if self.hi.abs() >= 1.0 {
            if self.hi == 1.0 {
                return Self::infinity();
            } else if self.hi == -1.0 {
                return Self::neg_infinity();
            }
            return Self::nan();
        }
        let one = Self::from(1.0);
        ((one + self) / (one - self)).ln() * Self::from(0.5)
    }

    fn integer_decode(self) -> (u64, i16, i8) {
        self.hi.integer_decode()
    }

    fn copysign(self, sign: Self) -> Self {
        if is_negative(sign) {
            -self.abs()
        } else {
            self.abs()
        }
    }
}
