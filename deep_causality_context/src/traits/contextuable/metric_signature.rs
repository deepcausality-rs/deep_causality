/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::Metric;

/// Reports the signature of the metric a spacetime node measures separations with, with
/// generator `i` the node's coordinate `i`.
///
/// Which metric that is depends on the geometry:
/// - A relativistic spacetime has one metric `g_ab`, of Lorentz signature (Malament 2012, §2.1,
///   p. 119). [`MinkowskiSpacetime`](crate::MinkowskiSpacetime) and
///   [`TangentSpacetime`](crate::TangentSpacetime) report `Metric::Lorentzian(4)`, (−, +, +, +).
/// - A classical spacetime has two degenerate metrics: a temporal metric `t_ab` of signature
///   (1, 0, 0, 0) and a spatial metric `h^ab` of signature (0, 1, 1, 1) (Malament 2012, §4.1,
///   pp. 249–250). [`GalileanSpacetime`](crate::GalileanSpacetime) and
///   [`NewtonianSpacetime`](crate::NewtonianSpacetime) report the spatial one, (0, +, +, +),
///   which the metric crate names `Metric::PGA(4)`. For Newtonian spacetime that is also the
///   signature of `ĥ_ab`, the spatial metric relative to absolute space (Malament 2012,
///   Proposition 4.1.2).
///
/// A context whose spacetime varies holds nodes of more than one geometry, so no single constant
/// on the context can answer for all of them. Asking the node does.
///
/// # The signature is not the tensor
///
/// A signature says how many axes square positive, negative and zero. A tensor gives their
/// magnitudes at a point and varies across a manifold; [`MetricTensor4D`](crate::MetricTensor4D)
/// is where that lives, on the one type that carries it.
///
/// # Example
/// ```
/// use deep_causality_context::{
///     MetricSignature, MinkowskiSpacetime, NewtonianSpacetime, TimeScale,
/// };
/// use deep_causality_metric::Metric;
///
/// let newtonian = NewtonianSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
/// let relativistic = MinkowskiSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
///
/// assert_eq!(newtonian.metric(), Metric::PGA(4));
/// assert_eq!(newtonian.metric().sign_of_sq(0), 0); // the time axis is null
/// assert_eq!(relativistic.metric(), Metric::Lorentzian(4));
/// assert_eq!(relativistic.metric().sign_of_sq(0), -1); // the time axis is timelike
/// ```
///
/// # References
/// - Malament, D. B. (2012). *Topics in the Foundations of General Relativity and Newtonian
///   Gravitation Theory*. University of Chicago Press. §2.1 and §4.1.
pub trait MetricSignature {
    /// The signature, with generator `i` the node's coordinate `i`.
    fn metric(&self) -> Metric;
}
