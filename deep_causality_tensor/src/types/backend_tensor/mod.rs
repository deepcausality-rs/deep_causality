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

use crate::backend::{TensorBackend, TensorData};
use core::fmt::Debug;
use core::marker::PhantomData;

/// A backend-agnostic tensor wrapper.
///
/// This struct wraps a backend-specific tensor implementation (`B::Tensor<T>`) and provides
/// a unified API that delegates to the underlying backend `B`.
#[derive(Clone)]
pub struct BackendTensor<T, B>
where
    B: TensorBackend,
{
    pub(crate) inner: B::Tensor<T>,
    _backend: PhantomData<B>,
}

impl<T, B> Debug for BackendTensor<T, B>
where
    T: TensorData + Debug,
    B: TensorBackend,
    B::Tensor<T>: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("BackendTensor").field(&self.inner).finish()
    }
}

impl<T, B: TensorBackend> BackendTensor<T, B> {
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
