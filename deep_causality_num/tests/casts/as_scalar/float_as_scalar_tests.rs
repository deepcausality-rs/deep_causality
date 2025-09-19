/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::FloatAsScalar;

macro_rules! test_splat {
    ($name:ident, $type:ty, $value:expr) => {
        #[test]
        fn $name() {
            let value: $type = $value;
            assert_eq!(<$type as FloatAsScalar>::splat(value), value);
        }
    };
}

macro_rules! test_splat_nan {
    ($name:ident, $type:ty, $value:expr) => {
        #[test]
        fn $name() {
            let value: $type = $value;
            assert!(<$type as FloatAsScalar>::splat(value).is_nan());
        }
    };
}

mod f32_as_scalar_tests {
    use super::*;

    test_splat!(splat_f32_pos, f32, 1.0);
    test_splat!(splat_f32_neg, f32, -1.0);
    test_splat!(splat_f32_zero, f32, 0.0);
    test_splat_nan!(splat_f32_nan, f32, f32::NAN);
    test_splat!(splat_f32_infinity, f32, f32::INFINITY);
    test_splat!(splat_f32_neg_infinity, f32, f32::NEG_INFINITY);
}

mod f64_as_scalar_tests {
    use super::*;

    test_splat!(splat_f64_pos, f64, 1.0);
    test_splat!(splat_f64_neg, f64, -1.0);
    test_splat!(splat_f64_zero, f64, 0.0);
    test_splat_nan!(splat_f64_nan, f64, f64::NAN);
    test_splat!(splat_f64_infinity, f64, f64::INFINITY);
    test_splat!(splat_f64_neg_infinity, f64, f64::NEG_INFINITY);
}
