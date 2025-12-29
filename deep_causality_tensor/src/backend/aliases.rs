/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Default type aliases for backend-agnostic code.
//!
//! These aliases enable users to write code that automatically selects
//! the optimal backend based on compile-time feature flags.

use crate::CpuBackend;

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
use super::MlxBackend;

/// Default backend selection via feature flags.
///
/// - `mlx` feature on Apple Silicon: Uses `MlxBackend`
/// - `cuda` feature: Uses `CudaBackend` (future)
/// - Default: Uses `CpuBackend`
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub type DefaultBackend = MlxBackend;

#[cfg(not(any(
    all(feature = "mlx", target_os = "macos", target_arch = "aarch64"),
    feature = "cuda"
)))]
pub type DefaultBackend = CpuBackend;

/// Default floating-point precision matching the backend.
///
/// - `mlx` feature: `f32` (GPU native precision)
/// - Default: `f64` (Full precision for physics verification)
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub type DefaultFloat = f32;

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
pub type DefaultFloat = f64;

/// Convenience type alias for the "just works" tensor.
///
/// Uses the default backend and precision based on feature flags.
///
/// # Example
///
/// ```rust
/// use deep_causality_tensor::backend::Tensor;
///
/// // Automatically uses CpuBackend with f64 (or MlxBackend with f32)
/// let t = Tensor::zero(&[3, 3]);
/// ```
pub type Tensor = crate::CausalTensor<DefaultFloat>;
