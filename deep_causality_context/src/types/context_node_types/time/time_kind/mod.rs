/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod display;
mod identifiable;
mod recordable;
mod scalar_projector;
mod temporable;

use crate::{DiscreteTime, EntropicTime, MinkowskiTime, NewtonianTime};
use deep_causality_algebra::RealField;

/// One of the time types a context can hold, so that a single time slot can carry nodes of
/// different kinds.
///
/// # Variants
///
/// - `Newtonian(NewtonianTime<R>)`: the absolute time of a classical spacetime. Durations are the
///   same for every observer.
/// - `Minkowski(MinkowskiTime<R>)`: the coordinate time of an inertial frame of Minkowski
///   spacetime. It depends on the frame.
/// - `Discrete(DiscreteTime)`: a tick count; it implies a duration only through its `TimeScale`.
/// - `Entropic(EntropicTime)`: a monotone count of irreversible state changes, with no physical
///   unit.
///
/// The variants measure different things, so comparing the scalars of two different variants
/// compares numbers, not instants.
///
/// # Example
///
/// ```rust
/// use deep_causality_context::*;
///
/// let coordinate = TimeKind::Minkowski(MinkowskiTime::new(1, TimeScale::Second, 3.14));
///
/// // The tick-based variants carry no scalar of their own, so the enum's scalar is named here.
/// let discrete: TimeKind<FloatType> =
///     TimeKind::Discrete(DiscreteTime::new(2, TimeScale::Second, 42));
///
/// assert_eq!(coordinate.id(), 1);
/// assert_eq!(discrete.id(), 2);
/// ```
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TimeKind<R>
where
    R: RealField,
{
    /// The absolute time of a classical spacetime.
    Newtonian(NewtonianTime<R>),

    /// Entropic time for emergent causal models via entropy.
    Entropic(EntropicTime),

    /// Discrete tick-based time (steps, iterations, simulation frames).
    Discrete(DiscreteTime),

    /// The coordinate time of an inertial frame of Minkowski spacetime.
    Minkowski(MinkowskiTime<R>),
    // /// Symbolic or qualitative time labels (e.g., "before event A", "T1").
    // Symbolic(SymbolicTime),
}
