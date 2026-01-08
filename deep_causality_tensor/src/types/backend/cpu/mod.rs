/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CPU Backend implementation using existing `InternalCpuTensor<T>`.
//!
//! This backend wraps the pure Rust tensor implementation with no external
//! dependencies. It provides the reference implementation for correctness
//! verification and cross-platform fallback.

mod cpu_backend_linear_algebra;
mod cpu_backend_tensor;
mod cpu_tensor_impl;

use crate::Device;

/// CPU Backend using pure Rust `CpuTensor<T>`.
///
/// This backend provides:
/// - Full precision support (`f32`, `f64`, `Complex64`, etc.)
/// - No external dependencies
/// - Reference implementation for correctness verification
///
/// # Example
///
/// ```rust
/// use deep_causality_tensor::{CpuBackend, TensorBackend};
///
/// let data = vec![1.0f64, 2.0, 3.0, 4.0];
/// let tensor = CpuBackend::create(&data, &[2, 2]);
/// assert_eq!(CpuBackend::shape(&tensor), vec![2, 2]);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CpuBackend;

impl CpuBackend {
    /// Returns the device type for this backend.
    #[inline]
    pub const fn device() -> Device {
        Device::Cpu
    }
}
