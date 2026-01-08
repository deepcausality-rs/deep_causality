/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Type aliases for CausalMultiField with automatic backend selection.
//!
//! These aliases follow the pattern from `deep_causality_tensor`, selecting
//! the appropriate backend at compile-time based on feature flags.

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
use deep_causality_tensor::CpuBackend;

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
use deep_causality_tensor::MlxBackend;

/// Default backend for CausalMultiField.
///
/// - With `mlx` feature on macOS aarch64: Uses `MlxBackend` for GPU acceleration
/// - Otherwise: Uses `CpuBackend` for CPU computation
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub type DefaultMultiFieldBackend = MlxBackend;

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
pub type DefaultMultiFieldBackend = CpuBackend;

/// Default floating-point precision for CausalMultiField.
///
/// - With `mlx` feature: `f32` (GPU native precision)
/// - Otherwise: `f64` (Full precision for verification)
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub type DefaultMultiFieldFloat = f32;

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
pub type DefaultMultiFieldFloat = f64;
