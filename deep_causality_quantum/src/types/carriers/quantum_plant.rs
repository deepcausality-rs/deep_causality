/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::carriers::quantum_channel::Channel;
use crate::types::decision::Tolerance;
use crate::types::density_matrix::DensityMatrix;
use deep_causality_algebra::RealField;
use deep_causality_haft::Functor;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::{Complex, ComplexWitness};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

/// A sealed, validated state that evolves by operation.
///
/// The interior is a [`DensityMatrix`], which has passed the Hermiticity, positivity and
/// unit-trace checks of `DensityMatrix::with_tolerance`. There is no accessor yielding `&mut` to
/// it. [`evolve`](Self::evolve) applies a [`Channel`] through the channel's own validated family
/// and validates the result once as a new density matrix, so every state a caller can observe has
/// passed the checks, and the receiver is unchanged.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumPlant<R: RealField> {
    state: DensityMatrix<R>,
}

impl<R> QuantumPlant<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// A plant in a validated state.
    pub fn new(state: DensityMatrix<R>) -> Self {
        Self { state }
    }

    /// A plant in the pure state of a ket, through the shipped `DensityMatrix::from_ket`.
    ///
    /// # Errors
    ///
    /// As `DensityMatrix::from_ket`.
    pub fn from_ket(ket: &CausalTensor<Complex<R>>) -> Result<Self, QuantumError> {
        Ok(Self::new(DensityMatrix::from_ket(ket)?))
    }

    /// The plant after `channel`, as a new value.
    ///
    /// The evolved state is validated against the state member of the [`Tolerance`] family, which
    /// `DensityMatrix::with_tolerance` scales by the operator's Frobenius norm, so the check
    /// tightens with the scalar and no literal appears here.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] from the shipped shape check when the channel's input
    /// dimension differs from the plant's, before any state is built; and the density-matrix
    /// validation errors if a channel that passed its own checks nonetheless produced an invalid
    /// state, which rounding on a wide operator can do.
    pub fn evolve(&self, channel: &Channel<R>) -> Result<Self, QuantumError> {
        let evolved = channel.apply(self.state.matrix())?;
        let tol = Tolerance::<R>::state()
            .threshold(channel.d_out(), R::one())
            .expect("the state member answers the single-operator form");
        Ok(Self::new(DensityMatrix::with_tolerance(evolved, tol)?))
    }

    /// The same state at another scalar, re-validated there.
    ///
    /// Two functors composed, the outer over cells and the inner over `re` and `im`; see
    /// [`QubitOperator::lift`](crate::QubitOperator::lift).
    ///
    /// # Errors
    ///
    /// As `DensityMatrix::new` at the target scalar.
    pub fn lift<S, F>(&self, mut f: F) -> Result<QuantumPlant<S>, QuantumError>
    where
        S: RealField + FromPrimitive + Default + core::fmt::Debug,
        F: FnMut(R) -> S,
    {
        let lifted = CausalTensorWitness::fmap(self.state.matrix().clone(), |z| {
            ComplexWitness::fmap(z, &mut f)
        });
        Ok(QuantumPlant::new(DensityMatrix::new(lifted)?))
    }
}

impl<R: RealField> QuantumPlant<R> {
    /// The validated state, read-only.
    pub fn state(&self) -> &DensityMatrix<R> {
        &self.state
    }

    /// The state's matrix, read-only.
    pub fn matrix(&self) -> &CausalTensor<Complex<R>> {
        self.state.matrix()
    }

    /// The Hilbert dimension.
    pub fn dim(&self) -> usize {
        self.state.dim()
    }
}

/// A qubit in `|0⟩`, because the causal monad requires a `Default` for every value it carries.
impl<R> Default for QuantumPlant<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    fn default() -> Self {
        let ket = CausalTensor::from_slice(
            &[
                Complex::new(R::one(), R::zero()),
                Complex::new(R::zero(), R::zero()),
            ],
            &[2],
        );
        Self::from_ket(&ket).expect("|0⟩ is a state")
    }
}
