/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::carriers::quantum_plant::QuantumPlant;
use crate::types::qpu::born_sampler::sample_projector;
use crate::types::qpu::histogram::CountHistogram;
use crate::types::verdict::born::{born_projective_prob, born_projective_probability};
use crate::types::verdict::projection::Projection;
use alloc::string::String;
use deep_causality_algebra::{Prob, RealField};
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// A named projector carrying its own read-out.
///
/// This is the measurement boundary of the verdict law: the only site at which a verdict enters a
/// QCL pipeline. Every stage upstream carries operators; here a state meets a projection and a
/// number comes out, through the shipped `born_projective_probability`, or a `Prob` through
/// `born_projective_prob`. The observable folds no verdicts itself, because the fold rule depends
/// on which kind of verdict a world carries; it exposes its [`Projection`] so that `adjudicate`
/// can ask `commutes_with` of a pair before folding projection-valued verdicts.
#[derive(Debug, Clone)]
pub struct Observable<R: RealField, const D: usize> {
    name: String,
    projection: Projection<R, D>,
}

impl<R, const D: usize> Observable<R, D>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// A named observable over a validated projection.
    pub fn new(name: impl Into<String>, projection: Projection<R, D>) -> Self {
        Self {
            name: name.into(),
            projection,
        }
    }

    /// The rank-1 observable `|ψ⟩⟨ψ|` of a ket, validated by the shipped `Projection::from_ket`.
    ///
    /// # Errors
    ///
    /// As `Projection::from_ket`: a ket of the wrong shape or dimension, or a near-zero ket.
    pub fn from_ket(
        name: impl Into<String>,
        ket: &CausalTensor<Complex<R>>,
    ) -> Result<Self, QuantumError> {
        Ok(Self::new(name, Projection::from_ket(ket)?))
    }

    /// The Born probability `Tr(Pρ)` on the plant's state, in `[0, 1]`.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] naming both dimensions when the plant's `dim()` differs
    /// from `D`, which is the error `born_projective_probability` already raises.
    pub fn read_out(&self, plant: &QuantumPlant<R>) -> Result<R, QuantumError> {
        born_projective_probability(plant.state(), &self.projection)
    }

    /// The read-out as the `Prob` verdict, the boundary where the number becomes a classical
    /// proposition.
    ///
    /// # Errors
    ///
    /// As [`read_out`](Self::read_out).
    pub fn read_out_prob(&self, plant: &QuantumPlant<R>) -> Result<Prob, QuantumError>
    where
        R: Into<f64>,
    {
        born_projective_prob(plant.state(), &self.projection)
    }

    /// `shots` samples of the accepting outcome at `seed`, drawn from the Born probability this
    /// observable reads out. Outcome `1` is "accepted", `0` is "rejected".
    ///
    /// # Errors
    ///
    /// As [`read_out`](Self::read_out); nothing is drawn on the error path.
    pub fn sample(
        &self,
        plant: &QuantumPlant<R>,
        shots: u64,
        seed: u64,
    ) -> Result<CountHistogram, QuantumError> {
        sample_projector(plant.state(), &self.projection, shots, seed)
    }
}

impl<R: RealField, const D: usize> Observable<R, D> {
    /// The name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The projection, for `adjudicate`'s commutation guard.
    pub fn projection(&self) -> &Projection<R, D> {
        &self.projection
    }

    /// Always `D`.
    pub fn dim(&self) -> usize {
        D
    }
}

/// The zero projection under an empty name, because the causal monad requires a `Default` for
/// every value it carries.
impl<R, const D: usize> Default for Observable<R, D>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    fn default() -> Self {
        Self::new("", Projection::zero())
    }
}
