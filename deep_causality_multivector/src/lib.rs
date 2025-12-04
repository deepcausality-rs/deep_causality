/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub mod alias;
pub mod errors;
pub mod extensions;
pub mod traits;
pub mod types;

pub use crate::alias::{
    ComplexMultiVector, DixonAlgebra, HilbertState, HopfState, PGA3DMultiVector, RealMultiVector,
};
pub use crate::errors::causal_multivector_error::CausalMultiVectorError;
pub use crate::extensions::hkt::CausalMultiVectorWitness;
pub use crate::extensions::quantum::{QuantumGates, QuantumOps};
pub use crate::traits::multi_vector::MultiVector;
pub use crate::types::metric::Metric;
pub use crate::types::multivector::CausalMultiVector;
