/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Trait definitions for the tensor crate.

// Core tensor trait (contains the Tensor trait for CausalTensor)
pub(crate) mod tensor;

// Backend traits
mod backend_linear_algebra;
mod backend_tensor;
mod tensor_data;

// Re-export backend traits
pub use backend_linear_algebra::LinearAlgebraBackend;
pub use backend_tensor::TensorBackend;
pub use tensor_data::TensorData;
