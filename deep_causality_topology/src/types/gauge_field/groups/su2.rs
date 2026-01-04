/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! SU(2) gauge group - Weak isospin.
//!
//! The SU(2) group has three generators (W+, W-, Z before symmetry breaking).
//! It is non-abelian, so the field strength includes self-interaction: F = dA + A∧A.

use crate::types::gauge_field::group::GaugeGroup;

/// SU(2) gauge group marker.
///
/// Represents the gauge symmetry of the weak isospin interaction.
///
/// # Properties
///
/// - **Lie algebra dimension**: 3 (three generators)
/// - **Abelian**: No (non-commutative)
/// - **Convention**: West Coast (+---) by default
///
/// # Physics
///
/// The SU(2) gauge symmetry corresponds to weak isospin rotations.
/// Before electroweak symmetry breaking, it gives rise to three gauge bosons.
/// After symmetry breaking (combined with U(1)), these become W+, W-, and Z.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SU2;

impl GaugeGroup for SU2 {
    const LIE_ALGEBRA_DIM: usize = 3;
    const IS_ABELIAN: bool = false;

    fn name() -> &'static str {
        "SU(2)"
    }
}
