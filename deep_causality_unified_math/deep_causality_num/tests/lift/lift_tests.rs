/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The precision-boundary crossings, at the three shipped scalars and every primitive source.

use deep_causality_num::{
    Float106, Lift, Lower, lift, lift_count, lift_f32, lift_f64, lift_i8, lift_i16, lift_i32,
    lift_i64, lift_i128, lift_isize, lift_u8, lift_u16, lift_u32, lift_u64, lift_u128, lift_usize,
    lower, lower_f32, to_count, try_lift, try_lift_count, try_lift_f32, try_lift_f64, try_lift_i8,
    try_lift_i16, try_lift_i32, try_lift_i64, try_lift_i128, try_lift_isize, try_lift_u8,
    try_lift_u16, try_lift_u32, try_lift_u64, try_lift_u128, try_lift_usize, try_lower,
    try_lower_f32,
};

#[test]
fn test_a_literal_lifts_into_each_working_type() {
    let wide: f64 = lift(0.375);
    let narrow: f32 = lift(0.375);
    let double: Float106 = lift(0.375);
    assert_eq!(wide, 0.375);
    assert_eq!(narrow, 0.375);
    assert_eq!(double, Float106::from(0.375));
    // The turbofish names the target where nothing else does.
    assert_eq!(lift::<f64>(2.0), 2.0);
    // `lift` is `lift_f64` under the common name.
    assert_eq!(lift::<Float106>(2.5), lift_f64::<Float106>(2.5));
    assert_eq!(try_lift::<f32>(2.5), try_lift_f64::<f32>(2.5));
}

#[test]
fn test_a_literal_is_exact_at_f64_and_rounded_into_f32() {
    let x = 0.1;
    let wide: f64 = lift(x);
    let narrow: f32 = lift(x);
    assert_eq!(wide, x);
    assert_ne!(f64::from(narrow), x, "f32 cannot hold 0.1 exactly");
    assert!((f64::from(narrow) - x).abs() < 1e-7);
}

#[test]
fn test_every_primitive_float_lifts_into_each_working_type() {
    assert_eq!(lift_f32::<f32>(1.5), 1.5);
    assert_eq!(lift_f32::<f64>(1.5), 1.5);
    assert_eq!(lift_f32::<Float106>(1.5), Float106::from(1.5));
    assert_eq!(lift_f64::<f32>(1.5), 1.5);
    assert_eq!(lift_f64::<f64>(1.5), 1.5);
    assert_eq!(lift_f64::<Float106>(1.5), Float106::from(1.5));
    assert_eq!(try_lift_f32::<f64>(0.25), Some(0.25));
    assert_eq!(try_lift_f64::<Float106>(0.25), Some(Float106::from(0.25)));
}

/// Every primitive integer reaches every working type, exactly, at a value they all hold.
macro_rules! assert_integer_lifts {
    ($($lift:ident, $try:ident: $src:ty;)*) => {
        $(
            let v: $src = 42;
            assert_eq!($lift::<f32>(v), 42.0, stringify!($lift into f32));
            assert_eq!($lift::<f64>(v), 42.0, stringify!($lift into f64));
            assert_eq!($lift::<Float106>(v), Float106::from(42.0), stringify!($lift into Float106));
            assert_eq!($try::<f64>(v), Some(42.0), stringify!($try into f64));
            assert_eq!($try::<u8>(v), Some(42u8), stringify!($try into u8));
            // The method form is the same crossing.
            assert_eq!(v.lift::<f64>(), 42.0, stringify!($src lift method));
            assert_eq!(v.try_lift::<Float106>(), Some(Float106::from(42.0)));
        )*
    };
}

#[test]
fn test_every_primitive_integer_lifts_into_each_working_type() {
    assert_integer_lifts! {
        lift_i8, try_lift_i8: i8;
        lift_i16, try_lift_i16: i16;
        lift_i32, try_lift_i32: i32;
        lift_i64, try_lift_i64: i64;
        lift_i128, try_lift_i128: i128;
        lift_isize, try_lift_isize: isize;
        lift_u8, try_lift_u8: u8;
        lift_u16, try_lift_u16: u16;
        lift_u32, try_lift_u32: u32;
        lift_u64, try_lift_u64: u64;
        lift_u128, try_lift_u128: u128;
        lift_usize, try_lift_usize: usize;
    }
}

#[test]
fn test_signed_integers_carry_their_sign() {
    assert_eq!(lift_i8::<f64>(-8), -8.0);
    assert_eq!(lift_i16::<f32>(-16), -16.0);
    assert_eq!(lift_i32::<Float106>(-32), Float106::from(-32.0));
    assert_eq!(lift_i64::<f64>(-64), -64.0);
    assert_eq!(lift_i128::<f64>(-128), -128.0);
    assert_eq!(lift_isize::<f64>(-1), -1.0);
    // A negative does not fit an unsigned target.
    assert_eq!(try_lift_i32::<u32>(-1), None);
    assert_eq!(try_lift_i128::<u64>(-1), None);
}

#[test]
fn test_wide_integers_round_beyond_the_mantissa() {
    // 2^53 + 1 is the first integer f64 cannot hold.
    let just_beyond: u128 = (1u128 << 53) + 1;
    let wide: f64 = lift_u128(just_beyond);
    assert_eq!(wide, 9_007_199_254_740_992.0);
    let signed: f64 = lift_i128(-(just_beyond as i128));
    assert_eq!(signed, -9_007_199_254_740_992.0);
    // 2^53 itself is exact.
    assert_eq!(lift_u64::<f64>(1 << 53), 9_007_199_254_740_992.0);
    assert_eq!(lift_u128::<f64>(u128::MAX), u128::MAX as f64);
}

#[test]
fn test_a_count_lifts_onto_the_real_axis() {
    let shots: f64 = lift_count(1024);
    assert_eq!(shots, 1024.0);
    let shots: Float106 = lift_count(1024);
    assert_eq!(shots, Float106::from(1024.0));
    assert_eq!(try_lift_count::<f32>(1 << 24), Some(16_777_216.0));
    // `lift_count` is `lift_u64` under the common name.
    assert_eq!(lift_count::<f64>(7), lift_u64::<f64>(7));
    assert_eq!(try_lift_count::<u8>(300), try_lift_u64::<u8>(300));
}

#[test]
fn test_lowering_is_the_display_boundary() {
    assert_eq!(lower(0.25f32), 0.25);
    assert_eq!(lower(0.25f64), 0.25);
    assert_eq!(lower(Float106::from(0.25)), 0.25);
    assert_eq!(try_lower(3u8), Some(3.0));
    assert_eq!(lower_f32(0.25f64), 0.25f32);
    assert_eq!(lower_f32(Float106::from(0.25)), 0.25f32);
    assert_eq!(try_lower_f32(7i32), Some(7.0f32));
}

#[test]
fn test_a_real_rounds_back_to_a_count() {
    assert_eq!(to_count(1023.6f64), Some(1024));
    assert_eq!(to_count(1023.4f32), Some(1023));
    assert_eq!(to_count(Float106::from(409.5)), Some(410));
    assert_eq!(to_count(-1.0f64), None, "a count is not negative");
    assert_eq!(to_count(f64::NAN), None);
    assert_eq!(to_count(f64::INFINITY), None);
    assert_eq!(to_count(1e30f64), None, "does not fit");
}

#[test]
fn test_the_try_forms_report_what_does_not_fit() {
    assert_eq!(try_lift::<u8>(300.0), None);
    assert_eq!(try_lift::<u8>(30.0), Some(30));
    assert_eq!(try_lift_count::<u8>(300), None);
    assert_eq!(try_lift_u16::<u8>(256), None);
    assert_eq!(try_lift_u16::<u8>(255), Some(255));
    assert!(try_lift::<f32>(1.5).is_some());
}

#[test]
fn test_the_method_forms_read_at_the_call_site() {
    let x: Float106 = 0.5.lift();
    assert_eq!(x, Float106::from(0.5));
    let y: f64 = 0.5f32.lift();
    assert_eq!(y, 0.5);
    let n: f64 = 1024u64.lift();
    assert_eq!(n, 1024.0);
    let i: f32 = 7usize.lift();
    assert_eq!(i, 7.0);
    let w: f64 = 3u128.lift();
    assert_eq!(w, 3.0);
    assert_eq!(Float106::from(0.75).lower(), 0.75);
    assert_eq!(Float106::from(0.75).lower_f32(), 0.75f32);
    assert_eq!(0.75f32.try_lower(), Some(0.75));
    assert_eq!(0.75f64.try_lower_f32(), Some(0.75f32));
    assert_eq!((-3i32).lift::<f64>(), -3.0);
}

#[test]
fn test_the_lift_survives_a_round_trip_through_the_working_type() {
    for &x in &[0.0, 1.0, -2.5, 3.140625, 1e-9, 6.02e23] {
        let d: Float106 = lift(x);
        assert_eq!(lower(d), x);
        let w: f64 = lift(x);
        assert_eq!(lower(w), x);
    }
}

#[test]
#[should_panic(expected = "representable")]
fn test_an_unrepresentable_literal_panics_at_the_boundary() {
    let _: u8 = lift(300.0);
}

#[test]
#[should_panic(expected = "i32 must be representable")]
fn test_an_unrepresentable_integer_panics_and_names_its_source() {
    let _: u8 = lift_i32(-1);
}
