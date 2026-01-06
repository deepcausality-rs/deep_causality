/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Trait compliance tests for `DoubleFloat`.

use deep_causality_num::{
    AbelianGroup, Associative, Commutative, DivisionAlgebra, Distributive, DoubleFloat, Field,
    Float, Num, NumCast, One, RealField, ToPrimitive, Zero,
};

// =============================================================================
// Identity Trait Tests
// =============================================================================

#[test]
fn test_zero_trait() {
    let zero = DoubleFloat::zero();
    assert!(zero.is_zero());
    assert_eq!(zero.hi(), 0.0);
    assert_eq!(zero.lo(), 0.0);
}

#[test]
fn test_zero_set_zero() {
    let mut x = DoubleFloat::from_f64(42.0);
    x.set_zero();
    assert!(x.is_zero());
}

#[test]
fn test_one_trait() {
    let one = DoubleFloat::one();
    assert!(one.is_one());
    assert_eq!(one.hi(), 1.0);
    assert_eq!(one.lo(), 0.0);
}

#[test]
fn test_one_set_one() {
    let mut x = DoubleFloat::from_f64(42.0);
    x.set_one();
    assert!(x.is_one());
}

// =============================================================================
// Num Trait Tests
// =============================================================================

fn assert_num<T: Num>() {}

#[test]
fn test_num_trait_bound() {
    assert_num::<DoubleFloat>();
}

// =============================================================================
// Float Trait Tests
// =============================================================================

fn assert_float<T: Float>() {}

#[test]
fn test_float_trait_bound() {
    assert_float::<DoubleFloat>();
}

#[test]
fn test_is_nan() {
    assert!(<DoubleFloat as Float>::nan().is_nan());
    assert!(!DoubleFloat::from_f64(1.0).is_nan());
}

#[test]
fn test_is_infinite() {
    assert!(<DoubleFloat as Float>::infinity().is_infinite());
    assert!(<DoubleFloat as Float>::neg_infinity().is_infinite());
    assert!(!DoubleFloat::from_f64(1.0).is_infinite());
}

#[test]
fn test_is_finite() {
    assert!(DoubleFloat::from_f64(1.0).is_finite());
    assert!(!<DoubleFloat as Float>::infinity().is_finite());
    assert!(!<DoubleFloat as Float>::nan().is_finite());
}

#[test]
fn test_is_normal() {
    assert!(DoubleFloat::from_f64(1.0).is_normal());
    assert!(!DoubleFloat::from_f64(0.0).is_normal());
}

#[test]
fn test_classify() {
    use core::num::FpCategory;
    assert_eq!(
        DoubleFloat::from_f64(1.0).classify(),
        FpCategory::Normal
    );
    assert_eq!(
        DoubleFloat::from_f64(0.0).classify(),
        FpCategory::Zero
    );
    assert_eq!(<DoubleFloat as Float>::infinity().classify(), FpCategory::Infinite);
    assert_eq!(<DoubleFloat as Float>::nan().classify(), FpCategory::Nan);
}

// =============================================================================
// NumCast and ToPrimitive Tests
// =============================================================================

fn assert_numcast<T: NumCast>() {}

#[test]
fn test_numcast_trait_bound() {
    assert_numcast::<DoubleFloat>();
}

#[test]
fn test_to_f64() {
    let x = DoubleFloat::from_f64(42.5);
    assert_eq!(ToPrimitive::to_f64(&x), Some(42.5));
}

#[test]
fn test_to_f32() {
    let x = DoubleFloat::from_f64(42.5);
    assert_eq!(ToPrimitive::to_f32(&x), Some(42.5_f32));
}

#[test]
fn test_to_i64() {
    let x = DoubleFloat::from_f64(42.0);
    assert_eq!(ToPrimitive::to_i64(&x), Some(42));
}

#[test]
fn test_to_u64() {
    let x = DoubleFloat::from_f64(42.0);
    assert_eq!(ToPrimitive::to_u64(&x), Some(42));
}

#[test]
fn test_numcast_from_f64() {
    let x: DoubleFloat = NumCast::from(42.0_f64).unwrap();
    assert_eq!(x.hi(), 42.0);
}

#[test]
fn test_numcast_from_i32() {
    let x: DoubleFloat = NumCast::from(42_i32).unwrap();
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
    assert_associative::<DoubleFloat>();
}

#[test]
fn test_commutative_bound() {
    assert_commutative::<DoubleFloat>();
}

#[test]
fn test_distributive_bound() {
    assert_distributive::<DoubleFloat>();
}

// =============================================================================
// Algebra Trait Tests
// =============================================================================

fn assert_abelian_group<T: AbelianGroup>() {}
fn assert_field<T: Field>() {}
fn assert_real_field<T: RealField>() {}

#[test]
fn test_abelian_group_bound() {
    assert_abelian_group::<DoubleFloat>();
}

#[test]
fn test_field_bound() {
    assert_field::<DoubleFloat>();
}

#[test]
fn test_real_field_bound() {
    assert_real_field::<DoubleFloat>();
}

// =============================================================================
// DivisionAlgebra Tests
// =============================================================================

#[test]
fn test_conjugate() {
    let x = DoubleFloat::from_f64(5.0);
    // For reals, conjugate is identity
    assert_eq!(x.conjugate(), x);
}

#[test]
fn test_norm_sqr() {
    let x = DoubleFloat::from_f64(3.0);
    let norm_sq = x.norm_sqr();
    assert_eq!(norm_sq.hi(), 9.0);
}

#[test]
fn test_inverse() {
    let x = DoubleFloat::from_f64(4.0);
    let inv = x.inverse();
    assert_eq!(inv.hi(), 0.25);
}

#[test]
fn test_inverse_identity() {
    let x = DoubleFloat::from_f64(4.0);
    let product = x * x.inverse();
    assert!((product.hi() - 1.0).abs() < 1e-14);
}

// =============================================================================
// Copy/Clone/Default Tests
// =============================================================================

#[test]
fn test_copy() {
    let x = DoubleFloat::from_f64(42.0);
    let y = x; // Copy
    assert_eq!(x, y);
}

#[test]
fn test_clone() {
    let x = DoubleFloat::from_f64(42.0);
    #[allow(clippy::clone_on_copy)]
    let y = x.clone();
    assert_eq!(x, y);
}

#[test]
fn test_default() {
    let x = DoubleFloat::default();
    assert_eq!(x.hi(), 0.0);
    assert_eq!(x.lo(), 0.0);
}

// =============================================================================
// Debug/Display Tests
// =============================================================================

#[test]
fn test_debug() {
    let x = DoubleFloat::from_f64(42.0);
    let debug_str = format!("{:?}", x);
    assert!(debug_str.contains("DoubleFloat"));
    assert!(debug_str.contains("42"));
}

#[test]
fn test_display() {
    let x = DoubleFloat::from_f64(42.0);
    let display_str = format!("{}", x);
    assert!(display_str.contains("42"));
}
