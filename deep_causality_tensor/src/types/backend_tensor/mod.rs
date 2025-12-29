/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Generic tensor type that dispatches through a TensorBackend.
//!
//! `BackendTensor<T, B>` wraps the backend's native tensor representation and provides
//! a unified API for tensor operations that can execute on different hardware.
//!
//! # Example
//!
//! ```rust,ignore
//! use deep_causality_tensor::{BackendTensor, CpuBackend};
//!
//! // Create a tensor on CPU
//! let t: BackendTensor<f64, CpuBackend> = BackendTensor::zeros(&[2, 3]);
//!
//! // Create a tensor on MLX (macOS only)
//! #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
//! let t_gpu: BackendTensor<f64, MlxBackend> = BackendTensor::zeros(&[2, 3]);
//! ```

mod constructors;
mod getters;
mod linear_algebra;
mod ops;

use crate::traits::{TensorBackend, TensorData};
use core::marker::PhantomData;

/// A tensor that dispatches operations through a `TensorBackend`.
///
/// This provides hardware-agnostic tensor operations by delegating to the
/// backend's implementation, enabling the same code to run on CPU, GPU (MLX),
/// or other accelerators.
///
/// # Type Parameters
///
/// * `T` - Element type (f32, f64, etc.)
/// * `B` - Backend type implementing `TensorBackend`
#[derive(Clone)]
pub struct BackendTensor<T: TensorData, B: TensorBackend> {
    /// The underlying backend-specific tensor.
    pub(crate) inner: B::Tensor<T>,
    /// Phantom marker for the backend type.
    _backend: PhantomData<B>,
}

impl<T: TensorData, B: TensorBackend> BackendTensor<T, B> {
    /// Creates a `BackendTensor` from a backend-native tensor.
    pub fn from_inner(inner: B::Tensor<T>) -> Self {
        Self {
            inner,
            _backend: PhantomData,
        }
    }

    /// Returns a reference to the underlying backend tensor.
    pub fn inner(&self) -> &B::Tensor<T> {
        &self.inner
    }

    /// Consumes self and returns the underlying backend tensor.
    pub fn into_inner(self) -> B::Tensor<T> {
        self.inner
    }
}

// Debug implementation
impl<T: TensorData + core::fmt::Debug, B: TensorBackend> core::fmt::Debug for BackendTensor<T, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let data = B::to_vec(&self.inner);
        let shape = B::shape(&self.inner);
        f.debug_struct("BackendTensor")
            .field("shape", &shape)
            .field("data", &data)
            .finish()
    }
}

// PartialEq implementation
impl<T: TensorData + PartialEq, B: TensorBackend> PartialEq for BackendTensor<T, B> {
    fn eq(&self, other: &Self) -> bool {
        let self_shape = B::shape(&self.inner);
        let other_shape = B::shape(&other.inner);
        if self_shape != other_shape {
            return false;
        }
        B::to_vec(&self.inner) == B::to_vec(&other.inner)
    }
}
