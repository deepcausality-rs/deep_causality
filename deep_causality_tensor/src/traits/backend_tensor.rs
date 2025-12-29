/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! TensorBackend trait defining the compute backend contract.

use super::tensor_data::TensorData;
use crate::backend::Device;
use core::ops::Range;

/// Defines the compute backend contract for tensor operations.
///
/// This trait abstracts over hardware execution (CPU, MLX, CUDA), enabling
/// backend-agnostic physics code while providing precision and performance
/// flexibility.
///
/// # Implementing a Backend
///
/// Each backend must implement:
/// 1. **Creation:** `create`, `zeros`, `ones`, `from_shape_fn`
/// 2. **Arithmetic:** `add`, `sub`, `mul`, `div`
/// 3. **Shape ops:** `reshape`, `permute`, `slice`
/// 4. **Data access:** `to_vec`, `shape`
/// 5. **Reduction:** `sum`, `max`
///
/// # Example
///
/// ```rust
/// use deep_causality_tensor::backend::{CpuBackend, TensorBackend};
///
/// let data = vec![1.0f64, 2.0, 3.0, 4.0];
/// let a = CpuBackend::create(&data, &[2, 2]);
/// let b = CpuBackend::create(&data, &[2, 2]);
/// let c = CpuBackend::add(&a, &b);
/// ```
pub trait TensorBackend: Clone + Send + Sync + 'static {
    /// The concrete tensor type used by this backend.
    type Tensor<T: TensorData>: Clone + Send + Sync;

    /// Returns the device this backend operates on.
    fn device() -> Device;

    // --- Creation ---

    /// Creates a tensor from data with the given shape.
    ///
    /// # Arguments
    /// * `data` - Flat array of elements in row-major order
    /// * `shape` - Dimensions of the tensor
    ///
    /// # Panics
    /// Panics if `data.len() != shape.iter().product()`
    fn create<T: TensorData>(data: &[T], shape: &[usize]) -> Self::Tensor<T>;

    /// Creates a tensor filled with zeros.
    fn zeros<T: TensorData>(shape: &[usize]) -> Self::Tensor<T>;

    /// Creates a tensor filled with ones.
    fn ones<T: TensorData>(shape: &[usize]) -> Self::Tensor<T>;

    /// Creates a tensor from a function applied to each index.
    fn from_shape_fn<T: TensorData, F>(shape: &[usize], f: F) -> Self::Tensor<T>
    where
        F: FnMut(&[usize]) -> T;

    // --- Data Access ---

    /// Downloads tensor data to a host vector.
    fn to_vec<T: TensorData>(tensor: &Self::Tensor<T>) -> Vec<T>;

    /// Returns the shape of the tensor.
    fn shape<T: TensorData>(tensor: &Self::Tensor<T>) -> Vec<usize>;

    // --- Shape Manipulation ---

    /// Reshapes the tensor without copying data (if possible).
    fn reshape<T: TensorData>(tensor: &Self::Tensor<T>, shape: &[usize]) -> Self::Tensor<T>;

    /// Permutes the axes of the tensor.
    fn permute<T: TensorData>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T>;

    /// Extracts a slice of the tensor.
    fn slice<T: TensorData>(tensor: &Self::Tensor<T>, ranges: &[Range<usize>]) -> Self::Tensor<T>;

    // --- Element-wise Arithmetic ---

    /// Element-wise addition.
    fn add<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>;

    /// Element-wise subtraction.
    fn sub<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>;

    /// Element-wise multiplication.
    fn mul<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>;

    /// Element-wise division.
    fn div<T: TensorData>(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>;

    // --- Reduction ---

    /// Sums elements along specified axes.
    fn sum<T: TensorData>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T>;

    /// Finds maximum along specified axes.
    fn max<T: TensorData>(tensor: &Self::Tensor<T>, axes: &[usize]) -> Self::Tensor<T>;
}
