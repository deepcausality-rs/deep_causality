/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Quantum causal models on the causal monad.
//!
//! This crate carries the quantum-information layer of DeepCausality: the
//! [`QuantumGates`]/[`QuantumOps`] traits and gate kernels migrated out of
//! `deep_causality_physics`, with `HilbertState` (the pure-state ket) staying
//! in `deep_causality_multivector` as the foundational carrier.
//!
//! Two quantum senses share this crate but are kept strictly apart by
//! modality: the **verifiable** path (deterministic simulated
//! Choi–Jamiołkowski operators, checked at the freeze boundary) is the
//! default build and the target of the Lean proofs; the **emergent** path (a
//! physical QPU call as a monadic effect) is a seam only.
//!
//! All metric signatures come from `deep_causality_metric`, the single source
//! of truth; this crate defines no metric type of its own.

pub(crate) mod error;
pub(crate) mod types;

pub use crate::error::quantum_error::{QuantumError, QuantumErrorEnum};

pub use crate::types::qcm::*;
pub use crate::types::qgates::*;
pub use crate::types::verdict::*;
pub use crate::types::*;

#[cfg(feature = "qpu")]
pub use crate::types::qpu::*;