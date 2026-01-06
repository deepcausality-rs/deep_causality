/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! High-precision floating-point type using double-double arithmetic.
//!
//! `DoubleFloat` represents a number as the unevaluated sum of two `f64` values:
//! $$ x = x_{hi} + x_{lo} $$
//! where $|x_{lo}| \le 0.5 \cdot \text{ulp}(x_{hi})$.
//!
//! This provides approximately 106 bits of significand (~31 decimal digits) while
//! maintaining the same exponent range as `f64`.

/// A high-precision floating point number represented as the sum of two `f64`s.
///
/// Precision: ~31 decimal digits (106 bits significand).
/// Range: Same as `f64` (approx $10^{\pm 308}$).
///
/// # Layout
/// * **Alignment**: 16-byte aligned to enable efficient SIMD loads/stores.
/// * **Representation**: `C` compatible for FFI and stable layout.
///
/// # Invariant
/// For normalized values: `|lo| <= 0.5 * ulp(hi)`
#[derive(Copy, Clone, Default)]
#[repr(C, align(16))]
pub struct DoubleFloat {
    /// High-order component (most significant bits)
    pub(crate) hi: f64,
    /// Low-order component (correction term)
    pub(crate) lo: f64,
}

// =============================================================================
// Constructors
// =============================================================================

impl DoubleFloat {
    /// Creates a new `DoubleFloat` from high and low components.
    ///
    /// The components are normalized so that `|lo| <= 0.5 * ulp(hi)`.
    #[inline]
    pub fn new(hi: f64, lo: f64) -> Self {
        let (h, l) = quick_two_sum(hi, lo);
        Self { hi: h, lo: l }
    }

    /// Creates a `DoubleFloat` from a single `f64`.
    ///
    /// The low component is zero.
    #[inline]
    pub const fn from_f64(x: f64) -> Self {
        Self { hi: x, lo: 0.0 }
    }

    /// Creates a `DoubleFloat` from raw components without normalization.
    ///
    /// # Safety
    /// Caller must ensure the invariant `|lo| <= 0.5 * ulp(hi)` holds.
    #[inline]
    pub const fn from_raw(hi: f64, lo: f64) -> Self {
        Self { hi, lo }
    }
}

// =============================================================================
// Error-Free Transformations (EFT)
// =============================================================================

/// Knuth's TwoSum: Computes `s = a + b` and error `e` exactly.
///
/// Returns `(s, e)` such that `a + b = s + e` exactly.
#[inline]
pub(crate) fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let v = s - a;
    let e = (a - (s - v)) + (b - v);
    (s, e)
}

/// Quick TwoSum for when `|a| >= |b|`.
///
/// More efficient than `two_sum` when the magnitude ordering is known.
#[inline]
pub(crate) fn quick_two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let e = b - (s - a);
    (s, e)
}

/// Dekker's TwoProd: Computes `p = a * b` and error `e` exactly using FMA.
///
/// Returns `(p, e)` such that `a * b = p + e` exactly.
#[inline]
pub(crate) fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    let e = a.mul_add(b, -p);
    (p, e)
}
