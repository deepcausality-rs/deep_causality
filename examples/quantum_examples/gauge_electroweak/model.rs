/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the electroweak pipeline: the state the four stages thread, the constants the
//! physics is written with, and the reference values the run is judged against.
//!
//! Every constant is declared at the working type through `const_scalar_from_int!` and
//! `const_scalar_from_float!`, so the compiler resolves them against the alias in `main`.

use crate::FloatType;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int};
use deep_causality_physics::ElectroweakParams;

// =============================================================================
// The small numbers the model is written with
// =============================================================================

pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
pub const TWO: FloatType = const_scalar_from_int!(FloatType, 2);

/// Neutrino generations, which is what the invisible width of the Z counts.
pub const NEUTRINO_GENERATIONS: FloatType = const_scalar_from_int!(FloatType, 3);

/// MeV per GeV, for reporting a mass difference at the scale it is quoted in.
pub const MEV_PER_GEV: FloatType = const_scalar_from_int!(FloatType, 1000);

/// The weak isospin of a left-handed neutrino, `+1/2`, and its electric charge, `0`.
pub const NEUTRINO_ISOSPIN: FloatType = const_scalar_from_float!(FloatType, 0.5);
pub const NEUTRINO_CHARGE: FloatType = ZERO;

/// How far the computed W mass may sit from the measured one before the run calls it a deviation,
/// in GeV.
///
/// Twenty MeV is what a one-loop calculation is worth: the terms left out are two-loop and enter
/// at roughly that size. A tolerance looser than the accuracy being claimed would pass a
/// calculation that had stopped agreeing with the measurement, so this number and the claim in the
/// summary are the same number.
pub const W_MASS_TOLERANCE_GEV: FloatType = const_scalar_from_float!(FloatType, 0.020);

// =============================================================================
// The state the stages thread
// =============================================================================

/// What each stage adds to the pipeline.
///
/// Every field is printed by the summary or the stage that fills it, so the struct carries no
/// quantity the run computes and then drops.
#[derive(Debug, Clone, Default)]
pub struct EwState {
    pub params: Option<ElectroweakParams<FloatType>>,
    /// The W mass from the loop solver, in GeV.
    pub w_mass: FloatType,
    /// The W mass from the tree relation `g·v/2`, in GeV: what the theory gives before
    /// corrections, and the number the loop solver has to move.
    pub w_mass_tree: FloatType,
    /// The Z mass from `M_W / cos θ_W`, in GeV.
    pub z_mass: FloatType,
    pub higgs_quartic: FloatType,
    pub top_yukawa: FloatType,
    /// The peak cross-section on the Z resonance, in nb.
    pub z_peak_cross_section: FloatType,
    /// `ρ_eff − 1`, the size of the loop correction to the mass relation.
    pub delta_rho: FloatType,
    /// The total and hadronic Z widths, and the invisible width the neutrinos carry, in GeV.
    pub z_total_width: FloatType,
    pub z_hadronic_width: FloatType,
    pub z_invisible_width: FloatType,
}

// =============================================================================
// Reference values
// =============================================================================

/// How far the computed W mass sits from the measured one, in MeV.
pub fn w_mass_deviation_mev(state: &EwState, measured_gev: FloatType) -> FloatType {
    magnitude(state.w_mass - measured_gev) * MEV_PER_GEV
}

/// Whether the computed W mass sits inside the tolerance a one-loop calculation earns.
pub fn within_one_loop_accuracy(state: &EwState, measured_gev: FloatType) -> bool {
    magnitude(state.w_mass - measured_gev) < W_MASS_TOLERANCE_GEV
}

/// The magnitude of a signed quantity.
pub fn magnitude(x: FloatType) -> FloatType {
    if x < ZERO { -x } else { x }
}
