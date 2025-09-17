/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::{NumAssign, NumAssignRef, NumOps, NumRef, RefNum};

// Due to the "ZERO external deps" constraint, and the limitations of `macro_rules!`
// for dynamic test name generation without unstable features, these tests are
// manually written out for clarity and to adhere to the project's requirements.

// Test for i32 - NumOps
#[test]
fn test_i32_num_ops_add() {
    let a: i32 = 10;
    let b: i32 = 3;
    assert_eq!(a + b, 13);
}

#[test]
fn test_i32_num_ops_sub() {
    let a: i32 = 10;
    let b: i32 = 3;
    assert_eq!(a - b, 7);
}

#[test]
fn test_i32_num_ops_mul() {
    let a: i32 = 10;
    let b: i32 = 3;
    assert_eq!(a * b, 30);
}

#[test]
fn test_i32_num_ops_div() {
    let a: i32 = 10;
    let b: i32 = 3;
    assert_eq!(a / b, 3);
}

#[test]
fn test_i32_num_ops_rem() {
    let a: i32 = 10;
    let b: i32 = 3;
    assert_eq!(a % b, 1);
}

// Test for i32 - NumAssignOps
#[test]
fn test_i32_num_assign_ops_add_assign() {
    let mut a: i32 = 10;
    let b: i32 = 3;
    a += b;
    assert_eq!(a, 13);
}

#[test]
fn test_i32_num_assign_ops_sub_assign() {
    let mut a: i32 = 10;
    let b: i32 = 3;
    a -= b;
    assert_eq!(a, 7);
}

#[test]
fn test_i32_num_assign_ops_mul_assign() {
    let mut a: i32 = 10;
    let b: i32 = 3;
    a *= b;
    assert_eq!(a, 30);
}

#[test]
fn test_i32_num_assign_ops_div_assign() {
    let mut a: i32 = 10;
    let b: i32 = 3;
    a /= b;
    assert_eq!(a, 3);
}

#[test]
fn test_i32_num_assign_ops_rem_assign() {
    let mut a: i32 = 10;
    let b: i32 = 3;
    a %= b;
    assert_eq!(a, 1);
}

// Test for f32 - NumOps
#[test]
fn test_f32_num_ops_add() {
    let a: f32 = 10.0;
    let b: f32 = 3.0;
    assert_eq!(a + b, 13.0);
}

#[test]
fn test_f32_num_ops_sub() {
    let a: f32 = 10.0;
    let b: f32 = 3.0;
    assert_eq!(a - b, 7.0);
}

#[test]
fn test_f32_num_ops_mul() {
    let a: f32 = 10.0;
    let b: f32 = 3.0;
    assert_eq!(a * b, 30.0);
}

#[test]
fn test_f32_num_ops_div() {
    let a: f32 = 10.0;
    let b: f32 = 3.0;
    assert_eq!(a / b, 10.0 / 3.0);
}

#[test]
fn test_f32_num_ops_rem() {
    let a: f32 = 10.0;
    let b: f32 = 3.0;
    assert_eq!(a % b, 10.0 % 3.0);
}

// Test for f32 - NumAssignOps
#[test]
fn test_f32_num_assign_ops_add_assign() {
    let mut a: f32 = 10.0;
    let b: f32 = 3.0;
    a += b;
    assert_eq!(a, 13.0);
}

#[test]
fn test_f32_num_assign_ops_sub_assign() {
    let mut a: f32 = 10.0;
    let b: f32 = 3.0;
    a -= b;
    assert_eq!(a, 7.0);
}

#[test]
fn test_f32_num_assign_ops_mul_assign() {
    let mut a: f32 = 10.0;
    let b: f32 = 3.0;
    a *= b;
    assert_eq!(a, 30.0);
}

#[test]
fn test_f32_num_assign_ops_div_assign() {
    let mut a: f32 = 10.0;
    let b: f32 = 3.0;
    a /= b;
    assert_eq!(a, 10.0 / 3.0);
}

#[test]
fn test_f32_num_assign_ops_rem_assign() {
    let mut a: f32 = 10.0;
    let b: f32 = 3.0;
    a %= b;
    assert_eq!(a, 10.0 % 3.0);
}

// Test for u32 - NumOps
#[test]
fn test_u32_num_ops_add() {
    let a: u32 = 10;
    let b: u32 = 3;
    assert_eq!(a + b, 13);
}

#[test]
fn test_u32_num_ops_sub() {
    let a: u32 = 10;
    let b: u32 = 3;
    assert_eq!(a - b, 7);
}

#[test]
fn test_u32_num_ops_mul() {
    let a: u32 = 10;
    let b: u32 = 3;
    assert_eq!(a * b, 30);
}

#[test]
fn test_u32_num_ops_div() {
    let a: u32 = 10;
    let b: u32 = 3;
    assert_eq!(a / b, 3);
}

#[test]
fn test_u32_num_ops_rem() {
    let a: u32 = 10;
    let b: u32 = 3;
    assert_eq!(a % b, 1);
}

// Test for u32 - NumAssignOps
#[test]
fn test_u32_num_assign_ops_add_assign() {
    let mut a: u32 = 10;
    let b: u32 = 3;
    a += b;
    assert_eq!(a, 13);
}

#[test]
fn test_u32_num_assign_ops_sub_assign() {
    let mut a: u32 = 10;
    let b: u32 = 3;
    a -= b;
    assert_eq!(a, 7);
}

#[test]
fn test_u32_num_assign_ops_mul_assign() {
    let mut a: u32 = 10;
    let b: u32 = 3;
    a *= b;
    assert_eq!(a, 30);
}

#[test]
fn test_u32_num_assign_ops_div_assign() {
    let mut a: u32 = 10;
    let b: u32 = 3;
    a /= b;
    assert_eq!(a, 3);
}

#[test]
fn test_u32_num_assign_ops_rem_assign() {
    let mut a: u32 = 10;
    let b: u32 = 3;
    a %= b;
    assert_eq!(a, 1);
}

// Test marker traits
#[test]
fn test_num_ops_for_i32() {
    fn assert_impl<T: NumOps>() {}
    assert_impl::<i32>();
}

#[test]
fn test_num_ops_for_f32() {
    fn assert_impl<T: NumOps>() {}
    assert_impl::<f32>();
}

#[test]
fn test_num_ops_for_u32() {
    fn assert_impl<T: NumOps>() {}
    assert_impl::<u32>();
}

#[test]
fn test_num_ref_for_i32() {
    fn assert_impl<T: NumRef>() {}
    assert_impl::<i32>();
}

#[test]
fn test_num_ref_for_f32() {
    fn assert_impl<T: NumRef>() {}
    assert_impl::<f32>();
}

#[test]
fn test_num_ref_for_u32() {
    fn assert_impl<T: NumRef>() {}
    assert_impl::<u32>();
}

#[test]
fn test_ref_num_for_i32() {
    fn assert_impl<T: RefNum<i32>>() {}
    assert_impl::<&i32>();
}

#[test]
fn test_ref_num_for_f32() {
    fn assert_impl<T: RefNum<f32>>() {}
    assert_impl::<&f32>();
}

#[test]
fn test_ref_num_for_u32() {
    fn assert_impl<T: RefNum<u32>>() {}
    assert_impl::<&u32>();
}

#[test]
fn test_num_assign_for_i32() {
    fn assert_impl<T: NumAssign>() {}
    assert_impl::<i32>();
}

#[test]
fn test_num_assign_for_f32() {
    fn assert_impl<T: NumAssign>() {}
    assert_impl::<f32>();
}

#[test]
fn test_num_assign_for_u32() {
    fn assert_impl<T: NumAssign>() {}
    assert_impl::<u32>();
}

#[test]
fn test_num_assign_ref_for_i32() {
    fn assert_impl<T: NumAssignRef>() {}
    assert_impl::<i32>();
}

#[test]
fn test_num_assign_ref_for_f32() {
    fn assert_impl<T: NumAssignRef>() {}
    assert_impl::<f32>();
}

#[test]
fn test_num_assign_ref_for_u32() {
    fn assert_impl<T: NumAssignRef>() {}
    assert_impl::<u32>();
}
