/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Gf2;

const O: Gf2 = Gf2::ZERO;
const I: Gf2 = Gf2::ONE;

#[test]
fn test_addition_is_xor() {
    assert_eq!(O + O, O);
    assert_eq!(O + I, I);
    assert_eq!(I + O, I);
    assert_eq!(I + I, O);
}

#[test]
fn test_add_assign() {
    let mut a = I;
    a += I;
    assert_eq!(a, O);
    a += I;
    assert_eq!(a, I);
}

#[test]
fn test_subtraction_coincides_with_addition() {
    for (x, y) in [(O, O), (O, I), (I, O), (I, I)] {
        assert_eq!(x - y, x + y);
    }
}

#[test]
fn test_sub_assign() {
    let mut a = O;
    a -= I;
    assert_eq!(a, I);
    a -= I;
    assert_eq!(a, O);
}

#[test]
fn test_negation_is_the_identity() {
    assert_eq!(-O, O);
    assert_eq!(-I, I);
}

#[test]
fn test_every_element_is_its_own_additive_inverse() {
    for x in [O, I] {
        assert_eq!(x + (-x), O);
        assert_eq!(x + x, O);
    }
}

#[test]
fn test_multiplication_is_conjunction() {
    assert_eq!(O * O, O);
    assert_eq!(O * I, O);
    assert_eq!(I * O, O);
    assert_eq!(I * I, I);
}

#[test]
fn test_mul_assign() {
    let mut a = I;
    a *= I;
    assert_eq!(a, I);
    a *= O;
    assert_eq!(a, O);
}

#[test]
fn test_division_by_one() {
    assert_eq!(O / I, O);
    assert_eq!(I / I, I);
}

#[test]
fn test_div_assign_by_one() {
    let mut a = I;
    a /= I;
    assert_eq!(a, I);
    let mut b = O;
    b /= I;
    assert_eq!(b, O);
}

#[test]
#[should_panic(expected = "division by zero in GF(2)")]
fn test_division_by_zero_panics() {
    let _ = I / O;
}

#[test]
#[should_panic(expected = "division by zero in GF(2)")]
fn test_div_assign_by_zero_panics() {
    let mut a = I;
    a /= O;
}

#[test]
fn test_one_is_its_own_multiplicative_inverse() {
    assert_eq!(I * I, I);
    assert_eq!(I / I, I);
}

#[test]
fn test_distributivity() {
    for a in [O, I] {
        for b in [O, I] {
            for c in [O, I] {
                assert_eq!(a * (b + c), a * b + a * c);
            }
        }
    }
}

#[test]
fn test_associativity_and_commutativity() {
    for a in [O, I] {
        for b in [O, I] {
            assert_eq!(a + b, b + a);
            assert_eq!(a * b, b * a);
            for c in [O, I] {
                assert_eq!((a + b) + c, a + (b + c));
                assert_eq!((a * b) * c, a * (b * c));
            }
        }
    }
}

#[test]
fn test_characteristic_is_two() {
    // The fact DivisibleByIntegers exists to keep out of code that halves.
    assert_eq!(I + I, O);
}
