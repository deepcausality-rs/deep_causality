// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.


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
pub mod extensions;
pub mod protocols;
pub mod prelude;
pub mod types;
pub mod utils;
pub mod errors;

