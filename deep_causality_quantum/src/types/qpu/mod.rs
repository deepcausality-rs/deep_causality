/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The reified circuit, and the emergent-modality QPU seam.
//!
//! [`circuit`] is always compiled: `GateOp` and `QuantumCircuit` are plain data
//! with no dependency of their own, and the Haruna gate layer in `qgates` emits
//! them, so gating them would gate the logical gates too.
//!
//! Everything else here is behind feature `qpu`, off by default. A physical
//! cloud-QPU call as a monadic effect, kept strictly apart from the verifiable
//! path by the feature gate: the default build compiles none of it and pulls in
//! no network/async dependency. This crate ships only the seam and an
//! in-process deterministic simulator — no concrete vendor adapter.

pub(crate) mod circuit;
pub use circuit::*;

#[cfg(feature = "qpu")]
pub(crate) mod bridge;
#[cfg(feature = "qpu")]
pub(crate) mod sampler;
#[cfg(feature = "qpu")]
pub(crate) mod sim;

#[cfg(feature = "qpu")]
pub use bridge::*;
#[cfg(feature = "qpu")]
pub use sampler::*;
#[cfg(feature = "qpu")]
pub use sim::*;
