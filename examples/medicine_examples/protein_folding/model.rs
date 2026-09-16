/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the protein-folding simulation: the conformational states, the Markov
//! operator, the memory kernels, and one step of the generalized master equation.
//!
//! The distribution over states is a `Vec<Probability<FloatType>>`, and `VecWitness` carries three
//! categorical operations over it: `fold` totals the probability mass, `fmap` rescales each entry,
//! and `sequence` turns the vector of fallible re-wraps into one fallible vector.

use crate::FloatType;
use deep_causality_algebra::Real;
use deep_causality_haft::{Foldable, Functor, ResultWitness, Traversable, VecWitness};
use deep_causality_num::{lift, lift_count};
use deep_causality_physics::{PhysicsError, Probability, generalized_master_equation};
use deep_causality_tensor::{CausalTensor, CausalTensorError};

/// The conformational states, from unfolded to native.
pub const STATE_LABELS: [&str; 4] = ["unfolded", "intermediate 1", "intermediate 2", "native"];

/// Conformational states the chain moves between.
pub const N_STATES: usize = STATE_LABELS.len();

/// How many past distributions the memory kernels reach back over. A protein's next conformation
/// depends on the path it took, which is what makes the dynamics non-Markovian.
pub const MEMORY_DEPTH: usize = 3;

/// The single-step transition matrix, column-major by source state: entry `[i, j]` is the
/// probability of arriving in state `i` from state `j`. Each column sums to one, and the native
/// state is absorbing, so folding is a one-way trip once the chain reaches it.
#[rustfmt::skip]
const MARKOV_TRANSITIONS: [f64; N_STATES * N_STATES] = [
    // from:  unfolded  inter-1  inter-2  native
    /* to unfolded */ 0.70,  0.10,  0.00,  0.00,
    /* to inter-1  */ 0.30,  0.70,  0.10,  0.00,
    /* to inter-2  */ 0.00,  0.20,  0.40,  0.00,
    /* to native   */ 0.00,  0.00,  0.50,  1.00,
];

/// How strongly the memory term nudges the chain forward, and how fast that nudge decays with
/// how far back the remembered distribution sits.
const MEMORY_STRENGTH: f64 = 0.02;
const MEMORY_DECAY: f64 = 0.5;

/// The Markov transition operator as a `[N_STATES, N_STATES]` tensor.
pub fn markov_operator() -> Result<CausalTensor<FloatType>, CausalTensorError> {
    let data: Vec<FloatType> = MARKOV_TRANSITIONS
        .iter()
        .map(|&p| lift::<FloatType>(p))
        .collect();
    CausalTensor::new(data, vec![N_STATES, N_STATES])
}

/// One memory kernel per remembered lag.
///
/// Each kernel biases the chain along the folding pathway, unfolded to intermediate 1 to
/// intermediate 2 to native, and its weight decays exponentially with the lag. A protein that
/// recently occupied an intermediate is more likely to move on from it than one that arrived by
/// chance, which is the memory effect the generalized master equation carries.
pub fn memory_kernels() -> Result<Vec<CausalTensor<FloatType>>, CausalTensorError> {
    (0..MEMORY_DEPTH)
        .map(|lag| {
            let age = lift_count::<FloatType>(lag as u64 + 1);
            let weight = Real::exp(-lift::<FloatType>(MEMORY_DECAY) * age)
                * lift::<FloatType>(MEMORY_STRENGTH);

            let mut data = vec![lift::<FloatType>(0.0); N_STATES * N_STATES];
            for step in 0..N_STATES - 1 {
                // Row `step + 1`, column `step`: arriving in the next state along the pathway.
                data[(step + 1) * N_STATES + step] = weight;
            }
            CausalTensor::new(data, vec![N_STATES, N_STATES])
        })
        .collect()
}

/// The initial distribution: the chain starts fully unfolded.
pub fn unfolded_state() -> Result<Vec<Probability<FloatType>>, PhysicsError> {
    let one = lift::<FloatType>(1.0);
    let zero = lift::<FloatType>(0.0);
    (0..N_STATES)
        .map(|i| Probability::new(if i == 0 { one } else { zero }))
        .collect()
}

/// Advances the distribution by one step of the generalized master equation, then renormalises.
///
/// The memory term adds probability mass that the Markov operator alone conserves, so the step
/// leaves the distribution slightly off unit total. Rescaling restores it.
pub fn advance(
    state: &[Probability<FloatType>],
    history: &[Vec<Probability<FloatType>>],
    operator: &CausalTensor<FloatType>,
    kernels: &[CausalTensor<FloatType>],
) -> Result<Vec<Probability<FloatType>>, PhysicsError> {
    let effect = generalized_master_equation(state, history, Some(operator), kernels);

    match effect.into_value() {
        Some(stepped) => normalise(stepped),
        None => Err(PhysicsError::NormalizationError(
            "the master equation returned no distribution".to_string(),
        )),
    }
}

/// Rescales a distribution so its entries sum to one.
///
/// Three categorical steps: `fold` totals the mass, `fmap` divides each entry by that total, and
/// `sequence` flips the resulting `Vec<Result<..>>` into a `Result<Vec<..>>`, so a value that
/// leaves `[0, 1]` surfaces as one error for the whole distribution.
pub fn normalise(
    state: Vec<Probability<FloatType>>,
) -> Result<Vec<Probability<FloatType>>, PhysicsError> {
    let zero = lift::<FloatType>(0.0);

    let total = VecWitness::fold(state.clone(), zero, |sum, p| sum + p.value());
    if total <= zero {
        return Err(PhysicsError::NormalizationError(format!(
            "total probability mass reached {:?}, which leaves nothing to rescale",
            total
        )));
    }

    let rescaled = VecWitness::fmap(state, |p| Probability::new(p.value() / total));
    VecWitness::sequence::<Probability<FloatType>, ResultWitness<PhysicsError>>(rescaled)
}

/// The probability the chain has reached its native state.
pub fn native_fraction(state: &[Probability<FloatType>]) -> FloatType {
    state[N_STATES - 1].value()
}
