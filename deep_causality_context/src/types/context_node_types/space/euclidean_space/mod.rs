/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
use core::fmt::Debug;
use deep_causality_algebra::RealField;

mod adjustable;
mod coordinate;
mod display;
mod getters;
mod identifiable;
mod metric;
mod recordable;
mod spatial;

/// A point of three-dimensional Euclidean space, in Cartesian coordinates `x, y, z`.
///
/// Euclidean space is a three-dimensional affine space whose arrows between points carry a
/// Euclidean metric: every arrow has a length, `‖a·u‖ = |a|·‖u‖`, and for orthogonal arrows
/// `‖u + v‖² = ‖u‖² + ‖v‖²` (Weatherall 2021, §2). In Cartesian coordinates
/// [`Distance`](crate::Distance) is therefore `√(Δx² + Δy² + Δz²)`. The coordinates carry no
/// unit of their own; the distance is in the unit the coordinates are given in.
///
/// # Coordinate index mapping
/// - `0 => x`
/// - `1 => y`
/// - `2 => z`
///
/// # Example
/// ```
/// use deep_causality_context::*;
///
/// let a = EuclideanSpace::new(1, 1.0, 2.0, 3.0);
/// let b = EuclideanSpace::new(2, 4.0, 6.0, 3.0);
///
/// assert_eq!(a.dimension(), 3);
/// assert_eq!(a.coordinate(1).unwrap(), &2.0);
/// assert_eq!(a.distance(&b), 5.0);
/// ```
///
/// # References
/// - Weatherall, J. O. (2021). Classical Spacetime Structure. In E. Knox & A. Wilson (Eds.),
///   *The Routledge Companion to Philosophy of Physics*, pp. 33–45. Routledge. arXiv:1707.05887,
///   §2. Copy: `papers/weatherall_2017_classical_spacetime_structure_arXiv_1707.05887.pdf`.
#[derive(Debug, Clone, PartialEq)]
pub struct EuclideanSpace<R>
where
    R: RealField,
{
    id: ContextoidId,
    x: R,
    y: R,
    z: R,
}

impl<R> EuclideanSpace<R>
where
    R: RealField,
{
    pub fn new(id: ContextoidId, x: R, y: R, z: R) -> Self {
        Self { id, x, y, z }
    }
}
