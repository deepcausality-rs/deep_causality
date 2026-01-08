/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Backend Module
//!
//! This module defines the generic backend architecture for tensor operations.
//! It enables hardware-agnostic code that can run on CPU, MLX (Apple Silicon),
//! or CUDA (NVIDIA) backends.

pub(crate) mod cpu;

// Feature-gated MLX backend
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub(crate) mod mlx;

/// Represents the device where tensor computations occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
    /// CPU-based computation
    Cpu,
    /// GPU-based computation (index identifies which GPU)
    #[allow(dead_code)]
    Gpu(usize),
}
