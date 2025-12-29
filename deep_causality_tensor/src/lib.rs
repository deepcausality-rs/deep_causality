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
pub use crate::traits::tensor::Tensor;
pub use crate::types::backend_tensor::BackendTensor;
pub use crate::types::cpu_tensor::InternalCpuTensor;
// Re-export generic EinSum types for advanced usage (e.g., benchmarks with different backends)
pub use crate::types::cpu_tensor::{EinSumAST as GenericEinSumAST, EinSumOp as GenericEinSumOp};

// --- Type Aliases ---

/// Default backend for CausalTensor.
/// Defaults to CPU. Use `mlx` feature for MLX backend (when available).
#[cfg(not(feature = "mlx"))]
pub type DefaultBackend = backend::CpuBackend;

#[cfg(feature = "mlx")]
pub type DefaultBackend = backend::CpuBackend; // Fallback until MlxBackend is ready

/// Public CausalTensor type, generic over backing data but using DefaultBackend.
pub type CausalTensor<T> = BackendTensor<T, DefaultBackend>;

/// Public EinSumOp, generic over T, matches CausalTensor<T>.
pub type EinSumOp<T> = GenericEinSumOp<CausalTensor<T>>;

/// Public EinSumAST, generic over T, matches CausalTensor<T>.
pub type EinSumAST<T> = GenericEinSumAST<CausalTensor<T>>;
pub use crate::utils::utils_tests;

// Re-export commonly used backend types at crate root for convenience
pub use backend::{CpuBackend, LinearAlgebraBackend, TensorBackend, TensorData};
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub use backend::{MlxBackend, MlxCausalTensor};
