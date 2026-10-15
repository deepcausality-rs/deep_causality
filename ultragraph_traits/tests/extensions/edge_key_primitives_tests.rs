/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use ultragraph_traits::EdgeKey;

// Compiles only when `T::Key` is `()`, so the associated type is pinned at compile time; the
// `let () =` bindings below pin the value and run every impl.
fn unit_key<T: EdgeKey<Key = ()>>(payload: &T) {
    payload.edge_key()
}

#[test]
fn test_unit_has_the_unit_key() {
    unit_key(&());
    let () = ().edge_key();
}

#[test]
fn test_bool_has_the_unit_key() {
    unit_key(&false);
    unit_key(&true);
    let () = false.edge_key();
    let () = true.edge_key();
}

#[test]
fn test_u8_has_the_unit_key() {
    unit_key(&0_u8);
    unit_key(&u8::MAX);
    let () = 0_u8.edge_key();
    let () = u8::MAX.edge_key();
}

#[test]
fn test_u16_has_the_unit_key() {
    unit_key(&0_u16);
    unit_key(&u16::MAX);
    let () = 0_u16.edge_key();
    let () = u16::MAX.edge_key();
}

#[test]
fn test_u32_has_the_unit_key() {
    unit_key(&0_u32);
    unit_key(&u32::MAX);
    let () = 0_u32.edge_key();
    let () = u32::MAX.edge_key();
}

#[test]
fn test_u64_has_the_unit_key() {
    unit_key(&0_u64);
    unit_key(&u64::MAX);
    let () = 0_u64.edge_key();
    let () = u64::MAX.edge_key();
}

#[test]
fn test_u128_has_the_unit_key() {
    unit_key(&0_u128);
    unit_key(&u128::MAX);
    let () = 0_u128.edge_key();
    let () = u128::MAX.edge_key();
}

#[test]
fn test_usize_has_the_unit_key() {
    unit_key(&0_usize);
    unit_key(&usize::MAX);
    let () = 0_usize.edge_key();
    let () = usize::MAX.edge_key();
}

#[test]
fn test_i8_has_the_unit_key() {
    unit_key(&0_i8);
    unit_key(&i8::MIN);
    unit_key(&i8::MAX);
    let () = 0_i8.edge_key();
    let () = i8::MIN.edge_key();
    let () = i8::MAX.edge_key();
}

#[test]
fn test_i16_has_the_unit_key() {
    unit_key(&0_i16);
    unit_key(&i16::MIN);
    unit_key(&i16::MAX);
    let () = 0_i16.edge_key();
    let () = i16::MIN.edge_key();
    let () = i16::MAX.edge_key();
}

#[test]
fn test_i32_has_the_unit_key() {
    unit_key(&0_i32);
    unit_key(&i32::MIN);
    unit_key(&i32::MAX);
    let () = 0_i32.edge_key();
    let () = i32::MIN.edge_key();
    let () = i32::MAX.edge_key();
}

#[test]
fn test_i64_has_the_unit_key() {
    unit_key(&0_i64);
    unit_key(&i64::MIN);
    unit_key(&i64::MAX);
    let () = 0_i64.edge_key();
    let () = i64::MIN.edge_key();
    let () = i64::MAX.edge_key();
}

#[test]
fn test_i128_has_the_unit_key() {
    unit_key(&0_i128);
    unit_key(&i128::MIN);
    unit_key(&i128::MAX);
    let () = 0_i128.edge_key();
    let () = i128::MIN.edge_key();
    let () = i128::MAX.edge_key();
}

#[test]
fn test_isize_has_the_unit_key() {
    unit_key(&0_isize);
    unit_key(&isize::MIN);
    unit_key(&isize::MAX);
    let () = 0_isize.edge_key();
    let () = isize::MIN.edge_key();
    let () = isize::MAX.edge_key();
}

#[test]
fn test_f32_has_the_unit_key() {
    unit_key(&0.0_f32);
    unit_key(&(-0.0_f32));
    unit_key(&(-1.5_f32));
    unit_key(&f32::NAN);
    unit_key(&f32::INFINITY);
    unit_key(&f32::NEG_INFINITY);
    let () = 0.0_f32.edge_key();
    let () = (-0.0_f32).edge_key();
    let () = (-1.5_f32).edge_key();
    let () = f32::NAN.edge_key();
    let () = f32::INFINITY.edge_key();
    let () = f32::NEG_INFINITY.edge_key();
}

#[test]
fn test_f64_has_the_unit_key() {
    unit_key(&0.0_f64);
    unit_key(&(-0.0_f64));
    unit_key(&(-1.5_f64));
    unit_key(&f64::NAN);
    unit_key(&f64::INFINITY);
    unit_key(&f64::NEG_INFINITY);
    let () = 0.0_f64.edge_key();
    let () = (-0.0_f64).edge_key();
    let () = (-1.5_f64).edge_key();
    let () = f64::NAN.edge_key();
    let () = f64::INFINITY.edge_key();
    let () = f64::NEG_INFINITY.edge_key();
}
