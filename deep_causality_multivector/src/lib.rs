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
    ComplexMultiVector, DixonAlgebra, HilbertState, HopfState, PGA3DMultiVector, RealMultiVector,
};

// Errors
pub use crate::errors::causal_multivector_error::CausalMultiVectorError;

// Extensions
pub use crate::extensions::hkt::CausalMultiVectorWitness;

// Traits
pub use crate::traits::l2_norm::MultiVectorL2Norm;
pub use crate::traits::multi_vector::MultiVector;
pub use crate::traits::scalar_eval::ScalarEval;

// Types
pub use crate::types::multivector::CausalMultiVector;
pub use deep_causality_metric::Metric;
