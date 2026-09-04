/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Bound witnesses for `BFloat16`: each call compiles only if the type reaches the bound, so the
//! call is the assertion. This is what "fits into the unified math" means for a new scalar.

use deep_causality_algebra::{
    AbelianGroup, Additive, Annihilating, Associative, Characteristic, Commutative,
    CommutativeRing, Distributive, DivisibleByIntegers, DivisionAlgebra, Field, IntegralDomain,
    Invertible, Multiplicative, Real, RealField, Scalar,
};
use deep_causality_num::BFloat16;

// =============================================================================
// Law markers
// =============================================================================

fn assert_associative<T: Associative<Multiplicative>>() {}
fn assert_additive_associative<T: Associative<Additive>>() {}
fn assert_commutative<T: Commutative<Multiplicative>>() {}
fn assert_additive_commutative<T: Commutative<Additive>>() {}
fn assert_distributive<T: Distributive>() {}
fn assert_annihilating<T: Annihilating>() {}
fn assert_invertible<T: Invertible>() {}

#[test]
fn test_law_marker_bounds() {
    assert_associative::<BFloat16>();
    assert_additive_associative::<BFloat16>();
    assert_commutative::<BFloat16>();
    assert_additive_commutative::<BFloat16>();
    assert_distributive::<BFloat16>();
    assert_annihilating::<BFloat16>();
    assert_invertible::<BFloat16>();
}

// =============================================================================
// The tower
// =============================================================================

fn assert_abelian_group<T: AbelianGroup>() {}
fn assert_commutative_ring<T: CommutativeRing>() {}
fn assert_integral_domain<T: IntegralDomain>() {}
fn assert_field<T: Field>() {}
fn assert_real<T: Real>() {}
fn assert_real_field<T: RealField>() {}
fn assert_scalar<T: Scalar>() {}
fn assert_division_algebra<T: Field + DivisionAlgebra<T>>() {}

#[test]
fn test_tower_bounds() {
    assert_abelian_group::<BFloat16>();
    assert_commutative_ring::<BFloat16>();
    assert_integral_domain::<BFloat16>();
    assert_field::<BFloat16>();
    assert_real::<BFloat16>();
    assert_real_field::<BFloat16>();
    assert_scalar::<BFloat16>();
    assert_division_algebra::<BFloat16>();
}

// =============================================================================
// Characteristic
// =============================================================================

fn admits_characteristic_zero<T: DivisibleByIntegers>() {}

/// The shape of a body that divides by an integer literal.
fn halve<T: DivisibleByIntegers>(x: T) -> T {
    x / (T::one() + T::one())
}

#[test]
fn test_characteristic_zero() {
    assert_eq!(<BFloat16 as Characteristic>::CHARACTERISTIC, 0);
    admits_characteristic_zero::<BFloat16>();
    // 6 / 2 = 3 exactly.
    assert_eq!(halve(BFloat16::from(6.0)), BFloat16::from(3.0));
    // No positive multiple of one is zero: 255 ones sum to 255 = 0x437F, not zero.
    let mut n = BFloat16::ZERO;
    for _ in 0..255 {
        n += BFloat16::ONE;
    }
    assert_eq!(n.to_bits(), 0x437F);
}
