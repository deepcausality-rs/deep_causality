/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::Metric;

/// Reports the metric signature a spacetime node is in.
///
/// # Why the node and not the context
///
/// Every spacetime node is in some signature: [`EuclideanSpacetime`](crate::EuclideanSpacetime)
/// is in (+,+,+,+) and [`LorentzianSpacetime`](crate::LorentzianSpacetime) is in (−,+,+,+). The
/// question is never whether a signature exists, only who says what it is.
///
/// A context whose spacetime varies holds nodes in more than one signature, so no single constant
/// on the context can answer for all of them. Asking the node does, and it is the same shape
/// `deep_causality_physics` already relies on: `CausalMultiVector` carries its own metric, and the
/// kernels read it off the value rather than off a declaration.
///
/// # The signature is not the tensor
///
/// A signature says how many axes square positive, negative and zero. A tensor gives their
/// magnitudes at a point and varies across a manifold; [`MetricTensor4D`](crate::MetricTensor4D)
/// is where that lives, on the one type that carries it.
///
/// The signature does not vary with position, by continuity, so a node reports it as a value
/// derived from what it is rather than from what it currently holds. A numerically evolved
/// spacetime keeps its signature while every component of its tensor changes.
///
/// # Example
/// ```
/// use deep_causality_context::{
///     EuclideanSpacetime, LorentzianSpacetime, MetricSignature, TimeScale,
/// };
/// use deep_causality_metric::Metric;
///
/// let newtonian = EuclideanSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
/// let relativistic = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
///
/// assert_eq!(newtonian.metric(), Metric::Euclidean(4));
/// assert_eq!(relativistic.metric(), Metric::Lorentzian(4));
/// ```
pub trait MetricSignature {
    /// The metric signature this node is in.
    fn metric(&self) -> Metric;
}
