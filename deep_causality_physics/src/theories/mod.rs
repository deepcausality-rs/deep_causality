/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Physics Theory Modules
//!
//! High-level theory interfaces combining low-level kernels into cohesive APIs.
//!
//! - **QED** (U(1)): Quantum Electrodynamics
//! - **Weak** (SU(2)): Weak nuclear force
//! - **Electroweak** (SU(2)×U(1)): Unified electroweak theory
pub mod alias;
pub mod electroweak;
pub mod gr;
pub mod qed;
pub mod weak_force;

pub use alias::*;
pub use electroweak::*;
pub use gr::*;
pub use qed::*;
pub use weak_force::*;
