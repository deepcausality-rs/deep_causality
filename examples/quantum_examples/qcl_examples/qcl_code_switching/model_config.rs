/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The noise world of the switching gadget, as the context the chain is built from.

use crate::FloatType;
use deep_causality_algebra::RealField;
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_num::{FromPrimitive, lift_i64};
use deep_causality_num_rational::Rational;

/// The world the gadget runs in: its depolarising probability as an exact fraction.
///
/// The noise acts on a logical wire, not at a position or an instant, so the spatial, temporal and
/// spacetime slots are the absent ones.
pub type NoiseContext =
    Context<Data<Rational<i64>>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Contextoid id: the depolarising probability, an exact fraction, dimensionless.
const NOISE_PROBABILITY: ContextoidId = 1;

/// The gadget's noise world: depolarising probability one tenth, applied to the first logical wire
/// between the decoder of code A and the encoder of code B.
///
/// The probability is held as an exact fraction whose numerator and denominator are divided at the
/// precision in force, rather than as an `f64` literal that is widened afterwards.
/// `lift::<Float106>(0.1_f64)` is the `f64` approximation of a tenth carried into a wider type,
/// which is a different number from the tenth that type can represent, so the cross-precision rows
/// would be comparing arithmetic on three slightly different probabilities.
pub fn gadget_noise_world() -> Result<NoiseContext, ContextIndexError> {
    let facts = [(NOISE_PROBABILITY, Rational::new(1, 10))];
    let mut world = Context::with_capacity(1, "gadget noise", facts.len());
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
