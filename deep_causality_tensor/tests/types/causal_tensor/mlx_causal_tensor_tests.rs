/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for MlxCausalTensor: GPU-accelerated tensor operations.
//!
//! Tests cover:
//! - Construction (new_f32, new_from_f64, from_causal_tensor)
//! - Conversion (to_causal_tensor)
//! - Operations (matmul, ein_sum, transpose, add, sub, mul, sum)
//! - Shape handling

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
mod mlx_causal_tensor_tests {
    use deep_causality_tensor::{CausalTensor, MlxCausalTensor};

    // ═══════════════════════════════════════════════════════════════════════
    // Constructor Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_new_f32_creates_tensor() {
        let data = vec![1.0f32, 2.0, 3.0, 4.0];
        let shape = vec![2, 2];
        let tensor = MlxCausalTensor::new_f32(data, shape.clone());
        assert!(tensor.is_ok());

        let tensor = tensor.unwrap();
        assert_eq!(tensor.shape(), &shape);
        assert_eq!(tensor.len(), 4);
        assert_eq!(tensor.ndim(), 2);
        assert!(!tensor.is_empty());
    }

    #[test]
    fn test_new_from_f64_downcasts_to_f32() {
        let data = vec![1.0f64, 2.0, 3.0, 4.0];
        let shape = vec![2, 2];
        let tensor = MlxCausalTensor::new_from_f64(&data, shape.clone());
        assert!(tensor.is_ok());

        let tensor = tensor.unwrap();
        assert_eq!(tensor.shape(), &shape);
    }

    #[test]
    fn test_from_causal_tensor_f32() {
        let causal = CausalTensor::new(vec![1.0f32, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let mlx = MlxCausalTensor::from_causal_tensor(&causal);
        assert!(mlx.is_ok());

        let mlx = mlx.unwrap();
        assert_eq!(mlx.shape(), causal.shape());
    }

    #[test]
    fn test_from_causal_tensor_f64() {
        let causal = CausalTensor::new(vec![1.0f64, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let mlx = MlxCausalTensor::from_causal_tensor_f64(&causal);
        assert!(mlx.is_ok());

        let mlx = mlx.unwrap();
        assert_eq!(mlx.shape(), causal.shape());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Conversion Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_roundtrip_f32_preserves_data() {
        let original_data = vec![1.0f32, 2.0, 3.0, 4.0];
        let shape = vec![2, 2];

        let mlx_tensor = MlxCausalTensor::new_f32(original_data.clone(), shape.clone()).unwrap();
        let causal_tensor = mlx_tensor.to_causal_tensor().unwrap();

        assert_eq!(causal_tensor.shape(), &shape);
        assert_eq!(causal_tensor.data(), &original_data);
    }

    #[test]
    fn test_roundtrip_f64_approximate_after_downcast() {
        let original_data = vec![1.5f64, 2.5, 3.5, 4.5];
        let shape = vec![2, 2];

        let mlx_tensor = MlxCausalTensor::new_from_f64(&original_data, shape.clone()).unwrap();
        let causal_tensor = mlx_tensor.to_causal_tensor().unwrap();

        assert_eq!(causal_tensor.shape(), &shape);
        // Values should be approximately equal after f64->f32 downcast
        for (i, &val) in causal_tensor.data().iter().enumerate() {
            assert!((val - original_data[i] as f32).abs() < 1e-6);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Matmul Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_matmul_2x3_times_3x2() {
        // 2x3 @ 3x2 = 2x2
        let lhs_data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        let lhs = MlxCausalTensor::new_f32(lhs_data, vec![2, 3]).unwrap();

        let rhs_data = vec![7.0f32, 8.0, 9.0, 10.0, 11.0, 12.0];
        let rhs = MlxCausalTensor::new_f32(rhs_data, vec![3, 2]).unwrap();

        let result = lhs.matmul(&rhs);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.shape(), &[2, 2]);
    }

    #[test]
    fn test_matmul_values_correct() {
        // [[1, 2], [3, 4]] @ [[5, 6], [7, 8]] = [[19, 22], [43, 50]]
        let a = MlxCausalTensor::new_f32(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let b = MlxCausalTensor::new_f32(vec![5.0, 6.0, 7.0, 8.0], vec![2, 2]).unwrap();

        let result = a.matmul(&b).unwrap();
        let output = result.to_causal_tensor().unwrap();

        let expected = vec![19.0f32, 22.0, 43.0, 50.0];
        for (i, &val) in output.data().iter().enumerate() {
            assert!((val - expected[i]).abs() < 1e-5);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Einsum Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_ein_sum_matmul() {
        let a = MlxCausalTensor::new_f32(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let b = MlxCausalTensor::new_f32(vec![5.0, 6.0, 7.0, 8.0], vec![2, 2]).unwrap();

        let result = a.ein_sum("ij,jk->ik", &b);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.shape(), &[2, 2]);
    }

    #[test]
    fn test_ein_sum_unary_transpose() {
        let a = MlxCausalTensor::new_f32(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();

        let result = a.ein_sum_unary("ij->ji");
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.shape(), &[3, 2]);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Transpose Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_transpose_2d() {
        let a = MlxCausalTensor::new_f32(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();

        let result = a.transpose();
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.shape(), &[3, 2]);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Element-wise Operation Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_add() {
        let a = MlxCausalTensor::new_f32(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let b = MlxCausalTensor::new_f32(vec![5.0, 6.0, 7.0, 8.0], vec![2, 2]).unwrap();

        let result = a.add(&b).unwrap();
        let output = result.to_causal_tensor().unwrap();

        let expected = vec![6.0f32, 8.0, 10.0, 12.0];
        for (i, &val) in output.data().iter().enumerate() {
            assert!((val - expected[i]).abs() < 1e-5);
        }
    }

    #[test]
    fn test_sub() {
        let a = MlxCausalTensor::new_f32(vec![5.0, 6.0, 7.0, 8.0], vec![2, 2]).unwrap();
        let b = MlxCausalTensor::new_f32(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();

        let result = a.sub(&b).unwrap();
        let output = result.to_causal_tensor().unwrap();

        let expected = vec![4.0f32, 4.0, 4.0, 4.0];
        for (i, &val) in output.data().iter().enumerate() {
            assert!((val - expected[i]).abs() < 1e-5);
        }
    }

    #[test]
    fn test_mul() {
        let a = MlxCausalTensor::new_f32(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let b = MlxCausalTensor::new_f32(vec![2.0, 2.0, 2.0, 2.0], vec![2, 2]).unwrap();

        let result = a.mul(&b).unwrap();
        let output = result.to_causal_tensor().unwrap();

        let expected = vec![2.0f32, 4.0, 6.0, 8.0];
        for (i, &val) in output.data().iter().enumerate() {
            assert!((val - expected[i]).abs() < 1e-5);
        }
    }

    #[test]
    fn test_sum() {
        let a = MlxCausalTensor::new_f32(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();

        let result = a.sum().unwrap();
        assert!((result - 10.0).abs() < 1e-5);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Clone Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_clone() {
        let a = MlxCausalTensor::new_f32(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let b = a.clone();

        assert_eq!(a.shape(), b.shape());
        assert_eq!(a.len(), b.len());

        // Verify values are the same
        let a_data = a.to_causal_tensor().unwrap();
        let b_data = b.to_causal_tensor().unwrap();
        assert_eq!(a_data.data(), b_data.data());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Edge Case Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_1d_tensor() {
        let a = MlxCausalTensor::new_f32(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
        assert_eq!(a.shape(), &[4]);
        assert_eq!(a.ndim(), 1);
    }

    #[test]
    fn test_3d_tensor() {
        let data: Vec<f32> = (0..24).map(|i| i as f32).collect();
        let a = MlxCausalTensor::new_f32(data, vec![2, 3, 4]).unwrap();
        assert_eq!(a.shape(), &[2, 3, 4]);
        assert_eq!(a.ndim(), 3);
        assert_eq!(a.len(), 24);
    }

    #[test]
    fn test_large_tensor() {
        let n = 100;
        let data: Vec<f32> = (0..n * n).map(|i| i as f32).collect();
        let a = MlxCausalTensor::new_f32(data, vec![n, n]).unwrap();
        assert_eq!(a.shape(), &[n, n]);
        assert_eq!(a.len(), n * n);
    }
}
