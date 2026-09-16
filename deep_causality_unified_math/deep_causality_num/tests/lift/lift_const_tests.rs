/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The compile-time lifting crossings, against the runtime ones.
//!
//! # The property that matters
//!
//! A program swapping `lift(30.0)` for `const_scalar_from_int!(FloatType, 30)` must get the same
//! number. The suite therefore compares every macro against [`lift`] bit for bit, at all four
//! shipped scalars, rather than against a literal that would only restate the macro.
//!
//! # Suite audit
//!
//! **Not a tautology.** The four scalars disagree with each other on most of these inputs:
//! `BFloat16` rounds `0.0627` to `0.0625` and `f32` does not, and `Float106` keeps a residue past
//! 2⁵³ that the other three drop. A macro ignoring its target type fails.
//!
//! **Not circular.** The comparison is against `lift`, which reaches the value through
//! `FromPrimitive` at runtime, while the macros reach it through `const` evaluation of a bit
//! pattern or an `i128`. Two paths that share no code agreeing is evidence.

use deep_causality_num::{
    BFloat16, Float106, const_scalar_from_float, const_scalar_from_int, lift,
};

/// Renders a scalar by its exact debug form, so a comparison sees every bit a type carries.
fn exact<T: core::fmt::Debug>(x: T) -> String {
    format!("{x:?}")
}

// ============================================================================
// const_scalar_from_int! against the runtime lift
// ============================================================================

#[test]
fn test_const_scalar_from_int_agrees_with_lift_at_every_scalar() {
    const V_F64: f64 = const_scalar_from_int!(f64, 30);
    const V_F32: f32 = const_scalar_from_int!(f32, 30);
    const V_BF16: BFloat16 = const_scalar_from_int!(BFloat16, 30);
    const V_F106: Float106 = const_scalar_from_int!(Float106, 30);

    assert_eq!(exact(V_F64), exact(lift::<f64>(30.0)));
    assert_eq!(exact(V_F32), exact(lift::<f32>(30.0)));
    assert_eq!(exact(V_BF16), exact(lift::<BFloat16>(30.0)));
    assert_eq!(exact(V_F106), exact(lift::<Float106>(30.0)));
}

#[test]
fn test_const_scalar_from_int_carries_a_negative() {
    const BELOW: Float106 = const_scalar_from_int!(Float106, -30);
    const BELOW_F32: f32 = const_scalar_from_int!(f32, -30);

    assert_eq!(exact(BELOW), exact(lift::<Float106>(-30.0)));
    assert_eq!(BELOW_F32, -30.0);
}

#[test]
fn test_const_scalar_from_int_is_zero_at_zero() {
    const ZERO: Float106 = const_scalar_from_int!(Float106, 0);
    assert_eq!(ZERO.to_f64(), 0.0);
}

/// The reason the integer carrier exists rather than routing every literal through `f64`.
///
/// `2⁶⁰ + 1` needs 61 significand bits. `f64` holds 53 and drops the final one, so a program
/// writing the value as an `f64` literal loses it before `Float106` ever sees it. The `i128`
/// carrier hands `Float106` both words.
#[test]
fn test_const_scalar_from_int_stays_exact_past_the_f64_mantissa() {
    const N: i128 = 1_152_921_504_606_846_977; // 2^60 + 1
    const EXACT: Float106 = const_scalar_from_int!(Float106, N);

    // The head is what `f64` alone would have returned, and the residue is the bit it dropped.
    let via_f64: Float106 = lift(N as f64);
    assert_ne!(exact(EXACT), exact(via_f64), "the residue must survive");

    // Reading the pair back recovers the integer the literal named.
    let recovered = EXACT - via_f64;
    assert_eq!(recovered.to_f64(), 1.0);
}

#[test]
fn test_const_scalar_from_int_matches_lift_below_the_mantissa_ceiling() {
    // Under 2^53 the two routes must agree exactly, because `f64` loses nothing there.
    const SMALL: Float106 = const_scalar_from_int!(Float106, 9_007_199_254_740_991); // 2^53 - 1
    assert_eq!(
        exact(SMALL),
        exact(lift::<Float106>(9_007_199_254_740_991.0))
    );
}

// ============================================================================
// const_scalar_from_float! against the runtime lift
// ============================================================================

#[test]
fn test_const_scalar_from_float_agrees_with_lift_at_every_scalar() {
    const V_F64: f64 = const_scalar_from_float!(f64, 0.0627);
    const V_F32: f32 = const_scalar_from_float!(f32, 0.0627);
    const V_BF16: BFloat16 = const_scalar_from_float!(BFloat16, 0.0627);
    const V_F106: Float106 = const_scalar_from_float!(Float106, 0.0627);

    assert_eq!(exact(V_F64), exact(lift::<f64>(0.0627)));
    assert_eq!(exact(V_F32), exact(lift::<f32>(0.0627)));
    assert_eq!(exact(V_BF16), exact(lift::<BFloat16>(0.0627)));
    assert_eq!(exact(V_F106), exact(lift::<Float106>(0.0627)));
}

/// The scalars must disagree here, or the suite above would pass for a macro ignoring its target.
#[test]
fn test_const_scalar_from_float_rounds_per_scalar() {
    const V_BF16: BFloat16 = const_scalar_from_float!(BFloat16, 0.0627);
    const V_F64: f64 = const_scalar_from_float!(f64, 0.0627);

    // Eight significand bits reach the nearest binary fraction they can hold.
    assert_eq!(V_BF16.to_f64(), 0.0625);
    assert_eq!(V_F64, 0.0627);
    assert_ne!(V_BF16.to_f64(), V_F64);
}

#[test]
fn test_const_scalar_from_float_carries_a_negative_fraction() {
    const LATERAL: Float106 = const_scalar_from_float!(Float106, -0.0003);
    assert_eq!(exact(LATERAL), exact(lift::<Float106>(-0.0003)));
}

#[test]
fn test_const_scalar_from_float_handles_an_exact_binary_fraction() {
    // A half is exact at every scalar, so all four must land on the same number.
    const HALF_F64: f64 = const_scalar_from_float!(f64, 0.5);
    const HALF_BF16: BFloat16 = const_scalar_from_float!(BFloat16, 0.5);
    const HALF_F106: Float106 = const_scalar_from_float!(Float106, 0.5);

    assert_eq!(HALF_F64, 0.5);
    assert_eq!(HALF_BF16.to_f64(), 0.5);
    assert_eq!(HALF_F106.to_f64(), 0.5);
}

// ============================================================================
// The constants are constants
// ============================================================================

/// The point of the macros: the result is usable where only a constant is accepted.
#[test]
fn test_the_results_are_usable_in_const_position() {
    const DEPTH: Float106 = const_scalar_from_int!(Float106, 30);
    const VAPOUR: Float106 = const_scalar_from_float!(Float106, 0.0627);

    // A const array, whose elements must themselves be constants.
    const BOUNDS: [Float106; 2] = [DEPTH, VAPOUR];
    // An array length, which only a constant can fill.
    const COUNT: usize = BOUNDS.len();

    assert_eq!(COUNT, 2);
    assert_eq!(BOUNDS[0].to_f64(), 30.0);
}

/// A constant declared through the alias follows the alias, which is the whole point.
#[test]
fn test_a_constant_follows_the_working_type_alias() {
    type Working = f32;
    const AT_F32: Working = const_scalar_from_int!(Working, 30);

    type Wider = Float106;
    const AT_F106: Wider = const_scalar_from_int!(Wider, 30);

    assert_eq!(core::any::type_name::<Working>(), "f32");
    assert_eq!(AT_F32, 30.0);
    assert_eq!(AT_F106.to_f64(), 30.0);
}
