/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Trait definitions for the tensor crate.

// Core tensor trait (contains the Tensor trait for CausalTensor)
pub(crate) mod tensor;

// Backend traits
mod linear_algebra_backend;
mod tensor_backend;
mod tensor_data;

// Re-export backend traits
pub use linear_algebra_backend::LinearAlgebraBackend;
pub use tensor_backend::TensorBackend;
pub use tensor_data::TensorData;
