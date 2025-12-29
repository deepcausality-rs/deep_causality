/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! MLX Backend for Apple Silicon GPU acceleration.
//!
//! This backend wraps `mlx-rs` to provide hardware-accelerated tensor operations
//! on Apple Silicon (M-series) chips. It uses Metal Performance Shaders under the hood.
//!
//! # Note
//!
//! - Only available on macOS aarch64 with the `mlx` feature enabled
//! - f64 tensors are downcast to f32 for GPU execution (Metal limitation)
//! - Provides conversion methods to/from `CausalTensor`
mod mlx_backend_linear_algebra;
mod mlx_backend_tensor;

use crate::backend::Device;

/// Apple Silicon GPU Backend using MLX.
///
/// This backend provides:
/// - GPU-accelerated tensor operations via Metal
/// - Unified memory architecture (no explicit data transfers)
/// - Optimized batch operations and matrix multiplication
///
/// # Precision
///
/// MLX operates natively in f32. When using f64 data:
/// - Data is downcast to f32 for GPU computation
/// - Results are upcast back to f64 if needed
///
/// # Example
///
/// ```rust,ignore
/// use deep_causality_tensor::backend::{MlxBackend, TensorBackend};
///
/// let data = vec![1.0f32, 2.0, 3.0, 4.0];
/// let tensor = MlxBackend::create(&data, &[2, 2]);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MlxBackend;

impl MlxBackend {
    /// Returns the device type for this backend.
    #[inline]
    pub const fn device() -> Device {
        Device::Gpu(0)
    }
}

/// Wrapper around `mlx_rs::Array` for type-safe tensor operations.
///
/// This wrapper preserves the element type information at the type level
/// while storing the actual MLX array internally.
#[derive(Clone)]
pub struct MlxTensor<T> {
    /// The underlying MLX array
    pub(crate) array: mlx_rs::Array,
    /// Phantom data for type safety
    _marker: core::marker::PhantomData<T>,
}

impl<T> MlxTensor<T> {
    /// Creates a new MlxTensor from an MLX array.
    pub fn new(array: mlx_rs::Array) -> Self {
        Self {
            array,
            _marker: core::marker::PhantomData,
        }
    }

    /// Returns a reference to the underlying MLX array.
    pub fn as_array(&self) -> &mlx_rs::Array {
        &self.array
    }

    /// Consumes self and returns the underlying MLX array.
    pub fn into_array(self) -> mlx_rs::Array {
        self.array
    }
}

// Implement Send + Sync for MlxTensor since MLX arrays are thread-safe
unsafe impl<T: Send> Send for MlxTensor<T> {}
unsafe impl<T: Sync> Sync for MlxTensor<T> {}
