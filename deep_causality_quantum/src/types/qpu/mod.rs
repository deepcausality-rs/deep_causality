/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The emergent-modality QPU seam (feature `qpu`, off by default). A physical
//! cloud-QPU call as a monadic effect, kept strictly apart from the verifiable
//! path by the feature gate: the default build compiles none of this and pulls
//! in no network/async dependency. This crate ships only the seam and an
//! in-process deterministic simulator — no concrete vendor adapter.

pub(crate) mod bridge;
pub(crate) mod circuit;
pub(crate) mod sampler;
pub(crate) mod sim;

pub use bridge::*;
pub use circuit::*;
pub use sampler::*;
pub use sim::*;
