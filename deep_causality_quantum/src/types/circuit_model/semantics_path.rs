/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// Which semantics decided a value derived from a [`CircuitModel`](crate::CircuitModel).
///
/// The exact path carries Pauli and Clifford programs as their symplectic action and diagonal
/// Table 1 gates as [`GaugeFieldGate`](crate::GaugeFieldGate) phase tables; it has no
/// register-width limit and its residuals are zero or not. The numeric path carries a program at
/// the Kraus level and compares Choi operators of the composite channel in Frobenius norm under a
/// cap. Every report names the path, so a verdict reached numerically is never read as exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticsPath {
    /// Decided over 𝔽₂ and exact rationals.
    Exact,
    /// Decided on Choi operators in Frobenius norm against a tolerance.
    Numeric,
}

impl SemanticsPath {
    /// The path's name, for reports.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Numeric => "numeric",
        }
    }
}
