/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The noise world of the switching gadget, as the context the chain is built from.

use crate::FloatType;
use deep_causality_algebra::RealField;
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_num::{FromPrimitive, lift_i64};

/// The world the gadget runs in: its depolarising probability as an exact fraction, a numerator
/// and a denominator held as integers.
///
/// The noise acts on a logical wire, not at a position or an instant, so the spatial, temporal and
/// spacetime slots are the absent ones.
pub type NoiseContext = Context<Data<i64>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Node indices of the probability's numerator and denominator.
const NOISE_NUMERATOR: usize = 0;
const NOISE_DENOMINATOR: usize = 1;

/// The gadget's noise world: depolarising probability one tenth, applied to the first logical wire
/// between the decoder of code A and the encoder of code B.
///
/// The probability is held as a numerator and a denominator and divided at the precision in force,
/// rather than as an `f64` literal that is widened afterwards. `lift::<Float106>(0.1_f64)` is the
/// `f64` approximation of a tenth carried into a wider type, which is a different number from the
/// tenth that type can represent, so the cross-precision rows would be comparing arithmetic on
/// three slightly different probabilities.
pub fn gadget_noise_world() -> Result<NoiseContext, ContextIndexError> {
    let mut world = Context::with_capacity(1, "gadget noise", 2);
    for (id, value) in [(1, 1), (2, 10)] {
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

/// Read one integer out of the noise world.
fn read(world: &NoiseContext, index: usize) -> Result<i64, ContextIndexError> {
    world
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(Datable::get_data)
        .ok_or_else(|| ContextIndexError::new(format!("no noise datum at node {index}")))
}
