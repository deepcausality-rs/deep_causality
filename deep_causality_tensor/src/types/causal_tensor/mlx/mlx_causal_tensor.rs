/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! MLX-backed CausalTensor for Apple Silicon GPU acceleration.
//!
//! `MlxCausalTensor` provides a tensor that stores data directly in MLX's unified memory,
//! eliminating conversion overhead when MLX usage is known upfront.
//!
//! # Usage Patterns
//!
//! ## Scenario 1: Known MLX usage (no conversion overhead)
//! ```rust,ignore
//! let mlx_tensor = MlxCausalTensor::new_f32(data, shape)?;
//! let result = mlx_tensor.matmul(&other)?;
//! let output = result.to_causal_tensor()?;
//! ```
//!
//! ## Scenario 2: Bridge from existing CausalTensor
//! ```rust,ignore
//! let causal: CausalTensor<f64> = /* physics simulation */;
//! let mlx = MlxCausalTensor::from_causal_tensor_f64(&causal)?;  // downcast + copy
//! let result = mlx.matmul(&other)?;
//! ```
//!
//! # Precision Notes
//!
//! - `new_f32`: Direct f32, no conversion
//! - `new_from_f64`: **Downcasts f64 → f32** for GPU acceleration
//! - All operations return f32 results
//! - Upcast to f64 manually if precision is needed after GPU compute

use crate::CausalTensor;
use crate::CausalTensorError;

/// A CausalTensor backed by an MLX array for Apple Silicon GPU acceleration.
///
/// Unlike `CausalTensor<T>` which stores data in a Rust `Vec<T>`, this type
/// stores data directly in MLX's unified memory, enabling zero-copy GPU operations.
#[derive(Debug)]
pub struct MlxCausalTensor {
    array: mlx_rs::Array,
    shape: Vec<usize>,
}

impl MlxCausalTensor {
    /// Creates an MLX-backed tensor directly from f32 data.
    ///
    /// This is the most efficient constructor when you already have f32 data
    /// and know you want MLX acceleration. The data is copied once into MLX's
    /// unified memory.
    ///
    /// # Arguments
    /// * `data` - A vector of f32 values
    /// * `shape` - The shape of the tensor
    ///
    /// # Returns
    /// * `Ok(Self)` - The MLX-backed tensor
    /// * `Err(CausalTensorError::MlxConversionFailed)` - If MLX array creation fails
    pub fn new_f32(data: Vec<f32>, shape: Vec<usize>) -> Result<Self, CausalTensorError> {
        let mlx_shape: Vec<i32> = shape.iter().map(|&s| s as i32).collect();
        let array = mlx_rs::Array::from_slice(&data, &mlx_shape);
        Ok(Self { array, shape })
    }

    /// Creates an MLX-backed tensor from f64 data with precision downcast.
    ///
    /// **Warning:** This downcasts f64 → f32 for GPU acceleration.
    /// Metal GPU does not support f64 operations.
    ///
    /// Use this when:
    /// - GPU compute savings outweigh precision loss
    /// - Working with relative magnitudes (eigenvalue ratios, etc.)
    ///
    /// Do NOT use when:
    /// - Precision is critical (clock drift at 10⁻¹⁵ scale)
    /// - Accumulating over very large N
    ///
    /// # Arguments
    /// * `data` - A slice of f64 values (will be downcast to f32)
    /// * `shape` - The shape of the tensor
    pub fn new_from_f64(data: &[f64], shape: Vec<usize>) -> Result<Self, CausalTensorError> {
        let f32_data: Vec<f32> = data.iter().map(|&x| x as f32).collect();
        Self::new_f32(f32_data, shape)
    }

    /// Creates an MLX-backed tensor from an existing CausalTensor<f32>.
    ///
    /// Single copy: `Vec<f32>` → MLX unified memory.
    ///
    /// # Arguments
    /// * `tensor` - Reference to a CausalTensor<f32>
    pub fn from_causal_tensor(tensor: &CausalTensor<f32>) -> Result<Self, CausalTensorError> {
        Self::new_f32(tensor.data().to_vec(), tensor.shape().to_vec())
    }

    /// Creates an MLX-backed tensor from CausalTensor<f64> with downcast.
    ///
    /// **Warning:** This downcasts f64 → f32 for GPU acceleration.
    ///
    /// # Arguments
    /// * `tensor` - Reference to a CausalTensor<f64>
    pub fn from_causal_tensor_f64(tensor: &CausalTensor<f64>) -> Result<Self, CausalTensorError> {
        Self::new_from_f64(tensor.data(), tensor.shape().to_vec())
    }

    /// Converts back to CausalTensor<f32> after MLX operations.
    ///
    /// This forces evaluation of any lazy MLX operations and copies
    /// the data back to a Rust-managed `Vec<f32>`.
    ///
    /// # Returns
    /// * `Ok(CausalTensor<f32>)` - The tensor with computed values
    /// * `Err(CausalTensorError::MlxEvalFailed)` - If evaluation fails
    /// * `Err(CausalTensorError::MlxConversionFailed)` - If data extraction fails
    pub fn to_causal_tensor(&self) -> Result<CausalTensor<f32>, CausalTensorError> {
        self.array
            .eval()
            .map_err(|_| CausalTensorError::MlxEvalFailed)?;
        let data: Vec<f32> = self.array.as_slice::<f32>().to_vec();
        CausalTensor::new(data, self.shape.clone())
    }

    /// Access the underlying MLX array for advanced operations.
    ///
    /// Use this to access MLX operations not wrapped by this type.
    pub fn as_mlx_array(&self) -> &mlx_rs::Array {
        &self.array
    }

    /// Returns the shape of the tensor.
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    /// Returns the number of dimensions.
    pub fn ndim(&self) -> usize {
        self.shape.len()
    }

    /// Returns the total number of elements.
    pub fn len(&self) -> usize {
        self.shape.iter().product()
    }

    /// Returns true if the tensor is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// GPU-accelerated matrix multiplication.
    ///
    /// Computes `self @ other` using MLX's GPU-accelerated matmul.
    ///
    /// # Requirements
    /// - Both tensors must be at least 2D
    /// - Inner dimensions must match: self.shape[-1] == other.shape[-2]
    ///
    /// # Arguments
    /// * `other` - The right-hand side tensor
    ///
    /// # Returns
    /// * `Ok(MlxCausalTensor)` - The result of the matrix multiplication
    /// * `Err(CausalTensorError::MlxOperationFailed)` - If the operation fails
    pub fn matmul(&self, other: &MlxCausalTensor) -> Result<Self, CausalTensorError> {
        let result_array = mlx_rs::ops::matmul(&self.array, &other.array)
            .map_err(|e| CausalTensorError::MlxOperationFailed(format!("matmul failed: {}", e)))?;

        // Compute result shape
        let result_shape = Self::compute_matmul_shape(&self.shape, &other.shape)?;

        Ok(Self {
            array: result_array,
            shape: result_shape,
        })
    }

    /// Computes the output shape for matrix multiplication.
    fn compute_matmul_shape(lhs: &[usize], rhs: &[usize]) -> Result<Vec<usize>, CausalTensorError> {
        if lhs.len() < 2 || rhs.len() < 2 {
            return Err(CausalTensorError::MlxOperationFailed(
                "matmul requires at least 2D tensors".to_string(),
            ));
        }

        let lhs_inner = lhs[lhs.len() - 1];
        let rhs_inner = rhs[rhs.len() - 2];

        if lhs_inner != rhs_inner {
            return Err(CausalTensorError::MlxOperationFailed(format!(
                "matmul dimension mismatch: {} vs {}",
                lhs_inner, rhs_inner
            )));
        }

        // Result shape: [...lhs_batch, lhs[-2], rhs[-1]]
        let mut result_shape = lhs[..lhs.len() - 1].to_vec();
        result_shape.push(rhs[rhs.len() - 1]);
        Ok(result_shape)
    }

    /// GPU-accelerated Einstein summation.
    ///
    /// Computes an einsum operation using MLX's GPU-accelerated implementation.
    /// Uses NumPy-style subscript notation (e.g., "ij,jk->ik" for matmul).
    ///
    /// # Arguments
    /// * `subscripts` - Einstein summation subscript string (e.g., "ij,jk->ik")
    /// * `other` - The second operand tensor (for binary operations)
    ///
    /// # Examples
    /// ```rust,ignore
    /// // Matrix multiplication: C_ik = A_ij * B_jk
    /// let result = a.ein_sum("ij,jk->ik", &b)?;
    ///
    /// // Trace: sum of diagonal elements
    /// let trace = a.ein_sum_unary("ii->")?;
    ///
    /// // Transpose: B_ji = A_ij
    /// let transposed = a.ein_sum_unary("ij->ji")?;
    /// ```
    pub fn ein_sum(
        &self,
        subscripts: &str,
        other: &MlxCausalTensor,
    ) -> Result<Self, CausalTensorError> {
        let result_array =
            mlx_rs::ops::einsum(subscripts, [&self.array, &other.array]).map_err(|e| {
                CausalTensorError::MlxOperationFailed(format!(
                    "einsum '{}' failed: {}",
                    subscripts, e
                ))
            })?;

        // Get the result shape from the MLX array
        let result_shape: Vec<usize> = result_array.shape().iter().map(|&s| s as usize).collect();

        Ok(Self {
            array: result_array,
            shape: result_shape,
        })
    }

    /// Unary einsum operation (single tensor).
    ///
    /// # Examples
    /// ```rust,ignore
    /// let trace = a.ein_sum_unary("ii->")?;  // Trace
    /// let transposed = a.ein_sum_unary("ij->ji")?;  // Transpose
    /// ```
    pub fn ein_sum_unary(&self, subscripts: &str) -> Result<Self, CausalTensorError> {
        let result_array = mlx_rs::ops::einsum(subscripts, [&self.array]).map_err(|e| {
            CausalTensorError::MlxOperationFailed(format!("einsum '{}' failed: {}", subscripts, e))
        })?;

        let result_shape: Vec<usize> = result_array.shape().iter().map(|&s| s as usize).collect();

        Ok(Self {
            array: result_array,
            shape: result_shape,
        })
    }

    /// GPU-accelerated matrix transpose.
    ///
    /// Transposes the last two dimensions of the tensor.
    pub fn transpose(&self) -> Result<Self, CausalTensorError> {
        if self.shape.len() < 2 {
            return Err(CausalTensorError::MlxOperationFailed(
                "transpose requires at least 2D tensor".to_string(),
            ));
        }

        let result_array = mlx_rs::ops::transpose(&self.array).map_err(|e| {
            CausalTensorError::MlxOperationFailed(format!("transpose failed: {}", e))
        })?;

        // Reverse shape for transpose
        let mut result_shape = self.shape.clone();
        let n = result_shape.len();
        result_shape.swap(n - 2, n - 1);

        Ok(Self {
            array: result_array,
            shape: result_shape,
        })
    }

    /// GPU-accelerated matrix inverse.
    ///
    /// Computes the inverse of a square matrix.
    ///
    /// # Requirements
    /// - Tensor must be 2D (or batched 2D)
    /// - Matrix must be square
    /// - Matrix must be non-singular
    pub fn inverse(&self) -> Result<Self, CausalTensorError> {
        if self.shape.len() < 2 {
            return Err(CausalTensorError::MlxOperationFailed(
                "inverse requires at least 2D tensor".to_string(),
            ));
        }

        let n = self.shape.len();
        if self.shape[n - 2] != self.shape[n - 1] {
            return Err(CausalTensorError::MlxOperationFailed(format!(
                "inverse requires square matrix, got {}x{}",
                self.shape[n - 2],
                self.shape[n - 1]
            )));
        }

        let result_array = mlx_rs::linalg::inv(&self.array)
            .map_err(|e| CausalTensorError::MlxOperationFailed(format!("inverse failed: {}", e)))?;

        Ok(Self {
            array: result_array,
            shape: self.shape.clone(),
        })
    }

    /// GPU-accelerated element-wise addition.
    pub fn add(&self, other: &MlxCausalTensor) -> Result<Self, CausalTensorError> {
        let result_array = mlx_rs::ops::add(&self.array, &other.array)
            .map_err(|e| CausalTensorError::MlxOperationFailed(format!("add failed: {}", e)))?;

        let result_shape: Vec<usize> = result_array.shape().iter().map(|&s| s as usize).collect();

        Ok(Self {
            array: result_array,
            shape: result_shape,
        })
    }

    /// GPU-accelerated element-wise subtraction.
    pub fn sub(&self, other: &MlxCausalTensor) -> Result<Self, CausalTensorError> {
        let result_array = mlx_rs::ops::subtract(&self.array, &other.array).map_err(|e| {
            CausalTensorError::MlxOperationFailed(format!("subtract failed: {}", e))
        })?;

        let result_shape: Vec<usize> = result_array.shape().iter().map(|&s| s as usize).collect();

        Ok(Self {
            array: result_array,
            shape: result_shape,
        })
    }

    /// GPU-accelerated element-wise multiplication.
    pub fn mul(&self, other: &MlxCausalTensor) -> Result<Self, CausalTensorError> {
        let result_array = mlx_rs::ops::multiply(&self.array, &other.array).map_err(|e| {
            CausalTensorError::MlxOperationFailed(format!("multiply failed: {}", e))
        })?;

        let result_shape: Vec<usize> = result_array.shape().iter().map(|&s| s as usize).collect();

        Ok(Self {
            array: result_array,
            shape: result_shape,
        })
    }

    /// GPU-accelerated sum reduction over all elements.
    pub fn sum(&self) -> Result<f32, CausalTensorError> {
        let result = mlx_rs::ops::sum(&self.array, None)
            .map_err(|e| CausalTensorError::MlxOperationFailed(format!("sum failed: {}", e)))?;
        result
            .eval()
            .map_err(|_| CausalTensorError::MlxEvalFailed)?;
        let data: &[f32] = result.as_slice();
        Ok(data[0])
    }
}

impl Clone for MlxCausalTensor {
    fn clone(&self) -> Self {
        Self {
            array: self.array.clone(),
            shape: self.shape.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_f32() {
        let data = vec![1.0f32, 2.0, 3.0, 4.0];
        let shape = vec![2, 2];
        let tensor = MlxCausalTensor::new_f32(data, shape.clone());
        assert!(tensor.is_ok());
        let tensor = tensor.unwrap();
        assert_eq!(tensor.shape(), &shape);
        assert_eq!(tensor.len(), 4);
    }

    #[test]
    fn test_new_from_f64() {
        let data = vec![1.0f64, 2.0, 3.0, 4.0];
        let shape = vec![2, 2];
        let tensor = MlxCausalTensor::new_from_f64(&data, shape.clone());
        assert!(tensor.is_ok());
        let tensor = tensor.unwrap();
        assert_eq!(tensor.shape(), &shape);
    }

    #[test]
    fn test_roundtrip_f32() {
        let original_data = vec![1.0f32, 2.0, 3.0, 4.0];
        let shape = vec![2, 2];

        let mlx_tensor = MlxCausalTensor::new_f32(original_data.clone(), shape.clone()).unwrap();
        let causal_tensor = mlx_tensor.to_causal_tensor().unwrap();

        assert_eq!(causal_tensor.shape(), &shape);
        assert_eq!(causal_tensor.data(), &original_data);
    }

    #[test]
    fn test_matmul() {
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
}
