/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod errors;
mod extensions;
mod traits;
mod types;
mod utils;

/// Generic backend architecture for hardware-agnostic tensor operations.
///
/// This module provides:
/// - [`backend::TensorBackend`] - Core trait for tensor creation and manipulation
/// - [`backend::LinearAlgebraBackend`] - Extended trait for matrix operations
/// - [`backend::CpuBackend`] - Pure Rust implementation (reference)
/// - [`backend::MlxBackend`] - Apple Silicon GPU acceleration (feature-gated)
pub mod backend;

// Causal tensor type
pub use crate::errors::causal_tensor_error::CausalTensorError;
pub use crate::errors::ein_sum_validation_error::EinSumValidationError;
pub use crate::extensions::ext_hkt::CausalTensorWitness;
pub use crate::extensions::ext_math::CausalTensorMathExt;
pub use crate::extensions::ext_stack::CausalTensorStackExt;
pub use crate::traits::tensor::Tensor;
pub use crate::types::causal_tensor::{CausalTensor, EinSumAST, EinSumOp};
pub use crate::utils::utils_tests;

// Re-export commonly used backend types at crate root for convenience
pub use backend::{CpuBackend, LinearAlgebraBackend, TensorBackend, TensorData};
