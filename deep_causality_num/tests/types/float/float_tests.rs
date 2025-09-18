/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::Float;
use std::num::FpCategory;

macro_rules! test_float_method {
    ($name:ident, $method:ident, $ty:ty, $val:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let v: $ty = $val;
            let expected = $expected;
            let actual = v.$method();
            assert_eq!(actual, expected);
        }
    };
    ($name:ident, $method:ident, $ty:ty, $val:expr, $arg:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let v: $ty = $val;
            let arg: $ty = $arg;
            let expected = $expected;
            let actual = v.$method(arg);
            assert_eq!(actual, expected);
        }
    };
    ($name:ident, $method:ident, $ty:ty, $val:expr, $arg1:expr, $arg2:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let v: $ty = $val;
            let arg1: $ty = $arg1;
            let arg2: $ty = $arg2;
            let expected = $expected;
            let actual = v.$method(arg1, arg2);
            assert_eq!(actual, expected);
        }
    };
}

// Test for f32
mod f32_tests {
    use super::*;

    // Constants
    #[test]
    fn test_nan() {
        assert!(f32::nan().is_nan());
    }
    #[test]
    fn test_infinity() {
        assert!(f32::infinity().is_infinite());
    }
    #[test]
    fn test_neg_infinity() {
        assert!(f32::neg_infinity().is_infinite());
    }
    #[test]
    fn test_neg_zero() {
        assert_eq!(f32::neg_zero(), -0.0f32);
    }
    #[test]
    fn test_min_value() {
        assert_eq!(f32::min_value(), f32::MIN);
    }
    #[test]
    fn test_min_positive_value() {
        assert_eq!(f32::min_positive_value(), f32::MIN_POSITIVE);
    }
    #[test]
    fn test_epsilon() {
        assert_eq!(f32::epsilon(), f32::EPSILON);
    }
    #[test]
    fn test_max_value() {
        assert_eq!(f32::max_value(), f32::MAX);
    }

    // Methods
    test_float_method!(is_nan_true, is_nan, f32, f32::NAN, true);
    test_float_method!(is_nan_false, is_nan, f32, 1.0, false);
    test_float_method!(is_infinite_true, is_infinite, f32, f32::INFINITY, true);
    test_float_method!(is_infinite_false, is_infinite, f32, 1.0, false);
    test_float_method!(is_finite_true, is_finite, f32, 1.0, true);
    test_float_method!(is_finite_false, is_finite, f32, f32::INFINITY, false);
    test_float_method!(is_normal_true, is_normal, f32, 1.0, true);
    test_float_method!(is_normal_false, is_normal, f32, 0.0, false);
    test_float_method!(
        is_subnormal_true,
        is_subnormal,
        f32,
        f32::MIN_POSITIVE / 2.0,
        true
    );
    test_float_method!(is_subnormal_false, is_subnormal, f32, 1.0, false);
    test_float_method!(classify_normal, classify, f32, 1.0, FpCategory::Normal);
    test_float_method!(
        classify_infinite,
        classify,
        f32,
        f32::INFINITY,
        FpCategory::Infinite
    );
    test_float_method!(floor_val, floor, f32, 3.9, 3.0);
    test_float_method!(ceil_val, ceil, f32, 3.1, 4.0);
    test_float_method!(round_val, round, f32, 3.5, 4.0);
    test_float_method!(trunc_val, trunc, f32, 3.9, 3.0);
    test_float_method!(fract_val, fract, f32, 3.5, 0.5);
    test_float_method!(abs_val, abs, f32, -3.0, 3.0);
    test_float_method!(signum_pos, signum, f32, 3.0, 1.0);
    test_float_method!(signum_neg, signum, f32, -3.0, -1.0);
    test_float_method!(is_sign_positive_true, is_sign_positive, f32, 1.0, true);
    test_float_method!(is_sign_positive_false, is_sign_positive, f32, -1.0, false);
    test_float_method!(is_sign_negative_true, is_sign_negative, f32, -1.0, true);
    test_float_method!(is_sign_negative_false, is_sign_negative, f32, 1.0, false);
    test_float_method!(mul_add_val, mul_add, f32, 2.0, 3.0, 4.0, 10.0); // 2*3 + 4 = 10
    test_float_method!(recip_val, recip, f32, 2.0, 0.5);

    #[test]
    fn test_f32_powi_val() {
        let v: f32 = 2.0;
        let n: i32 = 3;
        let expected: f32 = 8.0;
        let actual = v.powi(n);
        assert_eq!(actual, expected);
    }

    test_float_method!(powf_val, powf, f32, 2.0, 3.0, 8.0); // 2^3.0 = 8.0
    test_float_method!(sqrt_val, sqrt, f32, 4.0, 2.0);
    test_float_method!(exp_val, exp, f32, 1.0, std::f32::consts::E);
    test_float_method!(exp2_val, exp2, f32, 3.0, 8.0); // 2^3 = 8
    #[test]
    fn ln_val() {
        let v: f32 = std::f32::consts::E;
        let expected = 1.0;
        let actual = v.ln();
        assert!((actual - expected).abs() < 1e-6);
    }
    test_float_method!(log_val, log, f32, 10.0, 10.0, 1.0); // log10(10) = 1
    test_float_method!(log2_val, log2, f32, 8.0, 3.0);
    test_float_method!(log10_val, log10, f32, 100.0, 2.0);
    test_float_method!(to_degrees_val, to_degrees, f32, std::f32::consts::PI, 180.0);
    test_float_method!(to_radians_val, to_radians, f32, 180.0, std::f32::consts::PI);
    test_float_method!(max_val, max, f32, 1.0, 2.0, 2.0);
    test_float_method!(min_val, min, f32, 1.0, 2.0, 1.0);
    test_float_method!(cbrt_val, cbrt, f32, 8.0, 2.0);
    test_float_method!(hypot_val, hypot, f32, 3.0, 4.0, 5.0);
    test_float_method!(sin_val, sin, f32, std::f32::consts::PI / 2.0, 1.0);
    test_float_method!(cos_val, cos, f32, std::f32::consts::PI, -1.0);
    test_float_method!(tan_val, tan, f32, std::f32::consts::PI / 4.0, 1.0);
    #[test]
    fn asin_val() {
        let v: f32 = 1.0;
        let expected = std::f32::consts::PI / 2.0;
        let actual = v.asin();
        assert!((actual - expected).abs() < 1e-6);
    }
    test_float_method!(acos_val, acos, f32, 0.0, std::f32::consts::PI / 2.0);
    test_float_method!(atan_val, atan, f32, 1.0, std::f32::consts::PI / 4.0);
    test_float_method!(atan2_val, atan2, f32, 1.0, 1.0, std::f32::consts::PI / 4.0);
    #[test]
    fn sin_cos_val() {
        let v: f32 = std::f32::consts::PI / 2.0;
        let expected = (1.0, 0.0);
        let actual = v.sin_cos();
        assert!((actual.0 - expected.0).abs() < 1e-6);
        assert!((actual.1 - expected.1).abs() < 1e-6);
    }
    test_float_method!(exp_m1_val, exp_m1, f32, 0.0, 0.0);
    test_float_method!(ln_1p_val, ln_1p, f32, 0.0, 0.0);
    test_float_method!(sinh_val, sinh, f32, 0.0, 0.0);
    test_float_method!(cosh_val, cosh, f32, 0.0, 1.0);
    test_float_method!(tanh_val, tanh, f32, 0.0, 0.0);
    test_float_method!(asinh_val, asinh, f32, 0.0, 0.0);
    test_float_method!(acosh_val, acosh, f32, 1.0, 0.0);
    test_float_method!(atanh_val, atanh, f32, 0.0, 0.0);
    test_float_method!(copysign_pos, copysign, f32, 1.0, 2.0, 1.0);
    test_float_method!(copysign_neg, copysign, f32, 1.0, -2.0, -1.0);
    test_float_method!(clamp_val, clamp, f32, 1.5, 1.0, 2.0, 1.5);

    #[test]
    fn integer_decode_val() {
        let v: f32 = 2.0;
        let (mantissa, exponent, sign) = v.integer_decode();
        assert_eq!(mantissa, 8388608);
        assert_eq!(exponent, -22);
        assert_eq!(sign, 1);
    }
}

// Test for f64
mod f64_tests {
    use super::*;

    // Constants
    #[test]
    fn test_nan() {
        assert!(f64::nan().is_nan());
    }
    #[test]
    fn test_infinity() {
        assert!(f64::infinity().is_infinite());
    }
    #[test]
    fn test_neg_infinity() {
        assert!(f64::neg_infinity().is_infinite());
    }
    #[test]
    fn test_neg_zero() {
        assert_eq!(f64::neg_zero(), -0.0f64);
    }
    #[test]
    fn test_min_value() {
        assert_eq!(f64::min_value(), f64::MIN);
    }
    #[test]
    fn test_min_positive_value() {
        assert_eq!(f64::min_positive_value(), f64::MIN_POSITIVE);
    }
    #[test]
    fn test_epsilon() {
        assert_eq!(f64::epsilon(), f64::EPSILON);
    }
    #[test]
    fn test_max_value() {
        assert_eq!(f64::max_value(), f64::MAX);
    }

    // Methods
    test_float_method!(is_nan_true, is_nan, f64, f64::NAN, true);
    test_float_method!(is_nan_false, is_nan, f64, 1.0, false);
    test_float_method!(is_infinite_true, is_infinite, f64, f64::INFINITY, true);
    test_float_method!(is_infinite_false, is_infinite, f64, 1.0, false);
    test_float_method!(is_finite_true, is_finite, f64, 1.0, true);
    test_float_method!(is_finite_false, is_finite, f64, f64::INFINITY, false);
    test_float_method!(is_normal_true, is_normal, f64, 1.0, true);
    test_float_method!(is_normal_false, is_normal, f64, 0.0, false);
    test_float_method!(
        is_subnormal_true,
        is_subnormal,
        f64,
        f64::MIN_POSITIVE / 2.0,
        true
    );
    test_float_method!(is_subnormal_false, is_subnormal, f64, 1.0, false);
    test_float_method!(classify_normal, classify, f64, 1.0, FpCategory::Normal);
    test_float_method!(
        classify_infinite,
        classify,
        f64,
        f64::INFINITY,
        FpCategory::Infinite
    );
    test_float_method!(floor_val, floor, f64, 3.9, 3.0);
    test_float_method!(ceil_val, ceil, f64, 3.1, 4.0);
    test_float_method!(round_val, round, f64, 3.5, 4.0);
    test_float_method!(trunc_val, trunc, f64, 3.9, 3.0);
    test_float_method!(fract_val, fract, f64, 3.5, 0.5);
    test_float_method!(abs_val, abs, f64, -3.0, 3.0);
    test_float_method!(signum_pos, signum, f64, 3.0, 1.0);
    test_float_method!(signum_neg, signum, f64, -3.0, -1.0);
    test_float_method!(is_sign_positive_true, is_sign_positive, f64, 1.0, true);
    test_float_method!(is_sign_positive_false, is_sign_positive, f64, -1.0, false);
    test_float_method!(is_sign_negative_true, is_sign_negative, f64, -1.0, true);
    test_float_method!(is_sign_negative_false, is_sign_negative, f64, 1.0, false);
    test_float_method!(mul_add_val, mul_add, f64, 2.0, 3.0, 4.0, 10.0);
    test_float_method!(recip_val, recip, f64, 2.0, 0.5);

    #[test]
    fn test_f64_powi_val() {
        let v: f64 = 2.0;
        let n: i32 = 3;
        let expected: f64 = 8.0;
        let actual = v.powi(n);
        assert_eq!(actual, expected);
    }

    test_float_method!(powf_val, powf, f64, 2.0, 3.0, 8.0);
    test_float_method!(sqrt_val, sqrt, f64, 4.0, 2.0);
    test_float_method!(exp_val, exp, f64, 1.0, std::f64::consts::E);
    test_float_method!(exp2_val, exp2, f64, 3.0, 8.0);
    test_float_method!(ln_val, ln, f64, std::f64::consts::E, 1.0);
    test_float_method!(log_val, log, f64, 10.0, 10.0, 1.0);
    test_float_method!(log2_val, log2, f64, 8.0, 3.0);
    test_float_method!(log10_val, log10, f64, 100.0, 2.0);
    test_float_method!(to_degrees_val, to_degrees, f64, std::f64::consts::PI, 180.0);
    test_float_method!(to_radians_val, to_radians, f64, 180.0, std::f64::consts::PI);
    test_float_method!(max_val, max, f64, 1.0, 2.0, 2.0);
    test_float_method!(min_val, min, f64, 1.0, 2.0, 1.0);
    test_float_method!(cbrt_val, cbrt, f64, 8.0, 2.0);
    test_float_method!(hypot_val, hypot, f64, 3.0, 4.0, 5.0);
    test_float_method!(sin_val, sin, f64, std::f64::consts::PI / 2.0, 1.0);
    test_float_method!(cos_val, cos, f64, std::f64::consts::PI, -1.0);
    #[test]
    fn tan_val() {
        let v: f64 = std::f64::consts::PI / 4.0;
        let expected = 1.0;
        let actual = v.tan();
        assert!((actual - expected).abs() < 1e-15);
    }
    test_float_method!(asin_val, asin, f64, 1.0, std::f64::consts::PI / 2.0);
    test_float_method!(acos_val, acos, f64, 0.0, std::f64::consts::PI / 2.0);
    test_float_method!(atan_val, atan, f64, 1.0, std::f64::consts::PI / 4.0);
    test_float_method!(atan2_val, atan2, f64, 1.0, 1.0, std::f64::consts::PI / 4.0);
    #[test]
    fn sin_cos_val() {
        let v: f64 = std::f64::consts::PI / 2.0;
        let expected = (1.0, 0.0);
        let actual = v.sin_cos();
        assert!((actual.0 - expected.0).abs() < 1e-15);
        assert!((actual.1 - expected.1).abs() < 1e-15);
    }
    test_float_method!(exp_m1_val, exp_m1, f64, 0.0, 0.0);
    test_float_method!(ln_1p_val, ln_1p, f64, 0.0, 0.0);
    test_float_method!(sinh_val, sinh, f64, 0.0, 0.0);
    test_float_method!(cosh_val, cosh, f64, 0.0, 1.0);
    test_float_method!(tanh_val, tanh, f64, 0.0, 0.0);
    test_float_method!(asinh_val, asinh, f64, 0.0, 0.0);
    test_float_method!(acosh_val, acosh, f64, 1.0, 0.0);
    test_float_method!(atanh_val, atanh, f64, 0.0, 0.0);
    test_float_method!(copysign_pos, copysign, f64, 1.0, 2.0, 1.0);
    test_float_method!(copysign_neg, copysign, f64, 1.0, -2.0, -1.0);
    test_float_method!(clamp_val, clamp, f64, 1.5, 1.0, 2.0, 1.5);

    #[test]
    fn integer_decode_val() {
        let v: f64 = 2.0;
        let (mantissa, exponent, sign) = v.integer_decode();
        assert_eq!(mantissa, 4503599627370496);
        assert_eq!(exponent, -51);
        assert_eq!(sign, 1);
    }
}
