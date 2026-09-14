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
                let res1 = s + &t;
                assert_eq!(res1.as_slice(), &[11, 12, 13]);

                // Test scalar + tensor
                let res2 = s + t;
                assert_eq!(res2.as_slice(), &[11, 12, 13]);
            }

            #[test]
            fn test_sub() {
                let s: $ty = 10;
                let t = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();

                // Test scalar - &tensor
                let res1 = s - &t;
                assert_eq!(res1.as_slice(), &[9, 8, 7]);

                // Test scalar - tensor
                let res2 = s - t;
                assert_eq!(res2.as_slice(), &[9, 8, 7]);
            }

            #[test]
            fn test_mul() {
                let s: $ty = 3;
                let t = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();

                // Test scalar * &tensor
                let res1 = s * &t;
                assert_eq!(res1.as_slice(), &[3, 6, 9]);

                // Test scalar * tensor
                let res2 = s * t;
                assert_eq!(res2.as_slice(), &[3, 6, 9]);
            }

            #[test]
            fn test_div() {
                let s: $ty = 12;
                let t = CausalTensor::new(vec![2, 3, 4], vec![3]).unwrap();

                // Test scalar / &tensor
                let res1 = s / &t;
                assert_eq!(res1.as_slice(), &[6, 4, 3]);

                // Test scalar / tensor
                let res2 = s / t;
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

// --- The software scalars ---
//
// `Float106` and `BFloat16` reach the operators through the same concrete impls as `f32` and
// `f64`, because a blanket `impl<T> Mul<CausalTensor<T>> for T` is forbidden by the orphan rule.
// Every value below is a small integer, exact in all four types including `BFloat16`'s eight-bit
// significand, so each result is compared exactly rather than against a tolerance.
macro_rules! test_scalar_tensor_ops_for_software_scalar {
    ($ty:ty, $test_name:ident) => {
        mod $test_name {
            use super::*;
            use deep_causality_num::lift;

            fn tensor(values: [f64; 3]) -> CausalTensor<$ty> {
                CausalTensor::new(values.iter().map(|&v| lift::<$ty>(v)).collect(), vec![3])
                    .unwrap()
            }

            fn assert_exact(got: &CausalTensor<$ty>, want: [f64; 3]) {
                let want: Vec<$ty> = want.iter().map(|&v| lift::<$ty>(v)).collect();
                assert_eq!(got.as_slice(), want.as_slice());
            }

            #[test]
            fn test_add() {
                let s = lift::<$ty>(10.0);
                assert_exact(&(s + &tensor([1.0, 2.0, 3.0])), [11.0, 12.0, 13.0]);
                assert_exact(&(s + tensor([1.0, 2.0, 3.0])), [11.0, 12.0, 13.0]);
            }

            #[test]
            fn test_sub() {
                // Subtraction is not commutative, so this also pins the operand order: the
                // scalar is the left operand and the tensor's entries are subtracted from it.
                let s = lift::<$ty>(10.0);
                assert_exact(&(s - &tensor([1.0, 2.0, 3.0])), [9.0, 8.0, 7.0]);
                assert_exact(&(s - tensor([1.0, 2.0, 3.0])), [9.0, 8.0, 7.0]);
            }

            #[test]
            fn test_mul() {
                let s = lift::<$ty>(3.0);
                assert_exact(&(s * &tensor([1.0, 2.0, 3.0])), [3.0, 6.0, 9.0]);
                assert_exact(&(s * tensor([1.0, 2.0, 3.0])), [3.0, 6.0, 9.0]);
            }

            #[test]
            fn test_div() {
                // Division is not commutative either: the scalar is the numerator.
                let s = lift::<$ty>(12.0);
                assert_exact(&(s / &tensor([2.0, 3.0, 4.0])), [6.0, 4.0, 3.0]);
                assert_exact(&(s / tensor([2.0, 3.0, 4.0])), [6.0, 4.0, 3.0]);
            }

            #[test]
            fn test_shape_is_preserved() {
                // The result carries the operand's shape rather than flattening it.
                let t =
                    CausalTensor::new((1..=6).map(|i| lift::<$ty>(i as f64)).collect(), vec![2, 3])
                        .unwrap();
                let scaled = lift::<$ty>(2.0) * &t;
                assert_eq!(scaled.shape(), &[2, 3]);
                assert_eq!(scaled.as_slice()[5], lift::<$ty>(12.0));
            }
        }
    };
}

test_scalar_tensor_ops_for_software_scalar!(deep_causality_num::Float106, test_float106);
test_scalar_tensor_ops_for_software_scalar!(deep_causality_num::BFloat16, test_bfloat16);
