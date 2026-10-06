/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lift};
mod adjustable;
mod coordinate;
mod display;
mod getters;
mod identifiable;
mod metric_signature;
mod metric_tensor;
mod recordable;
mod space_temporal;
mod space_temporal_interval;
mod spatial;
mod temporal;

/// An event `p` of a relativistic spacetime, a tangent vector at `p`, and the metric tensor
/// `g_ab(p)` there, in coordinates `(t, x, y, z)` with `t` in seconds and `x, y, z` in metres.
///
/// A relativistic spacetime is a four-dimensional manifold with a metric of Lorentz signature
/// (Malament 2012, §2.1, p. 119). This type stores the metric only at its own event, as the 4×4
/// matrix `g` indexed in the coordinate order `t, x, y, z`. It starts as the Minkowski metric in
/// these units, `diag(−c², 1, 1, 1)`, which is `η = diag(−1, 1, 1, 1)` on `x⁰ = ct`
/// (Carroll 1997, eqs. (1.5) and (1.8)), and
/// [`update_metric_tensor`](crate::MetricTensor4D::update_metric_tensor) accepts only a
/// symmetric tensor of signature (−, +, +, +), so `metric()` stays `Metric::Lorentzian(4)`.
///
/// # The interval
/// [`interval_squared`](crate::SpaceTemporalInterval::interval_squared) returns
/// `g_ab(p) Δxᵃ Δxᵇ`, the coordinate displacement to the other event squared under the metric at
/// `self`. In flat spacetime with inertial coordinates this is the interval: twice Synge's world
/// function, `2σ = η_ab Δxᵃ Δxᵇ` (Poisson, Pound & Vega 2011, §3.1). On a curved manifold `σ`
/// is defined by an integral along the geodesic joining the events (eq. (3.1) there) and in
/// general differs from this value, and because the value uses `self`'s tensor,
/// `a.interval_squared(&b)` and `b.interval_squared(&a)` differ when the two tensors differ.
///
/// # The tangent vector
/// `(dt, dx, dy, dz)` are the components of a tangent vector at `p` in the same coordinate
/// order, typically the four-velocity `dxᵃ/dτ`.
///
/// # Coordinate index mapping
/// - `0 => t`
/// - `1 => x`
/// - `2 => y`
/// - `3 => z`
///
/// # Example
/// ```
/// use deep_causality_context::*;
///
/// // Arguments: id, x, y, z, t, dt, dx, dy, dz.
/// let mut s = TangentSpacetime::new(1, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0);
///
/// // A Lorentzian tensor with anisotropic spatial part, indexed t, x, y, z.
/// let warped = [
///     [-8.98755179e16, 0.0, 0.0, 0.0],
///     [0.0, 1.05, 0.0, 0.0],
///     [0.0, 0.0, 0.95, 0.0],
///     [0.0, 0.0, 0.0, 0.90],
/// ];
/// assert!(s.update_metric_tensor(warped).is_ok());
///
/// // A positive-definite tensor is not Lorentzian and is refused.
/// let riemannian = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
/// assert!(s.update_metric_tensor(riemannian).is_err());
/// ```
///
/// # References
/// - Malament, D. B. (2012). *Topics in the Foundations of General Relativity and Newtonian
///   Gravitation Theory*. University of Chicago Press. §2.1, p. 119.
/// - Carroll, S. M. (1997). *Lecture Notes on General Relativity*. arXiv:gr-qc/9712019, ch. 1.
///   Copy: `papers/carroll_1997_lecture_notes_on_general_relativity_arXiv_gr-qc_9712019.pdf`.
/// - Poisson, E., Pound, A., & Vega, I. (2011). The Motion of Point Particles in Curved
///   Spacetime. *Living Reviews in Relativity*, article 7. doi:10.12942/lrr-2011-7.
///   arXiv:1102.0529. Copy:
///   `papers/poisson_pound_vega_2011_motion_of_point_particles_in_curved_spacetime_arXiv_1102.0529.pdf`.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TangentSpacetime<R>
where
    R: RealField,
{
    id: ContextoidId,

    /// Coordinate time, in seconds.
    t: R,
    /// Position, in metres.
    x: R,
    y: R,
    z: R,
    /// Tangent vector components, in the coordinate order.
    dt: R,
    dx: R,
    dy: R,
    dz: R,
    /// The metric tensor at this event, indexed `t, x, y, z`; symmetric, of signature (−, +, +, +).
    metric: [[R; 4]; 4],
}

impl<R: RealField + FromPrimitive> TangentSpacetime<R> {
    /// An event at `(t, x, y, z)` with tangent vector `(dt, dx, dy, dz)` and the Minkowski metric
    /// `diag(−c², 1, 1, 1)`, `c = 299 792 458 m/s`. The arguments are given as
    /// `id, x, y, z, t, dt, dx, dy, dz`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(id: ContextoidId, x: R, y: R, z: R, t: R, dt: R, dx: R, dy: R, dz: R) -> Self {
        let c: R = lift(299_792_458.0);
        let zero = R::zero();
        let one = R::one();
        let metric = [
            [-(c * c), zero, zero, zero],
            [zero, one, zero, zero],
            [zero, zero, one, zero],
            [zero, zero, zero, one],
        ];

        Self {
            id,
            t,
            x,
            y,
            z,
            dt,
            dx,
            dy,
            dz,
            metric,
        }
    }
}
