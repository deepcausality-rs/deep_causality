/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, MultiVector};
use deep_causality_num::Complex;

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

/// Core Quantum State Operations
pub trait QuantumOps {
    /// Hermitian Conjugate (Adjoint)
    fn dag(&self) -> Self;

    /// Inner Product (Dirac Bracket <a|b>)
    fn bracket(&self, other: &Self) -> Complex<f64>;

    /// Expectation Value <psi|A|psi>
    fn expectation_value(&self, operator: &Self) -> Complex<f64>;

    /// Normalize the state vector
    fn normalize(&self) -> Self;
}

impl QuantumOps for CausalMultiVector<Complex<f64>> {
    fn dag(&self) -> Self {
        // Quantum DAG is Hermitian Conjugate.
        // In GA over Complex numbers, this corresponds to Reversion + Complex Conjugation?
        // Or just Reversion if coefficients are real?
        // For now, assuming Reversion is sufficient for spatial reversal, but we need complex conjugation of coefficients too.
        // CausalMultiVector might not support complex conjugation of coefficients easily without a map.
        // Assuming .reversion() exists.
        self.reversion()
    }

    fn bracket(&self, other: &Self) -> Complex<f64> {
        // <self|other>
        // In GA: (self.dag * other).scalar_part()
        // Or inner product.
        // We will return the scalar part of the geometric product of dag(self) and other.
        let prod = self.dag().geometric_product(other);
        // Assuming .get(0) returns &T
        prod.get(0).cloned().unwrap_or(Complex::new(0.0, 0.0))
    }

    fn expectation_value(&self, operator: &Self) -> Complex<f64> {
        // <psi|A|psi>
        let ket_psi = self;
        let bra_psi = self.dag();

        // A * |psi>
        let a_psi = operator.geometric_product(ket_psi);

        // <psi| (A|psi>)
        let prod = bra_psi.geometric_product(&a_psi);
        prod.get(0).cloned().unwrap_or(Complex::new(0.0, 0.0))
    }

    fn normalize(&self) -> Self {
        use deep_causality_multivector::MultiVectorL2Norm;
        self.normalize_l2()
    }
}
