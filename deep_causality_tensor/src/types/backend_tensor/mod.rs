/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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

mod algebra;
mod constructors;
mod getters;
mod ops;

use crate::{TensorBackend, TensorData};
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

// Debug for CPU backend - uses InternalCpuTensor's derived Debug (only requires T: Debug)
impl<T: Debug> Debug for BackendTensor<T, crate::CpuBackend> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("CausalTensor").field(&self.inner).finish()
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

// PartialEq for CPU backend - uses InternalCpuTensor's derived PartialEq (no bounds on T)
impl<T: PartialEq> PartialEq for BackendTensor<T, crate::CpuBackend> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

// Deref to InternalCpuTensor for CPU backend.
// This gives direct access to all InternalCpuTensor methods that return references.
impl<T> core::ops::Deref for BackendTensor<T, crate::CpuBackend> {
    type Target = crate::InternalCpuTensor<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> core::ops::DerefMut for BackendTensor<T, crate::CpuBackend> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// Display for BackendTensor - uniform implementation
impl<T: core::fmt::Display + Clone, B: TensorBackend> core::fmt::Display for BackendTensor<T, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CausalTensor {{ data: [")?;
        let max_items = 10;
        let data = B::to_vec(&self.inner);
        for (i, item) in data.iter().take(max_items).enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        if data.len() > max_items {
            write!(f, ", ...")?;
        }
        write!(
            f,
            "], shape: {:?}, strides: {:?} }}",
            B::shape(&self.inner),
            B::strides(&self.inner)
        )
    }
}

// Default for CPU backend - creates empty tensor
impl<T: Default + Clone> Default for BackendTensor<T, crate::CpuBackend> {
    fn default() -> Self {
        Self::from_inner(crate::InternalCpuTensor::new(vec![], vec![0]).unwrap())
    }
}

// From<InternalCpuTensor> for convenience conversions
impl<T> From<crate::InternalCpuTensor<T>> for BackendTensor<T, crate::CpuBackend> {
    fn from(inner: crate::InternalCpuTensor<T>) -> Self {
        Self::from_inner(inner)
    }
}

// From scalar T to 0-D Tensor
impl<T> From<T> for BackendTensor<T, crate::CpuBackend>
where
    T: TensorData,
{
    fn from(scalar: T) -> Self {
        Self::from_inner(crate::InternalCpuTensor::new(vec![scalar], vec![]).unwrap())
    }
}

// From &T to 0-D Tensor
impl<T> From<&T> for BackendTensor<T, crate::CpuBackend>
where
    T: TensorData + Clone,
{
    fn from(scalar: &T) -> Self {
        Self::from_inner(crate::InternalCpuTensor::new(vec![*scalar], vec![]).unwrap())
    }
}
