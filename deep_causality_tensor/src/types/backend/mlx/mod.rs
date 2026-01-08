/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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
mod mlx_tensor_impl;

use crate::types::backend::Device;

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
/// use deep_causality_tensor::{MlxBackend, TensorBackend};
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

pub use crate::types::mlx_tensor::MlxTensor;

/// Explicit type alias for MLX-backed CausalTensor.
/// Use this when you specifically require GPU acceleration.
pub type MlxCausalTensor<T> = crate::types::backend_tensor::BackendTensor<T, MlxBackend>;
