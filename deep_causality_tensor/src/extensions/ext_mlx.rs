/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! MLX bridge module for Apple Silicon GPU acceleration.
//!
//! This module provides the `MlxCompatible` trait for converting Rust numeric types
//! to MLX arrays, enabling GPU-accelerated tensor operations on Apple Silicon.
//!
//! # Type Conversion Strategy
//!
//! - **f32**: Native MLX support, direct conversion
//! - **f64**: Downcast to f32 for GPU acceleration (Metal does not support f64)
//!
//! # Feature Flag
//!
//! This module is only available when the `mlx` feature is enabled and the
//! target is macOS aarch64 (Apple Silicon).

use crate::CausalTensorError;

/// Trait for types that can be converted to MLX arrays.
///
/// This trait bridges Rust's static generic types to MLX's dynamic Dtype system.
/// Implementations handle the conversion from Rust slices to MLX arrays.
pub trait MlxCompatible: Sized + Copy {
    /// Returns the MLX Dtype for GPU execution.
    ///
    /// Note: For f64, this returns Float32 since Metal GPU does not support f64.
    fn to_mlx_dtype() -> mlx_rs::Dtype;

    /// Converts a slice of values to an MLX array with the given shape.
    ///
    /// # Arguments
    /// * `data` - Slice of values to convert
    /// * `shape` - Shape of the resulting MLX array (as i32 for MLX API)
    ///
    /// # Returns
    /// * `Ok(Array)` - The MLX array
    /// * `Err(CausalTensorError::MlxConversionFailed)` - If conversion fails
    fn into_mlx_array(data: &[Self], shape: &[i32]) -> Result<mlx_rs::Array, CausalTensorError>;
}

/// Native f32 implementation - direct conversion to MLX.
impl MlxCompatible for f32 {
    fn to_mlx_dtype() -> mlx_rs::Dtype {
        mlx_rs::Dtype::Float32
    }

    fn into_mlx_array(data: &[Self], shape: &[i32]) -> Result<mlx_rs::Array, CausalTensorError> {
        Ok(mlx_rs::Array::from_slice(data, shape))
    }
}

/// f64 implementation with precision downcast.
///
/// **Warning**: This downcasts f64 → f32 for GPU acceleration.
/// Metal GPU does not support f64 operations. Results are returned as f32.
///
/// Use this when:
/// - Compute savings outweigh precision loss
/// - Working with relative magnitudes (e.g., eigenvalue ratios)
/// - Bulk matrix operations (matmul, einsum)
///
/// Do NOT use when:
/// - Accumulating over large N (precision loss compounds)
/// - Working with very small differences (e.g., clock drift ~10⁻¹⁵)
impl MlxCompatible for f64 {
    fn to_mlx_dtype() -> mlx_rs::Dtype {
        // Return f32 dtype since Metal GPU does not support f64
        mlx_rs::Dtype::Float32
    }

    fn into_mlx_array(data: &[Self], shape: &[i32]) -> Result<mlx_rs::Array, CausalTensorError> {
        // Downcast f64 → f32 for GPU acceleration
        let f32_data: Vec<f32> = data.iter().map(|&x| x as f32).collect();
        Ok(mlx_rs::Array::from_slice(&f32_data, shape))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_dtype() {
        assert_eq!(f32::to_mlx_dtype(), mlx_rs::Dtype::Float32);
    }

    #[test]
    fn test_f64_dtype_returns_f32() {
        // f64 should return Float32 dtype for GPU compatibility
        assert_eq!(f64::to_mlx_dtype(), mlx_rs::Dtype::Float32);
    }

    #[test]
    fn test_f32_conversion() {
        let data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let shape = vec![2, 2];
        let mlx_shape: Vec<i32> = shape.iter().map(|&s| s as i32).collect();
        let result = f32::into_mlx_array(&data, &mlx_shape);
        assert!(result.is_ok());
    }

    #[test]
    fn test_f64_conversion_with_downcast() {
        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
        let shape = vec![2, 2];
        let mlx_shape: Vec<i32> = shape.iter().map(|&s| s as i32).collect();
        let result = f64::into_mlx_array(&data, &mlx_shape);
        assert!(result.is_ok());

        // Verify the resulting array has Float32 dtype
        let array = result.unwrap();
        assert_eq!(array.dtype(), mlx_rs::Dtype::Float32);
    }
}
