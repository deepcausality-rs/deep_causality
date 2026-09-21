/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextFrame, EuclideanSpace, EuclideanSpacetime, EuclideanTime, FloatType};
use deep_causality_metric::MetricFamily;

/// A frame naming concrete Euclidean types, at this crate's [`FloatType`].
///
/// This is the specialisation, for a context whose world is fixed and known at compile time. It
/// matches the members of [`BaseContext`](crate::BaseContext). Prefer
/// [`UniformFrame`](crate::UniformFrame) when the world is not known at compile time, because a
/// fixed frame cannot hold a context whose spacetime varies.
///
/// # Signature
///
/// [`Metric::Euclidean`] over four dimensions: three spatial axes and an imaginary-time axis.
/// This is the Wick-rotated signature (+,+,+,+) that quantum and statistical models work in, not
/// the Lorentzian signature of a relativistic one.
///
/// The metric is flat, so the family is [`MetricFamily::Flat`] and the signature says which flat
/// case it is.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct BaseFrame;

impl ContextFrame for BaseFrame {
    type Scalar = FloatType;
    type Space = EuclideanSpace<FloatType>;
    type Time = EuclideanTime<FloatType>;
    type SpaceTime = EuclideanSpacetime<FloatType>;

    fn metric_family() -> Option<MetricFamily<FloatType>> {
        Some(MetricFamily::Flat)
    }
}
