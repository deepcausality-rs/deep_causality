/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::IntoFloat;

macro_rules! test_into_float {
    ($name:ident, $type:ty, $val:expr, $exponent:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let value: $type = $val;
            let exponent: i32 = $exponent;
            let expected: <$type as IntoFloat>::F = $expected;
            let actual = value.into_float_with_exponent(exponent);
            assert_eq!(actual, expected);
        }
    };
}

mod u32_into_float_tests {
    use super::*;

    // Basic cases
    test_into_float!(into_float_u32_zero_exponent_zero, u32, 0, 0, 1.0f32);
    test_into_float!(
        into_float_u32_one_exponent_zero,
        u32,
        1,
        0,
        1.0f32 + f32::EPSILON
    );

    // Positive exponents
    test_into_float!(into_float_u32_zero_exponent_one, u32, 0, 1, 2.0f32);
    test_into_float!(into_float_u32_zero_exponent_two, u32, 0, 2, 4.0f32);

    // Negative exponents
    test_into_float!(into_float_u32_zero_exponent_neg_one, u32, 0, -1, 0.5f32);
    test_into_float!(into_float_u32_zero_exponent_neg_two, u32, 0, -2, 0.25f32);

    // Edge cases for exponent
    test_into_float!(
        into_float_u32_zero_exponent_max,
        u32,
        0,
        127,
        f32::from_bits(254 << 23)
    ); // Max normal exponent
    test_into_float!(
        into_float_u32_zero_exponent_min,
        u32,
        0,
        -126,
        f32::from_bits(1 << 23)
    ); // Min normal exponent

    // Fraction bits filled
    test_into_float!(
        into_float_u32_fraction_max_exponent_zero,
        u32,
        (1 << 23) - 1,
        0,
        f32::from_bits((127 << 23) | ((1 << 23) - 1))
    );

    // Combined cases
    test_into_float!(
        into_float_u32_combined_1,
        u32,
        10,
        5,
        f32::from_bits(((127 + 5) as u32) << 23 | 10)
    );
    test_into_float!(
        into_float_u32_combined_2,
        u32,
        100,
        -10,
        f32::from_bits(((127 - 10) as u32) << 23 | 100)
    );
}
