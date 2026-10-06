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

/// One noise world: the depolarising probability on every physical qubit, as an exact fraction, a
/// numerator and a denominator held as integers.
///
/// The noise acts on qubits, not at a position or an instant, so the spatial, temporal and
/// spacetime slots are the absent ones.
pub type NoiseContext = Context<Data<i64>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Contextoid id: the depolarising probability's numerator, dimensionless.
const NOISE_NUMERATOR: ContextoidId = 1;
/// Contextoid id: the depolarising probability's denominator, dimensionless.
const NOISE_DENOMINATOR: ContextoidId = 2;

/// The swept noise worlds, one per depolarising probability: `0`, `1/100` and `5/100`. Zero is the
/// noiseless round.
///
/// Each probability is held as a numerator and a denominator and divided at the precision in force,
/// rather than as an `f64` literal that is widened afterwards. `lift::<Float106>(0.01_f64)` is the
/// `f64` approximation of a hundredth carried into a wider type, which is a different number from
/// the hundredth that type can represent. Dividing the integers keeps every precision's probability
/// its own nearest value, which is what makes a comparison across precisions mean anything.
pub fn noise_worlds() -> Result<Vec<NoiseContext>, ContextIndexError> {
    [0, 1, 5]
        .into_iter()
        .map(|numerator| noise_world(numerator, 100))
        .collect()
}

/// One noise world, depolarising with probability `numerator / denominator`.
fn noise_world(numerator: i64, denominator: i64) -> Result<NoiseContext, ContextIndexError> {
    let facts = [
        (NOISE_NUMERATOR, numerator),
        (NOISE_DENOMINATOR, denominator),
    ];
    let mut world = Context::with_capacity(1, "depolarising noise", facts.len());
    for (id, value) in facts {
        world.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(world)
}

/// The depolarising probability as the world holds it: `(numerator, denominator)`.
pub fn noise_fraction(world: &NoiseContext) -> Result<(i64, i64), ContextIndexError> {
    Ok((
        read(world, NOISE_NUMERATOR)?,
        read(world, NOISE_DENOMINATOR)?,
    ))
}

/// The depolarising probability at the precision `S`, divided from the world's two integers.
pub fn depolarising_probability<S>(world: &NoiseContext) -> Result<S, ContextIndexError>
where
    S: RealField + FromPrimitive,
{
    let (numerator, denominator) = noise_fraction(world)?;
    Ok(lift_i64::<S>(numerator) / lift_i64::<S>(denominator))
}

/// Read the integer with contextoid id `id` out of a noise world.
fn read(world: &NoiseContext, id: ContextoidId) -> Result<i64, ContextIndexError> {
    world
        .get_data_by_id(id)
        .ok_or_else(|| ContextIndexError::new(format!("no noise datum with contextoid id {id}")))
}
