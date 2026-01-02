/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod alias;
mod errors;
mod extensions;
mod traits;
mod types;

// Aliases
pub use crate::alias::{
    ComplexMultiVector, DefaultMultiFieldBackend, DefaultMultiFieldFloat, DixonAlgebra,
    HilbertState, HopfState, PGA3DMultiVector, RealMultiVector,
};

// Errors
pub use crate::errors::causal_multivector_error::CausalMultiVectorError;

// Extensions
pub use crate::extensions::hkt_multifield::CausalMultiFieldWitness;
pub use crate::extensions::hkt_multivector::CausalMultiVectorWitness;

// Traits
pub use crate::traits::l2_norm::MultiVectorL2Norm;
pub use crate::traits::matrix_rep::MatrixRep;
pub use crate::traits::multi_vector::MultiVector;
pub use crate::traits::scalar_eval::ScalarEval;
pub use crate::types::multifield::ops::batched_matmul::BatchedMatMul;
// Types
pub use crate::types::multifield::CausalMultiField;
pub use crate::types::multifield::gamma::cpu::CpuGammaLoader;
pub use crate::types::multifield::gamma::*;
pub use crate::types::multifield::ops::differential::Axis;
pub use crate::types::multivector::CausalMultiVector;
pub use deep_causality_metric::Metric;

// Backend re-exports (convenience)
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub use deep_causality_tensor::MlxBackend;
pub use deep_causality_tensor::{CpuBackend, LinearAlgebraBackend, TensorBackend};

// === Type Aliases ===

/// Default backend for Multivector/MultiField operations.
/// - `mlx` feature: Uses `MlxBackend` for GPU acceleration (on macOS aarch64).
/// - Default: Uses `CpuBackend`.
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub type DefaultMultivectorBackend = deep_causality_tensor::MlxBackend;

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
pub type DefaultMultivectorBackend = CpuBackend;

/// Default floating-point precision.
/// - `mlx` feature: `f32` (GPU native precision).
/// - Default: `f64` (Full precision).
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub type DefaultMultivectorFloat = f32;

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
pub type DefaultMultivectorFloat = f64;

/// Alias for `CausalMultiVector` using the default float type.
/// Note: `CausalMultiVector` is always CPU-resident (coefficients).
pub type StandardMultiVector<T = DefaultMultivectorFloat> = CausalMultiVector<T>;

/// Alias for `CausalMultiField` using the default backend and float type.
/// This type changes automatically based on the `mlx` feature flag.
pub type MultiField<T = DefaultMultivectorFloat> = CausalMultiField<DefaultMultivectorBackend, T>;
