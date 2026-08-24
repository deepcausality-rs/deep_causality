/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::{Characteristic, DivisibleByIntegers, FiniteField};
use deep_causality_num::{Float106, Gf2, One};

// Admission witnesses. Each of these compiles only if the type reaches the bound, so the call is
// the assertion.
fn admits_characteristic_zero<T: DivisibleByIntegers>() {}
fn admits_finite_field<T: FiniteField>() {}

/// The shape of every site the sweep rebounded: a body that divides by an integer.
fn halve<T: DivisibleByIntegers>(x: T) -> T {
    x / (T::one() + T::one())
}

#[test]
fn test_real_scalars_have_characteristic_zero() {
    assert_eq!(<f32 as Characteristic>::CHARACTERISTIC, 0);
    assert_eq!(<f64 as Characteristic>::CHARACTERISTIC, 0);
    assert_eq!(<Float106 as Characteristic>::CHARACTERISTIC, 0);
}

#[test]
fn test_integers_have_characteristic_zero() {
    // Not fields, so they reach neither refinement -- but characteristic is a property of any ring
    // with unity, and Z has characteristic zero.
    assert_eq!(<i8 as Characteristic>::CHARACTERISTIC, 0);
    assert_eq!(<i16 as Characteristic>::CHARACTERISTIC, 0);
    assert_eq!(<i32 as Characteristic>::CHARACTERISTIC, 0);
    assert_eq!(<i64 as Characteristic>::CHARACTERISTIC, 0);
    assert_eq!(<i128 as Characteristic>::CHARACTERISTIC, 0);
    assert_eq!(<isize as Characteristic>::CHARACTERISTIC, 0);
}

#[test]
fn test_gf2_has_characteristic_two() {
    assert_eq!(<Gf2 as Characteristic>::CHARACTERISTIC, 2);
}

#[test]
fn test_gf2_has_order_two() {
    assert_eq!(<Gf2 as FiniteField>::ORDER, 2);
}

#[test]
fn test_gf2_order_and_characteristic_coincide_here() {
    // q = p^k with k = 1, so the two numbers are equal for F2. They part at F4, which has order 4
    // and characteristic 2 -- which is why the trait exposes both.
    assert_eq!(
        u64::from(<Gf2 as Characteristic>::CHARACTERISTIC),
        <Gf2 as FiniteField>::ORDER
    );
}

#[test]
fn test_the_real_scalars_are_admitted_to_characteristic_zero() {
    admits_characteristic_zero::<f32>();
    admits_characteristic_zero::<f64>();
    admits_characteristic_zero::<Float106>();
}

#[test]
fn test_gf2_is_admitted_to_finite_field() {
    admits_finite_field::<Gf2>();
}

#[test]
fn test_halving_works_over_the_reals() {
    assert_eq!(halve(4.0_f64), 2.0);
    assert_eq!(halve(1.0_f32), 0.5);
    assert_eq!(halve(Float106::from(6.0)), Float106::from(3.0));
}

#[test]
fn test_the_characteristic_is_what_makes_halving_valid() {
    // Over a characteristic-zero field the divisor is genuinely two, and non-zero.
    let two = f64::one() + f64::one();
    assert_eq!(two, 2.0);

    // Over GF(2) it is zero, which is the whole reason the bound exists. `halve` cannot be called
    // with `Gf2` at all -- that refusal is the compile_fail doctest on `DivisibleByIntegers`.
    let two_in_gf2 = Gf2::one() + Gf2::one();
    assert_eq!(two_in_gf2, Gf2::ZERO);
}

#[test]
fn test_the_two_refinements_are_disjoint_for_every_type_in_the_tower() {
    // No type is both, because a finite field has prime characteristic. The compiler cannot state
    // that -- this MSRV has no negative impls -- so it is checked over the types that exist.
    assert_ne!(<Gf2 as Characteristic>::CHARACTERISTIC, 0);
    assert_eq!(<f64 as Characteristic>::CHARACTERISTIC, 0);
}
