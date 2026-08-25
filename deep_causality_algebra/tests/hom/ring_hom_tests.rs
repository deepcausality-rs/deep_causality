/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::{Hom, RingHom};

/// The identity on ℤ, which preserves everything.
struct IdInt;
impl Hom for IdInt {
    type Domain = i64;
    type Codomain = i64;
    fn apply(&self, x: i64) -> i64 {
        x
    }
}
impl RingHom for IdInt {}

fn assert_ring_hom<H: RingHom>() {}

#[test]
fn test_ring_hom_is_a_hom() {
    assert_ring_hom::<IdInt>();
    assert_eq!(IdInt.apply(5), 5);
}

#[test]
fn test_additive_law() {
    let (a, b) = (17_i64, 25_i64);
    assert_eq!(IdInt.apply(a + b), IdInt.apply(a) + IdInt.apply(b));
}

#[test]
fn test_multiplicative_law() {
    let (a, b) = (6_i64, 7_i64);
    assert_eq!(IdInt.apply(a * b), IdInt.apply(a) * IdInt.apply(b));
}

#[test]
fn test_unital_law() {
    // Unitality does not follow from the other two: the zero map preserves both operations and
    // sends 1 to 0, which is why `RingHom` states it.
    assert_eq!(IdInt.apply(1), 1);

    struct ZeroMap;
    impl Hom for ZeroMap {
        type Domain = i64;
        type Codomain = i64;
        fn apply(&self, _: i64) -> i64 {
            0
        }
    }
    // The zero map is additive and multiplicative...
    let (a, b) = (3_i64, 4_i64);
    assert_eq!(ZeroMap.apply(a + b), ZeroMap.apply(a) + ZeroMap.apply(b));
    assert_eq!(ZeroMap.apply(a * b), ZeroMap.apply(a) * ZeroMap.apply(b));
    // ...but not unital, so it is deliberately not given a `RingHom` impl.
    assert_ne!(ZeroMap.apply(1), 1);
}
