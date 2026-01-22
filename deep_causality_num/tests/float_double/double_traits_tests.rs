/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Trait compliance tests for `DoubleFloat`.

use deep_causality_num::{
    AbelianGroup, Associative, Commutative, Distributive, DivisionAlgebra, Float106, Field,
    Float, Num, NumCast, One, RealField, ToPrimitive, Zero,
};

// =============================================================================
// Identity Trait Tests
// =============================================================================

#[test]
fn test_zero_trait() {
    let zero = Float106::zero();
    assert!(zero.is_zero());
    assert_eq!(zero.hi(), 0.0);
    assert_eq!(zero.lo(), 0.0);
}

#[test]
fn test_zero_set_zero() {
    let mut x = Float106::from_f64(42.0);
    x.set_zero();
    assert!(x.is_zero());
}

#[test]
fn test_one_trait() {
    let one = Float106::one();
    assert!(one.is_one());
    assert_eq!(one.hi(), 1.0);
    assert_eq!(one.lo(), 0.0);
}

#[test]
fn test_one_set_one() {
    let mut x = Float106::from_f64(42.0);
    x.set_one();
    assert!(x.is_one());
}

// =============================================================================
// Num Trait Tests
// =============================================================================

fn assert_num<T: Num>() {}

#[test]
fn test_num_trait_bound() {
    assert_num::<Float106>();
}

// =============================================================================
// Float Trait Tests
// =============================================================================

fn assert_float<T: Float>() {}

#[test]
fn test_float_trait_bound() {
    assert_float::<Float106>();
}

#[test]
fn test_is_nan() {
    assert!(<Float106 as Float>::nan().is_nan());
    assert!(!Float106::from_f64(1.0).is_nan());
}

#[test]
fn test_is_infinite() {
    assert!(<Float106 as Float>::infinity().is_infinite());
    assert!(<Float106 as Float>::neg_infinity().is_infinite());
    assert!(!Float106::from_f64(1.0).is_infinite());
}

#[test]
fn test_is_finite() {
    assert!(Float106::from_f64(1.0).is_finite());
    assert!(!<Float106 as Float>::infinity().is_finite());
    assert!(!<Float106 as Float>::nan().is_finite());
}

#[test]
fn test_is_normal() {
    assert!(Float106::from_f64(1.0).is_normal());
    assert!(!Float106::from_f64(0.0).is_normal());
}

#[test]
fn test_classify() {
    use core::num::FpCategory;
    assert_eq!(Float106::from_f64(1.0).classify(), FpCategory::Normal);
    assert_eq!(Float106::from_f64(0.0).classify(), FpCategory::Zero);
    assert_eq!(
        <Float106 as Float>::infinity().classify(),
        FpCategory::Infinite
    );
    assert_eq!(<Float106 as Float>::nan().classify(), FpCategory::Nan);
}

// =============================================================================
// NumCast and ToPrimitive Tests
// =============================================================================

fn assert_numcast<T: NumCast>() {}

#[test]
fn test_numcast_trait_bound() {
    assert_numcast::<Float106>();
}

#[test]
fn test_to_f64() {
    let x = Float106::from_f64(42.5);
    assert_eq!(ToPrimitive::to_f64(&x), Some(42.5));
}

#[test]
fn test_to_f32() {
    let x = Float106::from_f64(42.5);
    assert_eq!(ToPrimitive::to_f32(&x), Some(42.5_f32));
}

#[test]
fn test_to_i64() {
    let x = Float106::from_f64(42.0);
    assert_eq!(ToPrimitive::to_i64(&x), Some(42));
}

#[test]
fn test_to_u64() {
    let x = Float106::from_f64(42.0);
    assert_eq!(ToPrimitive::to_u64(&x), Some(42));
}

#[test]
fn test_numcast_from_f64() {
    let x: Float106 = NumCast::from(42.0_f64).unwrap();
    assert_eq!(x.hi(), 42.0);
}

#[test]
fn test_numcast_from_i32() {
    let x: Float106 = NumCast::from(42_i32).unwrap();
    assert_eq!(x.hi(), 42.0);
}

// =============================================================================
// Marker Trait Tests
// =============================================================================

fn assert_associative<T: Associative>() {}
fn assert_commutative<T: Commutative>() {}
fn assert_distributive<T: Distributive>() {}

#[test]
fn test_associative_bound() {
    assert_associative::<Float106>();
}

#[test]
fn test_commutative_bound() {
    assert_commutative::<Float106>();
}

#[test]
fn test_distributive_bound() {
    assert_distributive::<Float106>();
}

// =============================================================================
// Algebra Trait Tests
// =============================================================================

fn assert_abelian_group<T: AbelianGroup>() {}
fn assert_field<T: Field>() {}
fn assert_real_field<T: RealField>() {}

#[test]
fn test_abelian_group_bound() {
    assert_abelian_group::<Float106>();
}

#[test]
fn test_field_bound() {
    assert_field::<Float106>();
}

#[test]
fn test_real_field_bound() {
    assert_real_field::<Float106>();
}

// =============================================================================
// DivisionAlgebra Tests
// =============================================================================

#[test]
fn test_conjugate() {
    let x = Float106::from_f64(5.0);
    // For reals, conjugate is identity
    assert_eq!(x.conjugate(), x);
}

#[test]
fn test_norm_sqr() {
    let x = Float106::from_f64(3.0);
    let norm_sq = x.norm_sqr();
    assert_eq!(norm_sq.hi(), 9.0);
}

#[test]
fn test_inverse() {
    let x = Float106::from_f64(4.0);
    let inv = x.inverse();
    assert_eq!(inv.hi(), 0.25);
}

#[test]
fn test_inverse_identity() {
    let x = Float106::from_f64(4.0);
    let product = x * x.inverse();
    assert!((product.hi() - 1.0).abs() < 1e-14);
}

// =============================================================================
// Copy/Clone/Default Tests
// =============================================================================

#[test]
fn test_copy() {
    let x = Float106::from_f64(42.0);
    let y = x; // Copy
    assert_eq!(x, y);
}

#[test]
fn test_clone() {
    let x = Float106::from_f64(42.0);
    #[allow(clippy::clone_on_copy)]
    let y = x.clone();
    assert_eq!(x, y);
}

#[test]
fn test_default() {
    let x = Float106::default();
    assert_eq!(x.hi(), 0.0);
    assert_eq!(x.lo(), 0.0);
}

// =============================================================================
// Debug/Display Tests
// =============================================================================

#[test]
fn test_debug() {
    let x = Float106::from_f64(42.0);
    let debug_str = format!("{:?}", x);
    assert!(debug_str.contains("DoubleFloat"));
    assert!(debug_str.contains("42"));
}

#[test]
fn test_display() {
    let x = Float106::from_f64(42.0);
    let display_str = format!("{}", x);
    assert!(display_str.contains("42"));
}

#[test]
fn test_trig() {
    let angle_val = 0.5_f64;
    let angle = Float106::from_f64(angle_val);

    // sin
    let s = RealField::sin(angle);
    assert!((s.to_f64() - angle_val.sin()).abs() < 1e-12, "sin failed");

    // cos
    let c = RealField::cos(angle);
    assert!((c.to_f64() - angle_val.cos()).abs() < 1e-12, "cos failed");

    // tan
    let t = RealField::tan(angle);
    assert!((t.to_f64() - angle_val.tan()).abs() < 1e-12, "tan failed");

    // asin
    let as_val = RealField::asin(angle);
    assert!(
        (as_val.to_f64() - angle_val.asin()).abs() < 1e-12,
        "asin failed"
    );

    // acos
    let ac_val = RealField::acos(angle);
    assert!(
        (ac_val.to_f64() - angle_val.acos()).abs() < 1e-12,
        "acos failed"
    );

    // atan
    let at_val = RealField::atan(angle);
    assert!(
        (at_val.to_f64() - angle_val.atan()).abs() < 1e-12,
        "atan failed"
    );
}
