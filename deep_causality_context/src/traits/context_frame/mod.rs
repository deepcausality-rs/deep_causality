/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{SpaceTemporal, Spatial, Temporal};
use deep_causality_algebra::RealField;
use deep_causality_metric::MetricFamily;

/// The frame of reference a context is built in.
///
/// A frame of reference is what licenses a split into space and time, so the three member types
/// belong together. Naming them separately lets a context pair a space with a spacetime that
/// cannot contain it; a frame fixes them in one place.
///
/// # The variant enums are the ordinary case
///
/// [`SpaceKind`](crate::SpaceKind), [`TimeKind`](crate::TimeKind) and
/// [`SpaceTimeKind`](crate::SpaceTimeKind) satisfy these bounds, and a frame naming them is the
/// case to reach for by default. A frame naming concrete types is the specialisation, for a
/// context whose world is fixed and known at compile time.
///
/// Two things put it that way round. A causal model can change regime within one run — reasoning
/// one way far from a mass and another way close to it — and that is one context whose spacetime
/// varies, not two contexts. And a context assembled from stored records carries its frame per
/// record, so a read spanning several can return more than one; a fixed frame cannot hold the
/// result at all.
///
/// # The scalar binds space and spacetime, but not time
///
/// [`Self::Space`] and [`Self::SpaceTime`] are bound to measure in [`Self::Scalar`], so a frame
/// cannot report a position in one scalar and an interval in another.
///
/// [`Self::Time`] is deliberately left free. A tick is a count rather than a precision, so
/// [`DiscreteTime`](crate::DiscreteTime) and [`EntropicTime`](crate::EntropicTime) measure in an
/// integer, and a frame pairing a real-valued space with a tick-based clock is a legitimate thing
/// to want.
///
/// # The frame carries no signature
///
/// The signature belongs to the node, not to the context. Every spacetime type reports its own
/// through [`MetricSignature`](crate::MetricSignature), and a context whose spacetime varies holds
/// nodes in more than one, so no constant here could answer for all of them. A causaloid reads the
/// signature off the node in hand, which is the same shape `deep_causality_physics` already uses
/// when it reads a metric off a `CausalMultiVector`.
///
/// # The frame carries the family, and no tensor
///
/// [`Self::metric_family`] stays, because a family is not derivable from a node type. A node can
/// say it is Lorentzian; it cannot say it is Schwarzschild with a central mass of five solar
/// masses. That is a choice about the model, which is what a frame is for.
///
/// [`Self::metric_family`] is a function rather than a constant because a family carries
/// parameters that differ between one model and the next. It returns `None` for a numerically
/// evolved metric, which has no closed form; there the tensor is carried per point by the
/// coordinate type that needs it, such as [`TangentSpacetime`](crate::TangentSpacetime).
///
/// A frame carries no metric tensor. A tensor is a field varying with position, which
/// `deep_causality_physics` computes from the family's parameters.
///
/// # Example
/// ```
/// use deep_causality_context::{ContextFrame, NoSpaceTime, EuclideanTime};
/// use deep_causality_metric::MetricFamily;
///
/// struct Ticker;
///
/// impl ContextFrame for Ticker {
///     type Scalar = f64;
///     type Space = NoSpaceTime<f64>;
///     type Time = EuclideanTime<f64>;
///     type SpaceTime = NoSpaceTime<f64>;
///
///     fn metric_family() -> Option<MetricFamily<f64>> {
///         Some(MetricFamily::Flat)
///     }
/// }
///
/// // The family resolves without a context ever being built.
/// assert_eq!(Ticker::metric_family(), Some(MetricFamily::Flat));
/// ```
pub trait ContextFrame {
    /// The scalar this frame's space and spacetime measure in.
    type Scalar: RealField;

    /// The spatial type, measuring in [`Self::Scalar`].
    type Space: Spatial<Coord = Self::Scalar>;

    /// The temporal type. Not bound to [`Self::Scalar`], so a tick-based clock is allowed.
    type Time: Temporal;

    /// The spacetime type, measuring in [`Self::Scalar`].
    type SpaceTime: SpaceTemporal<Coord = Self::Scalar>;

    /// The analytic form of the metric, or `None` when it is numerically evolved and has no
    /// closed form.
    fn metric_family() -> Option<MetricFamily<Self::Scalar>>;
}
