/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;

// Macro to generate tests for scalar-tensor operations for a specific integer type.
macro_rules! test_scalar_tensor_ops_for_type {
    ($ty:ty, $test_name:ident) => {
        mod $test_name {
            use super::*;

            #[test]
            fn test_add() {
                let s: $ty = 10;
                let t = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();

                // Test scalar + &tensor
                let res1: CausalTensor<$ty> = s + &t;
                assert_eq!(res1.as_slice(), &[11, 12, 13]);

                // Test scalar + tensor
                let res2: CausalTensor<$ty> = s + t;
                assert_eq!(res2.as_slice(), &[11, 12, 13]);
            }

            #[test]
            fn test_sub() {
                let s: $ty = 10;
                let t = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();

                // Test scalar - &tensor
                let res1: CausalTensor<$ty> = s - &t;
                assert_eq!(res1.as_slice(), &[9, 8, 7]);

                // Test scalar - tensor
                let res2: CausalTensor<$ty> = s - t;
                assert_eq!(res2.as_slice(), &[9, 8, 7]);
            }

            #[test]
            fn test_mul() {
                let s: $ty = 3;
                let t = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();

                // Test scalar * &tensor
                let res1: CausalTensor<$ty> = s * &t;
                assert_eq!(res1.as_slice(), &[3, 6, 9]);

                // Test scalar * tensor
                let res2: CausalTensor<$ty> = s * t;
                assert_eq!(res2.as_slice(), &[3, 6, 9]);
            }

            #[test]
            fn test_div() {
                let s: $ty = 12;
                let t = CausalTensor::new(vec![2, 3, 4], vec![3]).unwrap();

                // Test scalar / &tensor
                let res1: CausalTensor<$ty> = s / &t;
                assert_eq!(res1.as_slice(), &[6, 4, 3]);

                // Test scalar / tensor
                let res2: CausalTensor<$ty> = s / t;
                assert_eq!(res2.as_slice(), &[6, 4, 3]);
            }
        }
    };
}

// --- Generate tests for all integer types ---
test_scalar_tensor_ops_for_type!(i8, test_i8);
test_scalar_tensor_ops_for_type!(i16, test_i16);
test_scalar_tensor_ops_for_type!(i32, test_i32);
test_scalar_tensor_ops_for_type!(i64, test_i64);
test_scalar_tensor_ops_for_type!(i128, test_i128);

test_scalar_tensor_ops_for_type!(u8, test_u8);
test_scalar_tensor_ops_for_type!(u16, test_u16);
test_scalar_tensor_ops_for_type!(u32, test_u32);
test_scalar_tensor_ops_for_type!(u64, test_u64);
test_scalar_tensor_ops_for_type!(u128, test_u128);

// --- Special handling for floats ---
mod test_f32 {
    use super::*;
    const TOLERANCE: f32 = 1e-6;

    #[test]
    fn test_add() {
        let s: f32 = 10.0;
        let t = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
        let expected = [11.0, 12.0, 13.0];

        let res1 = s + &t;
        res1.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));

        let res2 = s + t;
        res2.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));
    }

    #[test]
    fn test_sub() {
        let s: f32 = 10.0;
        let t = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
        let expected = [9.0, 8.0, 7.0];

        let res1 = s - &t;
        res1.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));

        let res2 = s - t;
        res2.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));
    }

    #[test]
    fn test_mul() {
        let s: f32 = 3.0;
        let t = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
        let expected = [3.0, 6.0, 9.0];

        let res1 = s * &t;
        res1.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));

        let res2 = s * t;
        res2.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));
    }

    #[test]
    fn test_div() {
        let s: f32 = 12.0;
        let t = CausalTensor::new(vec![2.0, 3.0, 4.0], vec![3]).unwrap();
        let expected = [6.0, 4.0, 3.0];

        let res1 = s / &t;
        res1.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));

        let res2 = s / t;
        res2.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));
    }
}

mod test_f64 {
    use super::*;
    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn test_add() {
        let s: f64 = 10.0;
        let t = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
        let expected = [11.0, 12.0, 13.0];

        let res1 = s + &t;
        res1.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));

        let res2 = s + t;
        res2.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));
    }

    #[test]
    fn test_sub() {
        let s: f64 = 10.0;
        let t = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
        let expected = [9.0, 8.0, 7.0];

        let res1 = s - &t;
        res1.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));

        let res2 = s - t;
        res2.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));
    }

    #[test]
    fn test_mul() {
        let s: f64 = 3.0;
        let t = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
        let expected = [3.0, 6.0, 9.0];

        let res1 = s * &t;
        res1.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));

        let res2 = s * t;
        res2.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));
    }

    #[test]
    fn test_div() {
        let s: f64 = 12.0;
        let t = CausalTensor::new(vec![2.0, 3.0, 4.0], vec![3]).unwrap();
        let expected = [6.0, 4.0, 3.0];

        let res1 = s / &t;
        res1.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));

        let res2 = s / t;
        res2.as_slice()
            .iter()
            .zip(expected.iter())
            .for_each(|(a, b)| assert!((a - b).abs() < TOLERANCE));
    }
}
