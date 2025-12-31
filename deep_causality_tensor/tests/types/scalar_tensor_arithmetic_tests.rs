/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for scalar + tensor arithmetic operations (Add, Sub, Mul, Div)
//! covering all numeric types: f32, f64, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128

use deep_causality_tensor::InternalCpuTensor;

/// Macro to generate tests for scalar + tensor arithmetic for a given type
macro_rules! test_scalar_tensor_arithmetic {
    ($t:ty, $mod_name:ident) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn test_add_scalar_ref_tensor() {
                let tensor = InternalCpuTensor::<$t>::new(
                    vec![1 as $t, 2 as $t, 3 as $t, 4 as $t],
                    vec![2, 2],
                )
                .unwrap();
                let scalar: $t = 10 as $t;
                let result = scalar + &tensor;
                assert_eq!(result.shape(), &[2, 2]);
                assert_eq!(result.as_slice(), &[11 as $t, 12 as $t, 13 as $t, 14 as $t]);
            }

            #[test]
            fn test_add_scalar_owned_tensor() {
                let tensor = InternalCpuTensor::<$t>::new(
                    vec![1 as $t, 2 as $t, 3 as $t, 4 as $t],
                    vec![2, 2],
                )
                .unwrap();
                let scalar: $t = 10 as $t;
                let result = scalar + tensor;
                assert_eq!(result.shape(), &[2, 2]);
                assert_eq!(result.as_slice(), &[11 as $t, 12 as $t, 13 as $t, 14 as $t]);
            }

            #[test]
            fn test_sub_scalar_ref_tensor() {
                let tensor = InternalCpuTensor::<$t>::new(
                    vec![1 as $t, 2 as $t, 3 as $t, 4 as $t],
                    vec![2, 2],
                )
                .unwrap();
                let scalar: $t = 10 as $t;
                let result = scalar - &tensor;
                assert_eq!(result.shape(), &[2, 2]);
                assert_eq!(result.as_slice(), &[9 as $t, 8 as $t, 7 as $t, 6 as $t]);
            }

            #[test]
            fn test_sub_scalar_owned_tensor() {
                let tensor = InternalCpuTensor::<$t>::new(
                    vec![1 as $t, 2 as $t, 3 as $t, 4 as $t],
                    vec![2, 2],
                )
                .unwrap();
                let scalar: $t = 10 as $t;
                let result = scalar - tensor;
                assert_eq!(result.shape(), &[2, 2]);
                assert_eq!(result.as_slice(), &[9 as $t, 8 as $t, 7 as $t, 6 as $t]);
            }

            #[test]
            fn test_mul_scalar_ref_tensor() {
                let tensor = InternalCpuTensor::<$t>::new(
                    vec![1 as $t, 2 as $t, 3 as $t, 4 as $t],
                    vec![2, 2],
                )
                .unwrap();
                let scalar: $t = 2 as $t;
                let result = scalar * &tensor;
                assert_eq!(result.shape(), &[2, 2]);
                assert_eq!(result.as_slice(), &[2 as $t, 4 as $t, 6 as $t, 8 as $t]);
            }

            #[test]
            fn test_mul_scalar_owned_tensor() {
                let tensor = InternalCpuTensor::<$t>::new(
                    vec![1 as $t, 2 as $t, 3 as $t, 4 as $t],
                    vec![2, 2],
                )
                .unwrap();
                let scalar: $t = 2 as $t;
                let result = scalar * tensor;
                assert_eq!(result.shape(), &[2, 2]);
                assert_eq!(result.as_slice(), &[2 as $t, 4 as $t, 6 as $t, 8 as $t]);
            }

            #[test]
            fn test_div_scalar_ref_tensor() {
                let tensor = InternalCpuTensor::<$t>::new(
                    vec![1 as $t, 2 as $t, 4 as $t, 5 as $t],
                    vec![2, 2],
                )
                .unwrap();
                let scalar: $t = 10 as $t;
                let result = scalar / &tensor;
                assert_eq!(result.shape(), &[2, 2]);
                assert_eq!(result.as_slice(), &[10 as $t, 5 as $t, 2 as $t, 2 as $t]);
            }

            #[test]
            fn test_div_scalar_owned_tensor() {
                let tensor = InternalCpuTensor::<$t>::new(
                    vec![1 as $t, 2 as $t, 4 as $t, 5 as $t],
                    vec![2, 2],
                )
                .unwrap();
                let scalar: $t = 10 as $t;
                let result = scalar / tensor;
                assert_eq!(result.shape(), &[2, 2]);
                assert_eq!(result.as_slice(), &[10 as $t, 5 as $t, 2 as $t, 2 as $t]);
            }
        }
    };
}

// Generate tests for all integer types
test_scalar_tensor_arithmetic!(i8, i8_tests);
test_scalar_tensor_arithmetic!(i16, i16_tests);
test_scalar_tensor_arithmetic!(i32, i32_tests);
test_scalar_tensor_arithmetic!(i64, i64_tests);
test_scalar_tensor_arithmetic!(i128, i128_tests);
test_scalar_tensor_arithmetic!(u8, u8_tests);
test_scalar_tensor_arithmetic!(u16, u16_tests);
test_scalar_tensor_arithmetic!(u32, u32_tests);
test_scalar_tensor_arithmetic!(u64, u64_tests);
test_scalar_tensor_arithmetic!(u128, u128_tests);

// Float-specific tests with exact values
mod f32_tests {
    use super::*;

    #[test]
    fn test_add_scalar_ref_tensor() {
        let tensor = InternalCpuTensor::<f32>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let result = 10.0f32 + &tensor;
        assert_eq!(result.as_slice(), &[11.0, 12.0, 13.0, 14.0]);
    }

    #[test]
    fn test_add_scalar_owned_tensor() {
        let tensor = InternalCpuTensor::<f32>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let result = 10.0f32 + tensor;
        assert_eq!(result.as_slice(), &[11.0, 12.0, 13.0, 14.0]);
    }

    #[test]
    fn test_sub_scalar_ref_tensor() {
        let tensor = InternalCpuTensor::<f32>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let result = 10.0f32 - &tensor;
        assert_eq!(result.as_slice(), &[9.0, 8.0, 7.0, 6.0]);
    }

    #[test]
    fn test_sub_scalar_owned_tensor() {
        let tensor = InternalCpuTensor::<f32>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let result = 10.0f32 - tensor;
        assert_eq!(result.as_slice(), &[9.0, 8.0, 7.0, 6.0]);
    }

    #[test]
    fn test_mul_scalar_ref_tensor() {
        let tensor = InternalCpuTensor::<f32>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let result = 2.0f32 * &tensor;
        assert_eq!(result.as_slice(), &[2.0, 4.0, 6.0, 8.0]);
    }

    #[test]
    fn test_mul_scalar_owned_tensor() {
        let tensor = InternalCpuTensor::<f32>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let result = 2.0f32 * tensor;
        assert_eq!(result.as_slice(), &[2.0, 4.0, 6.0, 8.0]);
    }

    #[test]
    fn test_div_scalar_ref_tensor() {
        let tensor = InternalCpuTensor::<f32>::new(vec![1.0, 2.0, 4.0, 5.0], vec![2, 2]).unwrap();
        let result = 10.0f32 / &tensor;
        assert_eq!(result.as_slice(), &[10.0, 5.0, 2.5, 2.0]);
    }

    #[test]
    fn test_div_scalar_owned_tensor() {
        let tensor = InternalCpuTensor::<f32>::new(vec![1.0, 2.0, 4.0, 5.0], vec![2, 2]).unwrap();
        let result = 10.0f32 / tensor;
        assert_eq!(result.as_slice(), &[10.0, 5.0, 2.5, 2.0]);
    }
}

mod f64_tests {
    use super::*;

    #[test]
    fn test_add_scalar_ref_tensor() {
        let tensor = InternalCpuTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let result = 10.0f64 + &tensor;
        assert_eq!(result.as_slice(), &[11.0, 12.0, 13.0, 14.0]);
    }

    #[test]
    fn test_add_scalar_owned_tensor() {
        let tensor = InternalCpuTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let result = 10.0f64 + tensor;
        assert_eq!(result.as_slice(), &[11.0, 12.0, 13.0, 14.0]);
    }

    #[test]
    fn test_sub_scalar_ref_tensor() {
        let tensor = InternalCpuTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let result = 10.0f64 - &tensor;
        assert_eq!(result.as_slice(), &[9.0, 8.0, 7.0, 6.0]);
    }

    #[test]
    fn test_sub_scalar_owned_tensor() {
        let tensor = InternalCpuTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let result = 10.0f64 - tensor;
        assert_eq!(result.as_slice(), &[9.0, 8.0, 7.0, 6.0]);
    }

    #[test]
    fn test_mul_scalar_ref_tensor() {
        let tensor = InternalCpuTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let result = 2.0f64 * &tensor;
        assert_eq!(result.as_slice(), &[2.0, 4.0, 6.0, 8.0]);
    }

    #[test]
    fn test_mul_scalar_owned_tensor() {
        let tensor = InternalCpuTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let result = 2.0f64 * tensor;
        assert_eq!(result.as_slice(), &[2.0, 4.0, 6.0, 8.0]);
    }

    #[test]
    fn test_div_scalar_ref_tensor() {
        let tensor = InternalCpuTensor::<f64>::new(vec![1.0, 2.0, 4.0, 5.0], vec![2, 2]).unwrap();
        let result = 10.0f64 / &tensor;
        assert_eq!(result.as_slice(), &[10.0, 5.0, 2.5, 2.0]);
    }

    #[test]
    fn test_div_scalar_owned_tensor() {
        let tensor = InternalCpuTensor::<f64>::new(vec![1.0, 2.0, 4.0, 5.0], vec![2, 2]).unwrap();
        let result = 10.0f64 / tensor;
        assert_eq!(result.as_slice(), &[10.0, 5.0, 2.5, 2.0]);
    }
}
