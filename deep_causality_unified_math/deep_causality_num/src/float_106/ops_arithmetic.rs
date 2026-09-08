/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Arithmetic operations for `DoubleFloat` using Error-Free Transformations.

use crate::float_106::{Float106, quick_two_sum, two_prod, two_sum};
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

// =============================================================================
// Negation
// =============================================================================

impl Neg for Float106 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            hi: -self.hi,
            lo: -self.lo,
        }
    }
}

// =============================================================================
// Addition
// =============================================================================

impl Add for Float106 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        // The guard is on the *sum*, not on the operands. A non-finite operand has no low word to
        // correct, and the error-compensation terms below evaluate `inf - inf` on one: `two_sum`
        // would return a correct high word beside a NaN low word, which `is_nan` (a high-word
        // test) does not report and which contaminates every later operation. Two finite high
        // words that overflow reach the same place — `two_sum(MAX, MAX)` gives `(inf, NaN)`, and
        // `quick_two_sum` then turns the high word into NaN as well — so testing the operands
        // alone let an overflow through as NaN where IEEE 754 §7.4 gives an infinity. Testing the
        // sum covers both, and the high words alone give the IEEE answer. Same reason as the
        // guard in `Div` below.
        //
        // One case is answered by an infinity that the exact sum does not require: when the high
        // words overflow by no more than the low words could pull back, the exact value can still
        // be finite. The correction is at most `ulp(hi)`, so this is confined to the last binade,
        // and recovering it would cost a scaled path on every addition.
        let hi_sum = self.hi + rhs.hi;
        if !hi_sum.is_finite() {
            return Self::from_raw(hi_sum, 0.0);
        }
        // Sloppy addition: O(1) algorithm with ~2^-104 relative error
        let (s1, s2) = two_sum(self.hi, rhs.hi);
        let t1 = self.lo + rhs.lo;
        let t2 = s2 + t1;
        let (hi, lo) = quick_two_sum(s1, t2);
        Self::new(hi, lo)
    }
}

impl Add<f64> for Float106 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: f64) -> Self::Output {
        self + Self::from(rhs)
    }
}

impl Add<Float106> for f64 {
    type Output = Float106;

    #[inline]
    fn add(self, rhs: Float106) -> Self::Output {
        Float106::from(self) + rhs
    }
}

impl AddAssign for Float106 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl AddAssign<f64> for Float106 {
    #[inline]
    fn add_assign(&mut self, rhs: f64) {
        *self = *self + rhs;
    }
}

// =============================================================================
// Subtraction
// =============================================================================

impl Sub for Float106 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        // a - b = a + (-b)
        // Negation is exact, so this preserves EFT properties.
        self + (-rhs)
    }
}

impl Sub<f64> for Float106 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: f64) -> Self::Output {
        self - Self::from(rhs)
    }
}

impl Sub<Float106> for f64 {
    type Output = Float106;

    #[inline]
    fn sub(self, rhs: Float106) -> Self::Output {
        Float106::from(self) - rhs
    }
}

impl SubAssign for Float106 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl SubAssign<f64> for Float106 {
    #[inline]
    fn sub_assign(&mut self, rhs: f64) {
        *self = *self - rhs;
    }
}

// =============================================================================
// Multiplication
// =============================================================================

impl Mul for Float106 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // Guarded on the *product*, for the reason `Add` is guarded on the sum. `two_prod` on a
        // non-finite operand produces a NaN low word beside a correct high word; two finite high
        // words whose product overflows do the same, because the FMA error term evaluates
        // `a·b − inf = −inf` and `quick_two_sum(inf, −inf)` is NaN. The high words alone give the
        // IEEE answer in both cases, `inf · 0 = NaN` included.
        //
        // As in `Add`, an overflow the low words could have pulled back is answered with the
        // infinity. The correction `a.hi·b.lo + a.lo·b.hi` is bounded by `ulp(a.hi·b.hi)`, so the
        // band where the exact product is still finite is the last binade.
        let hi_prod = self.hi * rhs.hi;
        if !hi_prod.is_finite() {
            return Self::from_raw(hi_prod, 0.0);
        }
        // C = A * B
        // p1, p2 = two_prod(a.hi, b.hi)
        let (p1, p2) = two_prod(self.hi, rhs.hi);

        // p2 += a.hi * b.lo + a.lo * b.hi
        // The term a.lo * b.lo is O(ulp^2) and negligible
        let t = p2 + self.hi * rhs.lo + self.lo * rhs.hi;

        let (hi, lo) = quick_two_sum(p1, t);
        Self::new(hi, lo)
    }
}

impl Mul<f64> for Float106 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        // Same guard as `Mul<Self>`, which this had none of: a non-finite operand *or* a product
        // that overflows both leave `two_prod` with a NaN error term, and the whole result went
        // out as NaN — `Float106::from(2.0) * f64::INFINITY` among them.
        let hi_prod = self.hi * rhs;
        if !hi_prod.is_finite() {
            return Self::from_raw(hi_prod, 0.0);
        }
        // Optimized: single f64 multiply
        let (p1, p2) = two_prod(self.hi, rhs);
        let t = p2 + self.lo * rhs;
        let (hi, lo) = quick_two_sum(p1, t);
        Self::new(hi, lo)
    }
}

impl Mul<Float106> for f64 {
    type Output = Float106;

    #[inline]
    fn mul(self, rhs: Float106) -> Self::Output {
        rhs * self
    }
}

impl MulAssign for Float106 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl MulAssign<f64> for Float106 {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

// =============================================================================
// Division
// =============================================================================

impl Div for Float106 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        // The refinement multiplies the first quotient back by the divisor, and `inf · 0` is NaN,
        // so a zero or non-finite divisor, a non-finite dividend, and a quotient that overflows
        // are all answered by the `f64` division alone: `±inf`, `0` or NaN, as IEEE 754 has them.
        // The overflow belongs with the rest for the reason it does in `Add` and `Mul` — finite
        // operands reach `two_prod(inf, b.hi)` and come back NaN — and, as there, an overflow the
        // low words could have pulled back is answered with the infinity.
        let q1 = self.hi / rhs.hi;
        if rhs.hi == 0.0 || !rhs.hi.is_finite() || !self.hi.is_finite() || !q1.is_finite() {
            return Self::from(q1);
        }
        // High-precision division using iterative refinement, from `q1 = a.hi / b.hi`.

        // r = a - q1 * b
        // Compute (a.hi - q1 * b.hi) carefully using two_prod
        let (p1, p2) = two_prod(q1, rhs.hi);
        let s = self.hi - p1;
        let t = (s - p2) + self.lo - q1 * rhs.lo;

        // q2 = t / b.hi
        let q2 = t / rhs.hi;

        let (hi, lo) = quick_two_sum(q1, q2);
        Self::new(hi, lo)
    }
}

impl Div<f64> for Float106 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        // Optimized: single f64 divisor. Same guard as `Div<Self>`, overflow included.
        let q1 = self.hi / rhs;
        if rhs == 0.0 || !rhs.is_finite() || !self.hi.is_finite() || !q1.is_finite() {
            return Self::from(q1);
        }
        let (p1, p2) = two_prod(q1, rhs);
        let s = self.hi - p1;
        let t = (s - p2) + self.lo;
        let q2 = t / rhs;
        let (hi, lo) = quick_two_sum(q1, q2);
        Self::new(hi, lo)
    }
}

impl Div<Float106> for f64 {
    type Output = Float106;

    #[inline]
    fn div(self, rhs: Float106) -> Self::Output {
        Float106::from(self) / rhs
    }
}

impl DivAssign for Float106 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl DivAssign<f64> for Float106 {
    #[inline]
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
    }
}

// =============================================================================
// Remainder
// =============================================================================

impl Rem for Float106 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        // r = a - n * b, where n = trunc(a / b)
        let div = self / rhs;
        // `n` is the whole answer here, and a quotient outside the type has no `trunc`. IEEE 754
        // §5.3.1 puts a finite remainder under `|b|` on every finite pair, but this identity
        // cannot reach it once `a / b` leaves the range: carrying the infinity through gives
        // `a - inf = -inf`, a definite and wrong remainder. NaN says there is no answer, which is
        // the true state of this algorithm on such a pair.
        if !div.hi.is_finite() {
            return Self::from_raw(f64::NAN, 0.0);
        }
        #[cfg(feature = "std")]
        let n = div.hi.trunc();
        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        let n = libm::trunc(div.hi);
        self - (rhs * Self::from(n))
    }
}

impl Rem<f64> for Float106 {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: f64) -> Self::Output {
        self % Self::from(rhs)
    }
}

impl Rem<Float106> for f64 {
    type Output = Float106;

    #[inline]
    fn rem(self, rhs: Float106) -> Self::Output {
        Float106::from(self) % rhs
    }
}

impl RemAssign for Float106 {
    #[inline]
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs;
    }
}

impl RemAssign<f64> for Float106 {
    #[inline]
    fn rem_assign(&mut self, rhs: f64) {
        *self = *self % rhs;
    }
}

// =============================================================================
// Reference Operations
// =============================================================================

impl Add<&Float106> for Float106 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: &Float106) -> Self::Output {
        self + *rhs
    }
}

impl Add<Float106> for &Float106 {
    type Output = Float106;

    #[inline]
    fn add(self, rhs: Float106) -> Self::Output {
        *self + rhs
    }
}

impl Add<&Float106> for &Float106 {
    type Output = Float106;

    #[inline]
    fn add(self, rhs: &Float106) -> Self::Output {
        *self + *rhs
    }
}

impl Sub<&Float106> for Float106 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: &Float106) -> Self::Output {
        self - *rhs
    }
}

impl Sub<Float106> for &Float106 {
    type Output = Float106;

    #[inline]
    fn sub(self, rhs: Float106) -> Self::Output {
        *self - rhs
    }
}

impl Sub<&Float106> for &Float106 {
    type Output = Float106;

    #[inline]
    fn sub(self, rhs: &Float106) -> Self::Output {
        *self - *rhs
    }
}

impl Mul<&Float106> for Float106 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: &Float106) -> Self::Output {
        self * *rhs
    }
}

impl Mul<Float106> for &Float106 {
    type Output = Float106;

    #[inline]
    fn mul(self, rhs: Float106) -> Self::Output {
        *self * rhs
    }
}

impl Mul<&Float106> for &Float106 {
    type Output = Float106;

    #[inline]
    fn mul(self, rhs: &Float106) -> Self::Output {
        *self * *rhs
    }
}

impl Div<&Float106> for Float106 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: &Float106) -> Self::Output {
        self / *rhs
    }
}

impl Div<Float106> for &Float106 {
    type Output = Float106;

    #[inline]
    fn div(self, rhs: Float106) -> Self::Output {
        *self / rhs
    }
}

impl Div<&Float106> for &Float106 {
    type Output = Float106;

    #[inline]
    fn div(self, rhs: &Float106) -> Self::Output {
        *self / *rhs
    }
}

impl Rem<&Float106> for Float106 {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: &Float106) -> Self::Output {
        self % *rhs
    }
}

impl Rem<Float106> for &Float106 {
    type Output = Float106;

    #[inline]
    fn rem(self, rhs: Float106) -> Self::Output {
        *self % rhs
    }
}

impl Rem<&Float106> for &Float106 {
    type Output = Float106;

    #[inline]
    fn rem(self, rhs: &Float106) -> Self::Output {
        *self % *rhs
    }
}
