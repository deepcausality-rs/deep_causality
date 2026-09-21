/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextFrame, EuclideanTime, FloatType, NoSpaceTime};
use deep_causality_metric::MetricFamily;

/// A frame for a context with a clock and no spatial extent, at this crate's [`FloatType`].
///
/// Both the spatial and the spacetime member are [`NoSpaceTime`], which is zero-sized, so naming
/// the absence costs nothing. The [`ContextFrame::Time`] member is a real clock: what this frame
/// lacks is a *position*, not a time.
///
/// This is the common shape rather than an edge case. Across the workspace the contextoid variants
/// are used Root 45, Datoid 47, Tempoid 15, Spaceoid 5, SpaceTempoid 5.
///
/// # Signature
///
/// [`Metric::Euclidean`] over zero dimensions: there are no axes to give a sign to. The family is
/// [`MetricFamily::Flat`], vacuously.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct ClockFrame;

impl ContextFrame for ClockFrame {
    type Scalar = FloatType;
    type Space = NoSpaceTime<FloatType>;
    type Time = EuclideanTime<FloatType>;
    type SpaceTime = NoSpaceTime<FloatType>;

    fn metric_family() -> Option<MetricFamily<FloatType>> {
        Some(MetricFamily::Flat)
    }
}
