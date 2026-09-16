/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Electroweak unification: the W mass, from first principles to within 8 MeV
//!
//! The electromagnetic and weak forces are one force above about 100 GeV, described by a gauge
//! theory with symmetry `SU(2) × U(1)`. Below that scale the Higgs field takes a vacuum value and
//! the symmetry breaks: three of the four gauge bosons acquire mass and become the `W⁺`, `W⁻` and
//! `Z`, and the fourth stays massless and is the photon.
//!
//! The theory then has almost no freedom left. Fix the fine-structure constant, the Fermi constant
//! and the Z mass, and everything else is predicted — including the W mass, which is measured to
//! better than a part in ten thousand. Comparing the two is one of the sharpest tests the Standard
//! Model faces.
//!
//! # Why the tree level is not enough
//!
//! At tree level `M_W = g·v/2`, which gives about 78.9 GeV against a measured 80.377. That is a
//! gap of 1.5 GeV, roughly two percent, and it is not experimental error: it is the one-loop
//! radiative corrections, dominated by the top quark running around the loop. The `ρ` parameter is
//! exactly 1 at tree level, and the loops move it by `Δρ ≈ 0.009`.
//!
//! The run prints both, so what the corrections are worth is visible rather than asserted.
//!
//! # What the run does
//!
//! Four stages, composed as one `CausalFlow`, each adding to the state the next one reads:
//!
//! ```text
//! bind   unification         couplings g and g' from α_EM and θ_W
//! bind   symmetry breaking   masses from the Higgs vacuum value
//! bind   gauge mixing        the W/Z mass ratio and the ρ parameter
//! bind   Z resonance         the widths and the peak cross-section
//! ```
//!
//! `bind_or_error` is what makes the chain a chain: a stage that fails stops the ones after it and
//! carries its reason to the summary, so no stage reads a state an earlier one never filled.

mod model;
mod utils_print;

use deep_causality_core::{
    CausalEffectPropagationProcess, CausalFlow, CausalityError, CausalityErrorEnum,
    PropagatingEffect,
};
use deep_causality_num::Float106;
use deep_causality_physics::ElectroweakParams;
use model::{EwState, NEUTRINO_CHARGE, NEUTRINO_GENERATIONS, NEUTRINO_ISOSPIN, ONE, TWO};
use utils_print::{
    print_gauge_mixing, print_header, print_resonance, print_summary, print_symmetry_breaking,
    print_unification,
};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the couplings,
/// the masses, the widths and the cross-section all recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let result = CausalFlow::from(stage_unification())
        .bind_or_error(stage_symmetry_breaking, "Symmetry breaking failed")
        .bind_or_error(stage_gauge_mixing, "Gauge mixing failed")
        .bind_or_error(stage_z_resonance, "Resonance calculation failed")
        .into_effect();

    print_summary(&result);

    Ok(())
}

/// Stage 1: the couplings.
///
/// `g` and `g'` follow from the electromagnetic coupling and the weak mixing angle, which is what
/// unification means in practice: one charge and one angle in place of two independent forces.
fn stage_unification() -> PropagatingEffect<EwState> {
    let params = ElectroweakParams::standard_model_precision();
    print_unification(&params);

    CausalEffectPropagationProcess::pure(EwState {
        params: Some(params),
        ..Default::default()
    })
}

/// Stage 2: the masses the Higgs vacuum value generates.
///
/// Both W masses are recorded: the tree relation `g·v/2` and the loop-corrected solution. The gap
/// between them is what the corrections are worth, and keeping both is what lets the summary say so
/// rather than print one number under two labels.
fn stage_symmetry_breaking(mut state: EwState, _: (), _: Option<()>) -> PropagatingEffect<EwState> {
    if let Some(params) = state.params {
        state.higgs_quartic = params.higgs_quartic();
        state.top_yukawa = params.top_yukawa();
        state.w_mass_tree = params.g_coupling() * params.higgs_vev() / TWO;
        state.w_mass = params.w_mass_computed();
        state.z_mass = params.z_mass_computed();

        print_symmetry_breaking(&params, &state);
    }

    CausalEffectPropagationProcess::pure(state)
}

/// Stage 3: the mass relation, and how far the loops move it.
fn stage_gauge_mixing(mut state: EwState, _: (), _: Option<()>) -> PropagatingEffect<EwState> {
    if let Some(params) = state.params {
        state.delta_rho = params.rho_effective() - ONE;

        print_gauge_mixing(&params, &state);
    }

    CausalEffectPropagationProcess::pure(state)
}

/// Stage 4: the Z resonance.
///
/// The invisible width is three neutrino generations, each a fermion of weak isospin `+1/2` and no
/// charge. Measuring it is how the generation count was established at LEP, so the number is a
/// prediction rather than an input.
fn stage_z_resonance(mut state: EwState, _: (), _: Option<()>) -> PropagatingEffect<EwState> {
    if let Some(params) = state.params {
        state.z_total_width = params.z_total_width_computed();
        state.z_hadronic_width = params.z_hadronic_width_computed();
        state.z_invisible_width = NEUTRINO_GENERATIONS
            * params.z_partial_width_fermion(false, NEUTRINO_ISOSPIN, NEUTRINO_CHARGE);

        match params.z_resonance_cross_section(state.z_mass, state.z_total_width) {
            Ok(sigma) => state.z_peak_cross_section = sigma,
            Err(e) => {
                return PropagatingEffect::from_error(CausalityError::new(
                    CausalityErrorEnum::Custom(format!("the Z cross-section is undefined: {e}")),
                ));
            }
        }

        print_resonance(&state);
    }

    CausalEffectPropagationProcess::pure(state)
}
