/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the MlxCompatible trait and its implementations.
//!
//! These tests verify:
//! - f32 native conversion to MLX arrays
//! - f64 downcast conversion to f32 for GPU compatibility
//! - Correct Dtype reporting

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
mod mlx_compatible_tests {
    use deep_causality_tensor::MlxCompatible;

    #[test]
    fn test_f32_dtype_returns_float32() {
        assert_eq!(f32::to_mlx_dtype(), mlx_rs::Dtype::Float32);
    }

    #[test]
    fn test_f64_dtype_returns_float32_for_gpu_compat() {
        // f64 should return Float32 dtype because Metal GPU does not support f64
        assert_eq!(f64::to_mlx_dtype(), mlx_rs::Dtype::Float32);
    }

    #[test]
    fn test_f32_into_mlx_array_succeeds() {
        let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let shape: Vec<i32> = vec![2, 2];
        let result = f32::into_mlx_array(&data, &shape);
        assert!(result.is_ok());

        let array = result.unwrap();
        assert_eq!(array.shape(), &[2, 2]);
        assert_eq!(array.dtype(), mlx_rs::Dtype::Float32);
    }

    #[test]
    fn test_f64_into_mlx_array_downcasts_to_f32() {
        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
        let shape: Vec<i32> = vec![2, 2];
        let result = f64::into_mlx_array(&data, &shape);
        assert!(result.is_ok());

        let array = result.unwrap();
        assert_eq!(array.shape(), &[2, 2]);
        // Verify the dtype is f32, not f64
        assert_eq!(array.dtype(), mlx_rs::Dtype::Float32);
    }

    #[test]
    fn test_f32_conversion_preserves_values() {
        let data: Vec<f32> = vec![1.5, 2.5, 3.5, 4.5];
        let shape: Vec<i32> = vec![4];
        let result = f32::into_mlx_array(&data, &shape).unwrap();

        result.eval().unwrap();
        let retrieved: &[f32] = result.as_slice();
        assert_eq!(retrieved, &[1.5, 2.5, 3.5, 4.5]);
    }

    #[test]
    fn test_f64_conversion_approximate_values_after_downcast() {
        // Large f64 values that fit in f32
        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
        let shape: Vec<i32> = vec![2, 2];
        let result = f64::into_mlx_array(&data, &shape).unwrap();

        result.eval().unwrap();
        let retrieved: &[f32] = result.as_slice();

        // Values should be approximately equal after downcast
        for (i, &val) in retrieved.iter().enumerate() {
            assert!((val - (i as f32 + 1.0)).abs() < 1e-6);
        }
    }

    #[test]
    fn test_empty_f32_array() {
        let data: Vec<f32> = vec![];
        let shape: Vec<i32> = vec![0];
        let result = f32::into_mlx_array(&data, &shape);
        assert!(result.is_ok());
    }

    #[test]
    fn test_3d_shape_f32() {
        let data: Vec<f32> = vec![1.0; 24]; // 2x3x4 = 24
        let shape: Vec<i32> = vec![2, 3, 4];
        let result = f32::into_mlx_array(&data, &shape);
        assert!(result.is_ok());

        let array = result.unwrap();
        assert_eq!(array.shape(), &[2, 3, 4]);
    }
}
