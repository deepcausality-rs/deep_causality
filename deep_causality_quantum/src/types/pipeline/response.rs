/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::carriers::Channel;
use crate::types::qcm::hypothesis::Hypothesis;
use crate::types::qcm::process_factors::CjFactor;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// What a configuration does in one candidate's world.
#[derive(Debug, Clone, PartialEq)]
pub enum Response<R: RealField> {
    /// For a mechanism candidate: the channel the configuration applies to the plant after the
    /// candidate's own.
    Channel(Channel<R>),
    /// For a structural candidate: the mechanism-level interventions `do(node ← factor)` the
    /// configuration makes, in order, and the instrument the intervened model is evaluated
    /// against, an operator on the candidate's joint space.
    Intervention {
        /// The interventions, each a node and its replacement factor.
        factors: Vec<(usize, CjFactor<R>)>,
        /// The instrument on the joint space.
        instrument: CausalTensor<Complex<R>>,
    },
}

/// The physics of an experiment: what configuration `C` does in each candidate's world. The
/// pipeline is generic over it, so a model is dispatched statically.
pub trait ResponseModel<R: RealField, C> {
    /// The response of `candidate`'s world to `configuration`.
    ///
    /// # Errors
    ///
    /// Whatever the model refuses, as a structured `QuantumError`.
    fn respond(
        &self,
        candidate: &Hypothesis<R>,
        configuration: &C,
    ) -> Result<Response<R>, QuantumError>;
}
