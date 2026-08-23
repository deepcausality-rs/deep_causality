/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{One, Zero};
use deep_causality_num_rational::Rational;

#[test]
fn a_fraction_renders_as_numerator_over_denominator() {
    assert_eq!(Rational::new(3_i64, 4).to_string(), "3/4");
    assert_eq!(Rational::new(22_i64, 7).to_string(), "22/7");
}

#[test]
fn a_unit_denominator_is_dropped() {
    assert_eq!(Rational::new(6_i64, 3).to_string(), "2");
    assert_eq!(Rational::<i64>::one().to_string(), "1");
    assert_eq!(Rational::<i64>::zero().to_string(), "0");
}

#[test]
fn the_sign_renders_on_the_numerator() {
    assert_eq!(Rational::new(1_i64, -2).to_string(), "-1/2");
    assert_eq!(Rational::new(-1_i64, 2).to_string(), "-1/2");
    assert_eq!(Rational::new(-4_i64, 2).to_string(), "-2");
}
