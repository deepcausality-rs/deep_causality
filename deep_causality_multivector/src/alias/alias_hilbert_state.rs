/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalMultiVector, CausalMultiVectorError, Metric};
use deep_causality_algebra::RealField;
use deep_causality_num::Zero;
use deep_causality_num_complex::Complex;
use std::fmt::{Display, Formatter};

/// A strong type representing a Quantum State Vector (Ket) |ψ>.
///
/// This represents a Minimal Left Ideal of the algebra Cl(10) (or others), acting as the Hilbert Space.
///
/// # Invariants
/// * The coefficients are `Complex<R>` for any `R: RealField` chosen by the caller
///   (`f32`, `f64`, `Float106`, …). The type parameter has no default — call sites
///   must spell the precision explicitly (e.g. `HilbertState::<f64>::new(...)`).
/// * The Metric is fixed at construction time (preventing mixed-algebra operations).
#[derive(Debug, Clone, PartialEq)]
pub struct HilbertState<R: RealField> {
    mv: CausalMultiVector<Complex<R>>,
}

impl<R: RealField> HilbertState<R> {
    /// Creates a new Hilbert State for the Grand Unified Algebra (Spin(10)).
    /// This enforces the metric Cl(10) (NonEuclidean, 10D).
    pub fn new_spin10(data: Vec<Complex<R>>) -> Result<Self, CausalMultiVectorError> {
        let metric = Metric::NonEuclidean(10);
        let mv = CausalMultiVector::new(data, metric)?;
        Ok(Self { mv })
    }

    /// Generic constructor for other quantum systems (e.g. Qubits / Cl(2)).
    pub fn new(data: Vec<Complex<R>>, metric: Metric) -> Result<Self, CausalMultiVectorError> {
        let mv = CausalMultiVector::new(data, metric)?;
        Ok(Self { mv })
    }

    pub fn new_unchecked(data: Vec<Complex<R>>, metric: Metric) -> Self {
        let mv = CausalMultiVector::unchecked(data, metric);
        Self { mv }
    }

    pub fn from_multivector(mv: CausalMultiVector<Complex<R>>) -> Self {
        Self { mv }
    }

    /// Unwraps the state to access the underlying algebraic object.
    /// Useful when you need to perform raw geometric operations.
    pub fn into_inner(self) -> CausalMultiVector<Complex<R>> {
        self.mv
    }

    /// Borrows the underlying algebraic object.
    pub fn as_inner(&self) -> &CausalMultiVector<Complex<R>> {
        &self.mv
    }

    pub fn mv(&self) -> &CausalMultiVector<Complex<R>> {
        &self.mv
    }
}

impl<R: RealField> Default for HilbertState<R> {
    fn default() -> Self {
        // Default to Scalar 0 in Euclidean(0) or similar
        let metric = Metric::Euclidean(0);
        let data = vec![Complex::zero()];
        let mv = CausalMultiVector::new(data.clone(), metric)
            .unwrap_or(CausalMultiVector::unchecked(data, metric));
        Self { mv }
    }
}

// Allow adding two Quantum States: |psi> + |phi> (Superposition)
impl<R: RealField> core::ops::Add for HilbertState<R> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            mv: self.mv + rhs.mv,
        }
    }
}

// Allow scaling a State: c * |psi>
impl<R: RealField> core::ops::Mul<Complex<R>> for HilbertState<R> {
    type Output = Self;
    fn mul(self, rhs: Complex<R>) -> Self::Output {
        Self { mv: self.mv * rhs }
    }
}

impl<R: RealField + core::fmt::Debug> Display for HilbertState<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.mv)
    }
}
