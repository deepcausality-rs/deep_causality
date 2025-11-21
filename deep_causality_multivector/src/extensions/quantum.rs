/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Complex;

/// Operations specific to Quantum Mechanics / Hilbert Spaces.
/// These correspond to Bra-Ket notation operations.
pub trait QuantumOps {
    /// The Hermitian Conjugate (The "Bra" <psi|)
    /// Corresponds to Reversion (Geometry) + Complex Conjugation (Coefficients).
    fn dag(&self) -> Self;

    /// The Inner Product <self | other>
    /// Returns the Probability Amplitude (Scalar).
    fn bracket(&self, other: &Self) -> Complex<f64>;

    /// The Expectation Value <self | Operator | self>
    /// Returns the observable value (Scalar).
    fn expectation_value(&self, operator: &Self) -> Complex<f64>;

    /// Normalizes the state so <psi|psi> = 1
    fn normalize(&self) -> Self;
}
