/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Getter methods for BackendTensor.
//!
//! For CPU backend, most getters are accessed via Deref to InternalCpuTensor.
//! This file defines only methods that aren't available on InternalCpuTensor
//! or need backend-specific implementations.

use super::BackendTensor;
use crate::TensorBackend;

impl<T, B: TensorBackend> BackendTensor<T, B> {
    /// Consumes the tensor and returns the storage as a vector.
    pub fn into_vec(self) -> Vec<T> {
        B::into_vec(self.into_inner())
    }
}

impl<T: Clone, B: TensorBackend> BackendTensor<T, B> {
    /// Downloads tensor data to a host vector.
    pub fn to_vec(&self) -> Vec<T> {
        B::to_vec(&self.inner)
    }
}
