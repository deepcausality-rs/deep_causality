/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, MultiVector};
use deep_causality_num::Complex;

/// Standard Quantum Gates interface.
pub trait QuantumGates {
    fn gate_identity() -> Self;
    fn gate_x() -> Self;
    fn gate_y() -> Self;
    fn gate_z() -> Self;
    fn gate_hadamard() -> Self;
    /// Controlled-NOT gate.
    /// Note: Implementation for single MultiVector implies 2-qubit representation support
    /// or specific tensor structure.
    fn gate_cnot() -> Self;
}

/// Core Quantum State Operations (Dirac Notation).
pub trait QuantumOps {
    /// Hermitian Conjugate (Adjoint) $A^\dagger$.
    fn dag(&self) -> Self;

    /// Inner Product (Dirac Bracket): $\langle \phi | \psi \rangle$.
    fn bracket(&self, other: &Self) -> Complex<f64>;

    /// Expectation Value: $\langle \psi | A | \psi \rangle$.
    fn expectation_value(&self, operator: &Self) -> Complex<f64>;

    /// Normalize the state vector: $|\psi\rangle / \sqrt{\langle\psi|\psi\rangle}$.
    fn normalize(&self) -> Self;
}

impl QuantumOps for CausalMultiVector<Complex<f64>> {
    fn dag(&self) -> Self {
        // Hermitian conjugate: reverse basis (reversion) and conjugate coefficients
        let reverted = self.reversion();
        let conjugated_data = reverted
            .data()
            .iter()
            .map(|c| Complex::new(c.re, -c.im))
            .collect::<Vec<_>>();
        // Construct with the same metric; do not silently drop conjugation
        // If this ever fails, return a zero-initialized vector with the same metric to avoid wrong math.
        CausalMultiVector::new(conjugated_data, reverted.metric()).unwrap_or_else(|_| {
            CausalMultiVector::new(
                vec![Complex::new(0.0, 0.0); reverted.data().len()],
                reverted.metric(),
            )
            .expect("consistent metric")
        })
    }

    fn bracket(&self, other: &Self) -> Complex<f64> {
        let prod = self.dag().geometric_product(other);
        prod.get(0).cloned().unwrap_or(Complex::new(0.0, 0.0))
    }

    fn expectation_value(&self, operator: &Self) -> Complex<f64> {
        let bra_psi = self.dag();
        let a_psi = operator.geometric_product(self);
        let prod = bra_psi.geometric_product(&a_psi);
        prod.get(0).cloned().unwrap_or(Complex::new(0.0, 0.0))
    }

    fn normalize(&self) -> Self {
        use deep_causality_multivector::MultiVectorL2Norm;
        self.normalize_l2()
    }
}
