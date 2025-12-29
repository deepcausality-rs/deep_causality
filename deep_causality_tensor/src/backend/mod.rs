/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Backend Module
//!
//! This module defines the generic backend architecture for tensor operations.
//! It enables hardware-agnostic code that can run on CPU, MLX (Apple Silicon),
//! or CUDA (NVIDIA) backends.
//!
//! ## Architecture
//!
//! - [`TensorBackend`] - Core trait for tensor creation and manipulation
//! - [`LinearAlgebraBackend`] - Extended trait for matrix operations (matmul, SVD, etc.)
//! - [`CpuBackend`] - Pure Rust implementation using existing `CausalTensor<T>`
//! - [`MlxBackend`] - Apple Silicon GPU acceleration (feature-gated)
//!
//! ## Usage
//!
//! ```rust
//! use deep_causality_tensor::backend::{CpuBackend, TensorBackend};
//!
//! // Create a tensor using the CPU backend
//! let data = vec![1.0, 2.0, 3.0, 4.0];
//! let tensor = CpuBackend::create(&data, &[2, 2]);
//! ```

mod aliases;
mod cpu;

// Feature-gated MLX backend
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
mod mlx;

// Re-export traits
pub use crate::traits::{LinearAlgebraBackend, TensorBackend, TensorData};

// Re-export backends
pub use cpu::CpuBackend;

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub use mlx::{MlxBackend, MlxCausalTensor};

// Re-export type aliases
pub use aliases::{DefaultBackend, DefaultFloat, Tensor};

/// Represents the device where tensor computations occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
    /// CPU-based computation
    Cpu,
    /// GPU-based computation (index identifies which GPU)
    #[allow(dead_code)]
    Gpu(usize),
}
