/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Trait linking LinearAlgebraBackend to its GammaLoader.

use super::BackendGamma;
use deep_causality_tensor::{CpuBackend, LinearAlgebraBackend, TensorData};

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
use deep_causality_tensor::MlxBackend;

/// Provides access to the associated GammaLoader for a backend.
pub trait GammaProvider<T>: LinearAlgebraBackend
where
    T: TensorData,
{
    /// The specific GammaLoader implementation for this backend.
    type GammaLoader: BackendGamma<Self, T>;
}

// === Implementations ===

impl<T> GammaProvider<T> for CpuBackend
where
    T: TensorData + Clone + std::ops::Neg<Output = T>,
{
    type GammaLoader = super::cpu::CpuGammaLoader;
}

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
impl<T> GammaProvider<T> for MlxBackend
where
    T: TensorData + Clone + std::ops::Neg<Output = T>,
{
    type GammaLoader = super::mlx::MlxGammaLoader;
}
