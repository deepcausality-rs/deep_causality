/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextFrame, FloatType, SpaceKind, SpaceTimeKind, TimeKind};
use deep_causality_metric::MetricFamily;

/// A frame over the variant `Kind` enums, at this crate's [`FloatType`].
///
/// This is the ordinary frame. Its members accept any of the concrete spatial, temporal and
/// spacetime variants, so one context can hold a spacetime contextoid in one regime and a second
/// in another — which is what a model that reasons one way far from a mass and another way close
/// to it needs.
///
/// # The signature comes from the node
///
/// This frame declares none. Its members span several manifolds — [`SpaceTimeKind`] holds the
/// Newtonian [`EuclideanSpacetime`](crate::EuclideanSpacetime) at signature (+,+,+,+) beside the
/// relativistic [`LorentzianSpacetime`](crate::LorentzianSpacetime) and
/// [`TangentSpacetime`](crate::TangentSpacetime) at (−,+,+,+) — so any one constant here would be
/// a claim the contents could contradict.
///
/// Each node answers for itself through [`MetricSignature`](crate::MetricSignature), and a
/// causaloid reads the signature off the node in hand. That is the same shape
/// `deep_causality_physics` already uses when it reads a metric off a `CausalMultiVector` rather
/// than off a declaration.
///
/// # No family
///
/// A context whose spacetime varies has no single closed form, so [`ContextFrame::metric_family`]
/// returns `None`. Where a per-point tensor is needed, `TangentSpacetime` carries it.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct UniformFrame;

impl ContextFrame for UniformFrame {
    type Scalar = FloatType;
    type Space = SpaceKind<FloatType>;
    type Time = TimeKind<FloatType>;
    type SpaceTime = SpaceTimeKind<FloatType>;

    fn metric_family() -> Option<MetricFamily<FloatType>> {
        None
    }
}
