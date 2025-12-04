/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalMultiVector, CausalMultiVectorError, Metric};
use deep_causality_num::Complex;
use std::fmt::{Display, Formatter};

/// A strong type representing a Quantum State Vector (Ket) |ψ>.
///
/// This represents a Minimal Left Ideal of the algebra Cl(10) (or others), acting as the Hilbert Space.
///
/// # Invariants
/// * The coefficients are always `Complex<f64>`.
/// * The Metric is fixed at construction time (preventing mixed-algebra operations).
#[derive(Debug, Clone, PartialEq)]
pub struct HilbertState {
    mv: CausalMultiVector<Complex<f64>>,
}

impl HilbertState {
    /// Creates a new Hilbert State for the Grand Unified Algebra (Spin(10)).
    /// This enforces the metric Cl(10) (NonEuclidean, 10D).
    pub fn new_spin10(data: Vec<Complex<f64>>) -> Result<Self, CausalMultiVectorError> {
        let metric = Metric::NonEuclidean(10);
        let mv = CausalMultiVector::new(data, metric)?;
        Ok(Self { mv })
    }

    /// Generic constructor for other quantum systems (e.g. Qubits / Cl(2)).
    pub fn new(data: Vec<Complex<f64>>, metric: Metric) -> Result<Self, CausalMultiVectorError> {
        let mv = CausalMultiVector::new(data, metric)?;
        Ok(Self { mv })
    }

    pub fn new_unchecked(data: Vec<Complex<f64>>, metric: Metric) -> Self {
        let mv = CausalMultiVector::unchecked(data, metric);
        Self { mv }
    }

    pub fn from_multivector(mv: CausalMultiVector<Complex<f64>>) -> Self {
        Self { mv }
    }

    /// Unwraps the state to access the underlying algebraic object.
    /// Useful when you need to perform raw geometric operations.
    pub fn into_inner(self) -> CausalMultiVector<Complex<f64>> {
        self.mv
    }

    /// Borrows the underlying algebraic object.
    pub fn as_inner(&self) -> &CausalMultiVector<Complex<f64>> {
        &self.mv
    }

    pub fn mv(&self) -> &CausalMultiVector<Complex<f64>> {
        &self.mv
    }
}

// Allow adding two Quantum States: |psi> + |phi> (Superposition)
impl core::ops::Add for HilbertState {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        // Delegate to underlying MultiVector addition
        Self {
            mv: self.mv + rhs.mv,
        }
    }
}

// Allow scaling a State: c * |psi>
impl core::ops::Mul<Complex<f64>> for HilbertState {
    type Output = Self;
    fn mul(self, rhs: Complex<f64>) -> Self::Output {
        Self { mv: self.mv * rhs }
    }
}

impl Display for HilbertState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.mv)
    }
}
