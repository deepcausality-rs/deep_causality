/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `ToPrimitive`, `FromPrimitive`, `NumCast`, `Sum` and `Product` on `BFloat16`, and
//! for the `lift` crossings that make the type usable as a precision parameter.

use deep_causality_num::{
    BFloat16, Float, Float106, FromPrimitive, Lift, Lower, NumCast, ToPrimitive, lift, lift_count,
    lower, lower_f32, to_count, try_lift,
};

// =============================================================================
// ToPrimitive: integers truncate toward zero, out of range is None
// =============================================================================

#[test]
fn test_to_signed_integers() {
    let x = BFloat16::round_from_f64(42.0);
    assert_eq!(x.to_isize(), Some(42));
    assert_eq!(x.to_i8(), Some(42));
    assert_eq!(x.to_i16(), Some(42));
    assert_eq!(x.to_i32(), Some(42));
    assert_eq!(x.to_i64(), Some(42));
    assert_eq!(x.to_i128(), Some(42));
    assert_eq!(BFloat16::round_from_f64(-42.0).to_i32(), Some(-42));
}

#[test]
fn test_to_integers_truncate_toward_zero() {
    assert_eq!(BFloat16::round_from_f64(3.75).to_i32(), Some(3));
    assert_eq!(BFloat16::round_from_f64(-3.75).to_i32(), Some(-3));
    assert_eq!(BFloat16::round_from_f64(0.5).to_u8(), Some(0));
}

#[test]
fn test_to_unsigned_integers() {
    let x = BFloat16::round_from_f64(200.0);
    assert_eq!(x.to_usize(), Some(200));
    assert_eq!(x.to_u8(), Some(200));
    assert_eq!(x.to_u16(), Some(200));
    assert_eq!(x.to_u32(), Some(200));
    assert_eq!(x.to_u64(), Some(200));
    assert_eq!(x.to_u128(), Some(200));
}

#[test]
fn test_to_integers_reject_out_of_range() {
    assert_eq!(BFloat16::round_from_f64(200.0).to_i8(), None);
    assert_eq!(BFloat16::round_from_f64(-1.0).to_u8(), None);
    assert_eq!(BFloat16::round_from_f64(70000.0).to_u16(), None);
    assert_eq!(BFloat16::round_from_f64(70000.0).to_i16(), None);
    assert_eq!(BFloat16::MAX.to_i64(), None);
    assert_eq!(BFloat16::MAX.to_u64(), None);
    // 2^126 = 0x7E80 fits an i128; 2^127 = 0x7F00 is one past i128::MAX; MAX fits a u128.
    assert_eq!(BFloat16::from_bits(0x7E80).to_i128(), Some(1i128 << 126));
    assert_eq!(BFloat16::from_bits(0x7F00).to_i128(), None);
    assert_eq!(BFloat16::from_bits(0xFF00).to_i128(), Some(i128::MIN));
    assert_eq!(BFloat16::from_bits(0x7F00).to_u128(), Some(1u128 << 127));
    assert_eq!(BFloat16::MAX.to_u128(), Some(0xFFu128 << 120));
}

#[test]
fn test_to_integers_reject_non_finite() {
    for x in [BFloat16::NAN, BFloat16::INFINITY, BFloat16::NEG_INFINITY] {
        assert_eq!(x.to_i32(), None);
        assert_eq!(x.to_u32(), None);
        assert_eq!(x.to_i128(), None);
        assert_eq!(x.to_usize(), None);
    }
}

#[test]
fn test_to_floats_are_exact() {
    let x = BFloat16::from_bits(0x3EAB);
    // 0x3EAB = 171/512 = 0.333984375; the quotient is exact in f32.
    assert_eq!(ToPrimitive::to_f32(&x), Some(171.0_f32 / 512.0));
    assert_eq!(ToPrimitive::to_f64(&x), Some(0.333984375_f64));
    assert_eq!(
        ToPrimitive::to_f32(&BFloat16::NEG_INFINITY),
        Some(f32::NEG_INFINITY)
    );
    assert!(ToPrimitive::to_f64(&BFloat16::NAN).unwrap().is_nan());
}

// =============================================================================
// FromPrimitive: integers round once, exactly
// =============================================================================

#[test]
fn test_from_small_integers_are_exact() {
    // -128 = -2^7: sign 1, biased exponent 134 = 0x86: 0xC300.
    assert_eq!(BFloat16::from_i8(-128).unwrap().to_bits(), 0xC300);
    assert_eq!(BFloat16::from_u8(255).unwrap().to_bits(), 0x437F);
    assert_eq!(BFloat16::from_i16(-42).unwrap().to_f32(), -42.0);
    assert_eq!(BFloat16::from_u16(1000).unwrap().to_f32(), 1000.0);
    assert_eq!(BFloat16::from_i32(0).unwrap().to_bits(), 0x0000);
    assert_eq!(BFloat16::from_isize(-1).unwrap().to_bits(), 0xBF80);
    assert_eq!(BFloat16::from_usize(128).unwrap().to_f32(), 128.0);
    assert_eq!(BFloat16::from_u32(256).unwrap().to_f32(), 256.0);
    assert_eq!(BFloat16::from_u64(1).unwrap(), BFloat16::ONE);
}

#[test]
fn test_from_integers_at_the_eight_bit_boundary() {
    // 255 has exactly eight bits and needs no rounding; 256 has nine with a zero tail; 128 has
    // eight with a single set bit. All three are exact.
    assert_eq!(BFloat16::from_u32(255).unwrap().to_f32(), 255.0);
    assert_eq!(BFloat16::from_u32(256).unwrap().to_f32(), 256.0);
    assert_eq!(BFloat16::from_u32(128).unwrap().to_f32(), 128.0);
    assert_eq!(BFloat16::from_u32(129).unwrap().to_f32(), 129.0);
}

#[test]
fn test_from_integers_round_to_nearest_even() {
    // Above 256 the step is 2. 257 is the tie between 256 (even) and 258 (odd); 259 is the tie
    // between 258 (odd) and 260 (even).
    assert_eq!(BFloat16::from_i32(257).unwrap().to_f32(), 256.0);
    assert_eq!(BFloat16::from_i32(259).unwrap().to_f32(), 260.0);
    assert_eq!(BFloat16::from_i32(-259).unwrap().to_f32(), -260.0);
    assert_eq!(BFloat16::from_i32(258).unwrap().to_f32(), 258.0);
    // Above 512 the step is 4: 1001 is below the midpoint of [1000, 1004], 1003 above it, and
    // 1002 is the tie whose even neighbour is 1000 (significand 122).
    assert_eq!(BFloat16::from_i64(1001).unwrap().to_f32(), 1000.0);
    assert_eq!(BFloat16::from_i64(1003).unwrap().to_f32(), 1004.0);
    assert_eq!(BFloat16::from_i64(1002).unwrap().to_f32(), 1000.0);
}

#[test]
fn test_from_wide_integers_round_once_at_the_top_of_the_range() {
    // u64::MAX = 2^64 - 1: the 8 kept bits are all ones and the remainder rounds up, carrying
    // into 2^64, biased exponent 191 = 0xBF: 0x5F80.
    assert_eq!(BFloat16::from_u64(u64::MAX).unwrap().to_bits(), 0x5F80);
    assert_eq!(
        BFloat16::from_u64(u64::MAX).unwrap().to_f64(),
        2f64.powi(64)
    );
    assert_eq!(
        BFloat16::from_i64(i64::MIN).unwrap().to_f64(),
        -(2f64.powi(63))
    );
    assert_eq!(
        BFloat16::from_i64(i64::MAX).unwrap().to_f64(),
        2f64.powi(63)
    );
    // 2^127 has biased exponent 254 = 0xFE: 0x7F00, the largest power of two the type holds.
    assert_eq!(BFloat16::from_i128(i128::MIN).unwrap().to_bits(), 0xFF00);
    assert_eq!(BFloat16::from_u128(1u128 << 127).unwrap().to_bits(), 0x7F00);
    assert_eq!(BFloat16::from_i128(i128::MAX).unwrap().to_bits(), 0x7F00);
    // 2^128 - 2^120 = 0xFF << 120 is exactly MAX; anything that rounds to 2^128 is infinity.
    assert_eq!(
        BFloat16::from_u128(0xFFu128 << 120).unwrap().to_bits(),
        0x7F7F
    );
    assert_eq!(
        BFloat16::from_u128((0xFFu128 << 120) + (1u128 << 119)).unwrap(),
        BFloat16::INFINITY
    );
    assert_eq!(BFloat16::from_u128(u128::MAX).unwrap(), BFloat16::INFINITY);
}

#[test]
fn test_from_integers_never_round_twice() {
    // 2^60 + 2^52 + 1 has 61 bits. Rounding it to f64 first drops the trailing 1, landing
    // exactly on the bf16 tie 2^60 + 2^52, which then falls to the even side 2^60. Rounded
    // once, the value is above the tie and goes to 2^60 + 2^53.
    let n: i64 = (1 << 60) + (1 << 52) + 1;
    assert_eq!(
        BFloat16::from_i64(n).unwrap().to_f64(),
        2f64.powi(60) + 2f64.powi(53)
    );
    let exact_tie: u64 = (1 << 60) + (1 << 52);
    assert_eq!(
        BFloat16::from_u64(exact_tie).unwrap().to_f64(),
        2f64.powi(60)
    );
}

#[test]
fn test_from_floats() {
    assert_eq!(BFloat16::from_f32(0.1).unwrap().to_bits(), 0x3DCD);
    assert_eq!(BFloat16::from_f64(0.1).unwrap().to_bits(), 0x3DCD);
    let above_tie = 1.0 + 2f64.powi(-8) + 2f64.powi(-30);
    assert_eq!(BFloat16::from_f64(above_tie).unwrap().to_bits(), 0x3F81);
    assert!(BFloat16::from_f64(f64::NAN).unwrap().is_nan());
    assert_eq!(BFloat16::from_f32(f32::INFINITY), Some(BFloat16::INFINITY));
}

// =============================================================================
// NumCast
// =============================================================================

fn assert_numcast<T: NumCast>() {}

#[test]
fn test_numcast_bound() {
    assert_numcast::<BFloat16>();
}

#[test]
fn test_numcast_from_primitives() {
    let a: BFloat16 = NumCast::from(42_i32).unwrap();
    assert_eq!(a.to_f32(), 42.0);
    let b: BFloat16 = NumCast::from(0.1_f64).unwrap();
    assert_eq!(b.to_bits(), 0x3DCD);
    let c: BFloat16 = NumCast::from(0.1_f32).unwrap();
    assert_eq!(c.to_bits(), 0x3DCD);
    let d: BFloat16 = NumCast::from(259_u16).unwrap();
    assert_eq!(d.to_f32(), 260.0);
}

#[test]
fn test_numcast_round_trips_itself_and_reaches_the_primitives() {
    let x = BFloat16::from_bits(0x3EAB);
    let y: BFloat16 = NumCast::from(x).unwrap();
    assert_eq!(y, x);
    let z: f64 = NumCast::from(x).unwrap();
    assert_eq!(z, 0.333984375);
    let n: i32 = NumCast::from(BFloat16::round_from_f64(-3.75)).unwrap();
    assert_eq!(n, -3);
    let none: Option<u8> = NumCast::from(BFloat16::round_from_f64(-1.0));
    assert_eq!(none, None);
}

// =============================================================================
// Sum and Product
// =============================================================================

#[test]
fn test_sum_is_not_product() {
    // half-rs 2.3.0 fixed a `Sum` that multiplied. 1, 2 and 4 sum to 7 and multiply to 8, so the
    // two are told apart; 1, 2, 3 would not have told them apart.
    let xs = [
        BFloat16::round_from_f64(1.0),
        BFloat16::round_from_f64(2.0),
        BFloat16::round_from_f64(4.0),
    ];
    let sum: BFloat16 = xs.iter().copied().sum();
    let product: BFloat16 = xs.iter().copied().product();
    assert_eq!(sum.to_f32(), 7.0);
    assert_eq!(product.to_f32(), 8.0);
    let sum_by_ref: BFloat16 = xs.iter().sum();
    let product_by_ref: BFloat16 = xs.iter().product();
    assert_eq!(sum_by_ref.to_f32(), 7.0);
    assert_eq!(product_by_ref.to_f32(), 8.0);
}

#[test]
fn test_sum_and_product_of_nothing_are_the_identities() {
    let sum: BFloat16 = core::iter::empty::<BFloat16>().sum();
    let product: BFloat16 = core::iter::empty::<BFloat16>().product();
    assert_eq!(sum.to_bits(), 0x0000);
    assert_eq!(product, BFloat16::ONE);
    let sum_by_ref: BFloat16 = [].iter().sum();
    let product_by_ref: BFloat16 = [].iter().product();
    assert_eq!(sum_by_ref.to_bits(), 0x0000);
    assert_eq!(product_by_ref, BFloat16::ONE);
}

#[test]
fn test_sum_and_product_of_one_element_are_that_element() {
    let xs = [BFloat16::round_from_f64(-6.5)];
    let sum: BFloat16 = xs.iter().sum();
    let product: BFloat16 = xs.iter().copied().product();
    assert_eq!(sum.to_f32(), -6.5);
    assert_eq!(product.to_f32(), -6.5);
}

#[test]
fn test_sum_rounds_at_every_step() {
    // 256 + 1 is the tie 257, which goes to 256 at each step, so a bf16 accumulator never
    // leaves 256. That is what an 8-bit significand does, and the fold does not hide it.
    let xs = [
        BFloat16::round_from_f64(256.0),
        BFloat16::round_from_f64(1.0),
        BFloat16::round_from_f64(1.0),
    ];
    let total: BFloat16 = xs.iter().sum();
    assert_eq!(total.to_f32(), 256.0);
}

#[test]
fn test_sum_and_product_propagate_non_finite_values() {
    let with_nan = [BFloat16::ONE, BFloat16::NAN];
    let sum: BFloat16 = with_nan.iter().sum();
    let product: BFloat16 = with_nan.iter().product();
    assert!(sum.is_nan());
    assert!(product.is_nan());
    let with_inf = [BFloat16::INFINITY, BFloat16::MIN];
    let sum: BFloat16 = with_inf.iter().sum();
    assert_eq!(sum, BFloat16::INFINITY);
}

// =============================================================================
// Lift and lower: the type as a precision parameter
// =============================================================================

#[test]
fn test_lift_from_f64_literal() {
    let x: BFloat16 = lift(0.375);
    assert_eq!(x.to_bits(), 0x3EC0);
    assert_eq!(lift::<BFloat16>(0.1).to_bits(), 0x3DCD);
    assert_eq!(try_lift::<BFloat16>(1e39), Some(BFloat16::INFINITY));
    let y: BFloat16 = 0.5.lift();
    assert_eq!(y.to_f32(), 0.5);
}

#[test]
fn test_lift_count_and_to_count() {
    let n: BFloat16 = lift_count(1024);
    assert_eq!(n.to_f32(), 1024.0);
    // 100.5 = 1100100.1b is exact; it rounds half away from zero to 101.
    assert_eq!(to_count(BFloat16::round_from_f64(100.5)), Some(101));
    assert_eq!(to_count(BFloat16::round_from_f64(100.0)), Some(100));
    assert_eq!(to_count(BFloat16::round_from_f64(-1.0)), None);
    assert_eq!(to_count(BFloat16::INFINITY), None);
    assert_eq!(to_count(BFloat16::NAN), None);
    let big: BFloat16 = 1024_u64.lift();
    assert_eq!(big, n);
}

#[test]
fn test_lower() {
    let x = BFloat16::from_bits(0x3EAB);
    assert_eq!(lower(x), 0.333984375);
    assert_eq!(lower_f32(x), 171.0_f32 / 512.0);
    assert_eq!(x.lower(), 0.333984375);
    assert_eq!(x.lower_f32(), 171.0_f32 / 512.0);
}

/// A body written once against the bound; the parameter is the only thing that changes.
fn mean<T: Float + FromPrimitive>(xs: &[T]) -> T {
    let sum = xs.iter().fold(T::zero(), |acc, &x| acc + x);
    sum / lift_count(xs.len() as u64)
}

#[test]
fn test_generic_code_runs_at_every_shipped_precision() {
    // (1 + 2 + 3 + 4) / 4 = 2.5 exactly, at every precision.
    let at_bf16: [BFloat16; 4] = [lift(1.0), lift(2.0), lift(3.0), lift(4.0)];
    let at_f32: [f32; 4] = [1.0, 2.0, 3.0, 4.0];
    let at_f64: [f64; 4] = [1.0, 2.0, 3.0, 4.0];
    let at_f106: [Float106; 4] = [lift(1.0), lift(2.0), lift(3.0), lift(4.0)];
    assert_eq!(lower(mean(&at_bf16)), 2.5);
    assert_eq!(lower(mean(&at_f32)), 2.5);
    assert_eq!(lower(mean(&at_f64)), 2.5);
    assert_eq!(lower(mean(&at_f106)), 2.5);
}
