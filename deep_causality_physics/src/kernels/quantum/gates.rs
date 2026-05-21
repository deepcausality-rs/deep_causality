/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, MultiVector};
use deep_causality_num::{Complex, RealField};

/// Standard Quantum Gates interface.
pub trait QuantumGates {
    fn gate_identity() -> Self;
    fn gate_x() -> Self;
    fn gate_y() -> Self;
    fn gate_z() -> Self;
    fn gate_hadamard() -> Self;
    /// Controlled-NOT gate.
    fn gate_cnot() -> Self;
}

/// Core Quantum State Operations (Dirac Notation), parameterized over the
/// underlying real field $R$ so the operations can be carried out at f32,
/// f64, f128, or any other real-field precision.
pub trait QuantumOps<R: RealField> {
    /// Hermitian Conjugate (Adjoint) $A^\dagger$.
    fn dag(&self) -> Self;

    /// Inner Product (Dirac Bracket): $\langle \phi | \psi \rangle$.
    fn bracket(&self, other: &Self) -> Complex<R>;

    /// Expectation Value: $\langle \psi | A | \psi \rangle$.
    fn expectation_value(&self, operator: &Self) -> Complex<R>;

    /// Normalize the state vector: $|\psi\rangle / \sqrt{\langle\psi|\psi\rangle}$.
    fn normalize(&self) -> Self;
}

impl<R: RealField + core::iter::Sum> QuantumOps<R> for CausalMultiVector<Complex<R>> {
    fn dag(&self) -> Self {
        // Hermitian conjugate: reverse basis (reversion) and conjugate coefficients
        let reverted = self.reversion();
        let conjugated_data = reverted
            .data()
            .iter()
            .map(|c| Complex::new(c.re, -c.im))
            .collect::<Vec<_>>();
        CausalMultiVector::new(conjugated_data, reverted.metric()).unwrap_or_else(|_| {
            CausalMultiVector::new(
                vec![Complex::new(R::zero(), R::zero()); reverted.data().len()],
                reverted.metric(),
            )
            .expect("consistent metric")
        })
    }

    fn bracket(&self, other: &Self) -> Complex<R> {
        let prod = self.dag().geometric_product(other);
        prod.get(0)
            .cloned()
            .unwrap_or(Complex::new(R::zero(), R::zero()))
    }

    fn expectation_value(&self, operator: &Self) -> Complex<R> {
        let bra_psi = self.dag();
        let a_psi = operator.geometric_product(self);
        let prod = bra_psi.geometric_product(&a_psi);
        prod.get(0)
            .cloned()
            .unwrap_or(Complex::new(R::zero(), R::zero()))
    }

    fn normalize(&self) -> Self {
        use deep_causality_multivector::MultiVectorL2Norm;
        self.normalize_l2()
    }
}
