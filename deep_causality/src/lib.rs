/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! DeepCausality is a hyper-geometric computational causality library
//! that enables fast and deterministic context-aware causal reasoning over complex causality models.
//!
//! Why DeepCausality?
//! * DeepCausality is written in Rust with production-grade safety, reliability, and performance in mind.
//! * DeepCausality provides recursive causal data structures that concisely express arbitrary complex causal structures.
//! * DeepCausality enables context awareness across data-like, time-like, space-like, spacetime-like entities stored within (multiple) context-hyper-graphs.
//! * DeepCausality simplifies modeling of complex tempo-spatial patterns.
//! * DeepCausality comes with Causal State Machine (CSM)
//!
pub mod errors;
pub mod extensions;
pub(crate) mod macros;
pub mod prelude;
pub mod traits;
pub mod types;
pub mod utils;
