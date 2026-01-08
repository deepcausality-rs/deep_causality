/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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

mod u64_into_float_tests {
    use super::*;

    // Basic cases
    test_into_float!(into_float_u64_zero_exponent_zero, u64, 0, 0, 1.0f64);
    test_into_float!(
        into_float_u64_one_exponent_zero,
        u64,
        1,
        0,
        1.0f64 + f64::EPSILON
    );

    // Positive exponents
    test_into_float!(into_float_u64_zero_exponent_one, u64, 0, 1, 2.0f64);
    test_into_float!(into_float_u64_zero_exponent_two, u64, 0, 2, 4.0f64);

    // Negative exponents
    test_into_float!(into_float_u64_zero_exponent_neg_one, u64, 0, -1, 0.5f64);
    test_into_float!(into_float_u64_zero_exponent_neg_two, u64, 0, -2, 0.25f64);

    // Edge cases for exponent
    test_into_float!(
        into_float_u64_zero_exponent_max,
        u64,
        0,
        1023,
        f64::from_bits(2046 << 52)
    ); // Max normal exponent
    test_into_float!(
        into_float_u64_zero_exponent_min,
        u64,
        0,
        -1022,
        f64::from_bits(1 << 52)
    ); // Min normal exponent

    // Fraction bits filled
    test_into_float!(
        into_float_u64_fraction_max_exponent_zero,
        u64,
        (1 << 52) - 1,
        0,
        f64::from_bits((1023 << 52) | ((1 << 52) - 1))
    );

    // Combined cases
    test_into_float!(
        into_float_u64_combined_1,
        u64,
        10,
        5,
        f64::from_bits(((1023 + 5) as u64) << 52 | 10)
    );
    test_into_float!(
        into_float_u64_combined_2,
        u64,
        100,
        -10,
        f64::from_bits(((1023 - 10) as u64) << 52 | 100)
    );
}
