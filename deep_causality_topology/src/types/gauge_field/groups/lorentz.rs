/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Lorentz SO(3,1) gauge group - General Relativity.
//!
//! The Lorentz group has six generators (3 rotations + 3 boosts).
//! When used as a gauge group, it provides the frame bundle for spacetime.

use crate::types::gauge_field::group::GaugeGroup;
use deep_causality_metric::Metric;

/// Lorentz SO(3,1) gauge group marker.
///
/// Represents the gauge symmetry of General Relativity (GR).
///
/// # Properties
///
/// - **Lie algebra dimension**: 6 (3 rotations + 3 boosts)
/// - **Abelian**: No (non-commutative)
/// - **Convention**: East Coast (-+++) by default (GR standard)
///
/// # Physics
///
/// The Lorentz group SO(3,1) describes spacetime symmetries:
/// - 3 spatial rotations (J₁, J₂, J₃)
/// - 3 Lorentz boosts (K₁, K₂, K₃)
///
/// In the gauge formulation of GR, the Lorentz group acts on the frame bundle,
/// with the connection (spin connection) measuring how frames rotate
/// as we move through spacetime.
///
/// # Metric Convention
///
/// GR traditionally uses the East Coast (MTW) convention:
/// η = diag(-1, +1, +1, +1)
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Lorentz;

impl GaugeGroup for Lorentz {
    const LIE_ALGEBRA_DIM: usize = 6;
    const IS_ABELIAN: bool = false;

    fn name() -> &'static str {
        "SO(3,1)"
    }

    /// East Coast convention (-+++) for GR.
    fn default_metric() -> Metric {
        // East Coast: time is negative, space is positive
        // Metric::from_signature(p, q, r) where p=positive, q=negative, r=degenerate
        // For East Coast (-+++): one negative (time), three positive (space)
        // So we use (3, 1, 0) - three +1, one -1
        Metric::Generic { p: 3, q: 1, r: 0 }
    }
}
