/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Quantum Electrodynamics (QED) Theory Module
//!
//! Provides a unified interface for electromagnetic field operations in the U(1) gauge theory
//! taking advantage of the topological `GaugeField` structure.

mod qed_ops;
mod qed_ops_impl;

pub use qed_ops::*;
