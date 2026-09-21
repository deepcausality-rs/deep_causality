/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod display;

/// The analytic form a metric tensor takes, with the parameters that fix it.
///
/// A [`Metric`](crate::Metric) is a signature and is constant across a manifold: a Lorentzian
/// manifold has signature (−,+,+,+) at every point, and Schwarzschild, Kerr and FLRW are all
/// Lorentzian. A family is the further choice of *which* analytic form, and it carries numbers
/// that differ between one model and the next, so it is a value rather than a constant.
///
/// # What is and is not carried
///
/// Each variant carries the parameters that fix the form across the whole manifold, and none of
/// the coordinates at which it is evaluated. A Schwarzschild metric is fixed by a mass and then
/// evaluated at a radius; the mass belongs here and the radius does not.
///
/// The variants correspond one to one with the closed forms in `deep_causality_physics`, and take
/// the same shape parameters those functions do.
///
/// # The flat case is one variant
///
/// [`MetricFamily::Flat`] covers both the Minkowski and the Euclidean cases. The signature already
/// separates them, so a second variant here would encode the same distinction twice and allow the
/// two to disagree.
///
/// # The absence of a family
///
/// A numerically evolved spacetime has no closed form at all: the tensor is carried per point and
/// nothing here describes it. That case is spelled `None` by whoever returns a family, rather than
/// by a variant, because a family that describes nothing is not a family.
///
/// # Example
/// ```
/// use deep_causality_metric::MetricFamily;
///
/// let flat: MetricFamily<f64> = MetricFamily::Flat;
/// let hole = MetricFamily::Schwarzschild { mass: 1.0 };
///
/// assert_ne!(flat, hole);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetricFamily<R> {
    /// No curvature. The signature says whether this is the Minkowski or the Euclidean case.
    Flat,

    /// The spherically symmetric vacuum solution, fixed by the central mass.
    Schwarzschild {
        /// Central mass, in geometric units.
        mass: R,
    },

    /// The axially symmetric rotating vacuum solution, fixed by mass and spin.
    Kerr {
        /// Central mass, in geometric units.
        mass: R,
        /// Angular momentum per unit mass, conventionally written `a`.
        spin: R,
    },

    /// The homogeneous isotropic cosmological solution.
    Flrw {
        /// The scale factor, which is positive.
        scale_factor: R,
        /// Spatial curvature: negative, zero or positive for open, flat and closed.
        curvature_k: R,
    },
}
