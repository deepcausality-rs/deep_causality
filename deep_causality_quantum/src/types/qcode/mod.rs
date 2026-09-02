/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Logical operators of a CSS code, and when two of them act alike.
//!
//! Junichi Haruna, *Note on Logical Gates by Gauge Field Formalism of Quantum Error Correction*,
//! arXiv:2511.15224, Theorem A.1 and Appendix B.1 to B.3.
//!
//! A CSS code is a chain complex, its logical `Z` operators are the classes of `H₁` and its logical
//! `X` operators the classes of `H¹`. Deciding whether two operators act the same way on the code
//! space is then a question about bitsets rather than about `2ⁿ × 2ⁿ` matrices, which is what makes
//! this module small.

pub(crate) mod clifford_action;
pub(crate) mod diagonal_phase;
pub(crate) mod logical_equivalence;
pub(crate) mod logical_pauli;

pub use clifford_action::*;
pub use diagonal_phase::*;
pub use logical_equivalence::*;
pub use logical_pauli::*;
