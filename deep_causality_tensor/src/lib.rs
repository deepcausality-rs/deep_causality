/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # deep_causality_tensor
//!
//! Hardware-agnostic tensor library with backend abstraction for CPU and GPU computation.

mod errors;
pub mod extensions;
mod traits;
mod types;
mod utils;

// === Errors ===
pub use crate::errors::causal_tensor_error::CausalTensorError;
pub use crate::errors::ein_sum_validation_error::EinSumValidationError;

// === Extensions ===
pub use crate::extensions::ext_hkt::CausalTensorWitness;
pub use crate::extensions::ext_math::CausalTensorMathExt;

// === Traits ===
pub use crate::traits::backend_linear_algebra::LinearAlgebraBackend;
pub use crate::traits::backend_tensor::TensorBackend;
pub use crate::traits::tensor::Tensor;
pub use crate::traits::tensor_data::TensorData;

// === Types ===
pub use crate::types::backend_tensor::BackendTensor;
pub use crate::types::cpu_tensor::InternalCpuTensor;
pub use crate::types::cpu_tensor::{EinSumAST as GenericEinSumAST, EinSumOp as GenericEinSumOp};

// === Backend Types ===
pub use crate::types::backend::Device;
pub use crate::types::backend::cpu::CpuBackend;
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub use crate::types::backend::mlx::{MlxBackend, MlxCausalTensor};

// === Utils (test support) ===
pub use crate::utils::utils_tests;

// === Type Aliases ===

/// Default backend for CausalTensor.
/// Defaults to CPU. Use `mlx` feature for MLX backend (when available).
#[cfg(not(feature = "mlx"))]
pub type DefaultBackend = CpuBackend;

#[cfg(feature = "mlx")]
pub type DefaultBackend = CpuBackend; // Fallback until MlxBackend is ready

/// Default floating-point precision matching the backend.
/// - `mlx` feature: `f32` (GPU native precision)
/// - Default: `f64` (Full precision for physics verification)
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub type DefaultFloat = f32;

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
pub type DefaultFloat = f64;

/// Public CausalTensor type, generic over backing data but using DefaultBackend.
pub type CausalTensor<T> = BackendTensor<T, DefaultBackend>;

/// Public EinSumOp, generic over T, matches CausalTensor<T>.
pub type EinSumOp<T> = GenericEinSumOp<CausalTensor<T>>;

/// Public EinSumAST, generic over T, matches CausalTensor<T>.
pub type EinSumAST<T> = GenericEinSumAST<CausalTensor<T>>;
