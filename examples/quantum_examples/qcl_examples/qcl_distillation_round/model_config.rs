/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The noise worlds the distillation round is swept over, as the contexts the chain is built from.

use crate::FloatType;
use deep_causality_algebra::RealField;
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_num::{FromPrimitive, lift_i64};
use deep_causality_num_rational::Rational;

/// One noise world: the depolarising probability on every physical qubit, as an exact fraction.
///
/// The noise acts on qubits, not at a position or an instant, so the spatial, temporal and
/// spacetime slots are the absent ones.
pub type NoiseContext =
    Context<Data<Rational<i64>>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Contextoid id: the depolarising probability, an exact fraction, dimensionless.
const NOISE_PROBABILITY: ContextoidId = 1;

/// The swept noise worlds, one per depolarising probability: `0`, `1/100` and `5/100`, held in
/// lowest terms, so the last is `1/20`. Zero is the noiseless round.
///
/// Each probability is held as an exact fraction whose numerator and denominator are divided at the
/// precision in force, rather than as an `f64` literal that is widened afterwards.
/// `lift::<Float106>(0.01_f64)` is the `f64` approximation of a hundredth carried into a wider
/// type, which is a different number from the hundredth that type can represent. Dividing the
/// integers keeps every precision's probability its own nearest value, which is what makes a
/// comparison across precisions mean anything.
pub fn noise_worlds() -> Result<Vec<NoiseContext>, ContextIndexError> {
    [0, 1, 5]
        .into_iter()
        .map(|numerator| noise_world(Rational::new(numerator, 100)))
        .collect()
}

/// One noise world, depolarising with probability `probability`.
fn noise_world(probability: Rational<i64>) -> Result<NoiseContext, ContextIndexError> {
    let facts = [(NOISE_PROBABILITY, probability)];
    let mut world = Context::with_capacity(1, "depolarising noise", facts.len());
    for (id, value) in facts {
        world.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(world)
}

/// The depolarising probability as the world holds it, an exact fraction.
pub fn noise_probability(world: &NoiseContext) -> Result<Rational<i64>, ContextIndexError> {
    world.get_data_by_id(NOISE_PROBABILITY).ok_or_else(|| {
        ContextIndexError::new(format!(
            "no noise datum with contextoid id {NOISE_PROBABILITY}"
        ))
    })
}

/// The depolarising probability at the precision `S`, its numerator divided by its denominator.
pub fn depolarising_probability<S>(world: &NoiseContext) -> Result<S, ContextIndexError>
where
    S: RealField + FromPrimitive,
{
    let probability = noise_probability(world)?;
    Ok(lift_i64::<S>(*probability.numer()) / lift_i64::<S>(*probability.denom()))
}
