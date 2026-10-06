/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::UpdateError;
use crate::traits::contextuable::coordinate::Coordinate;

/// Read and write access to the metric tensor `g_ab` at a spacetime event.
///
/// The tensor is a 4×4 matrix indexed in the implementor's coordinate order, which for every
/// spacetime of this crate is `t, x, y, z`, the order of the coordinate index mapping
/// `x⁰ = ct, x¹ = x, x² = y, x³ = z` (Carroll 1997, eq. (1.5)). A metric is symmetric, and on a
/// relativistic spacetime it has Lorentz signature (Malament 2012, §2.1, p. 119); an implementor
/// refuses a tensor that breaks either, so the tensor it holds always satisfies both.
///
/// # The scalar
/// The metric is contracted with the coordinates of the point it belongs to, so this trait reads
/// its scalar from [`Coordinate::Coord`] rather than declaring one of its own.
///
/// # References
/// - Carroll, S. M. (1997). *Lecture Notes on General Relativity*. arXiv:gr-qc/9712019, ch. 1.
/// - Malament, D. B. (2012). *Topics in the Foundations of General Relativity and Newtonian
///   Gravitation Theory*. University of Chicago Press. §2.1.
pub trait MetricTensor4D: Coordinate {
    /// The metric tensor `g_ab` at this event.
    fn metric_tensor(&self) -> [[Self::Coord; 4]; 4];

    /// Replaces the metric tensor with `new_metric`.
    ///
    /// # Errors
    /// [`UpdateError`] when `new_metric` has a non-finite entry, is not symmetric, or is not of the
    /// implementor's signature; the tensor is then unchanged.
    fn update_metric_tensor(
        &mut self,
        new_metric: [[Self::Coord; 4]; 4],
    ) -> Result<(), UpdateError>;
}
