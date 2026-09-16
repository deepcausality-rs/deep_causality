/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the electroweak pipeline.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use crate::model::{
    EwState, W_MASS_TOLERANCE_GEV, magnitude, w_mass_deviation_mev, within_one_loop_accuracy,
};
use deep_causality_core::PropagatingEffect;
use deep_causality_num::lower;
use deep_causality_physics::ElectroweakParams;

pub fn print_header() {
    println!("=== Electroweak unification: the W mass, from first principles ===\n");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!("Scheme:    on-shell, with one-loop radiative corrections\n");
}

/// Stage 1: the couplings, and the two mixing angles the schemes use.
pub fn print_unification(params: &ElectroweakParams<FloatType>) {
    println!("Stage 1: unification");
    println!(
        "  EM coupling        e   = {:.9}",
        lower(params.em_coupling())
    );
    println!(
        "  weak coupling      g   = {:.9}",
        lower(params.g_coupling())
    );
    println!(
        "  hypercharge        g'  = {:.9}",
        lower(params.g_prime_coupling())
    );
    println!(
        "  on-shell angle     sin^2 th_W = {:.9}",
        lower(params.sin2_theta_w())
    );

    if let Some(c) = params.corrections() {
        println!(
            "  effective angle    sin^2 th_eff = {:.9}",
            lower(c.sin2_theta_eff)
        );
        println!("  Veltman screening  d_rho = {:.9}", lower(c.delta_rho));
        println!("  radiative          d_r   = {:.9}", lower(c.delta_r));
        println!();
        println!("  The two angles differ because they are defined by different measurements:");
        println!("  one by the boson masses and one by the Z decay asymmetries. At tree level");
        println!("  they are the same number, and the gap between them is a loop effect.");
    }
    println!();
}

/// Stage 2: the masses the Higgs vacuum value generates, tree and corrected.
pub fn print_symmetry_breaking(params: &ElectroweakParams<FloatType>, state: &EwState) {
    println!("Stage 2: spontaneous symmetry breaking");
    println!(
        "  Higgs VEV          v   = {:.4} GeV",
        lower(params.higgs_vev())
    );
    println!(
        "  quartic coupling   l   = {:.9}",
        lower(state.higgs_quartic)
    );
    println!("  top Yukawa         y_t = {:.9}", lower(state.top_yukawa));
    println!();
    println!(
        "  M_W tree level     {:>12.6} GeV   from g*v/2",
        lower(state.w_mass_tree)
    );
    println!(
        "  M_W loop corrected {:>12.6} GeV   from the loop solver",
        lower(state.w_mass)
    );
    println!(
        "  the corrections move it by {:>7.6} GeV",
        lower(magnitude(state.w_mass - state.w_mass_tree))
    );
    println!(
        "  M_Z                {:>12.6} GeV   from M_W / cos th_W",
        lower(state.z_mass)
    );
    println!();
}

/// Stage 3: the mass relation, against the measurement.
pub fn print_gauge_mixing(params: &ElectroweakParams<FloatType>, state: &EwState) {
    let measured = params.w_mass();

    println!("Stage 3: gauge boson mixing");
    println!(
        "  rho tree level     {:.9}   the relation M_W = M_Z cos th_W, exactly",
        lower(params.rho_parameter_computed())
    );
    println!(
        "  rho effective      {:.9}   with the loop correction",
        lower(params.rho_effective())
    );
    println!(
        "  d_rho = rho - 1    {:.9}   dominated by the top quark in the loop",
        lower(state.delta_rho)
    );
    println!();
    println!("  M_W computed       {:>12.6} GeV", lower(state.w_mass));
    println!("  M_W measured       {:>12.6} GeV   PDG", lower(measured));
    println!(
        "  deviation          {:>12.3} MeV   tolerance {:.0} MeV",
        lower(w_mass_deviation_mev(state, measured)),
        lower(W_MASS_TOLERANCE_GEV * crate::model::MEV_PER_GEV)
    );
    println!(
        "  verdict            {}",
        if within_one_loop_accuracy(state, measured) {
            "inside one-loop accuracy"
        } else {
            "outside one-loop accuracy"
        }
    );
    println!();
}

/// Stage 4: the Z resonance.
pub fn print_resonance(state: &EwState) {
    println!("Stage 4: the Z resonance");
    println!("  peak energy        {:>12.6} GeV", lower(state.z_mass));
    println!(
        "  total width        {:>12.6} GeV   G_Z",
        lower(state.z_total_width)
    );
    println!(
        "  hadronic width     {:>12.6} GeV   G_had",
        lower(state.z_hadronic_width)
    );
    println!(
        "  invisible width    {:>12.6} GeV   three neutrino generations",
        lower(state.z_invisible_width)
    );
    println!(
        "  peak cross-section {:>12.6} nb",
        lower(state.z_peak_cross_section)
    );
    println!();
    println!("  The invisible width is a prediction, not an input. Measuring it at LEP is how");
    println!("  the number of light neutrino generations was established to be three.");
    println!();
}

/// What the pipeline came to.
pub fn print_summary(result: &PropagatingEffect<EwState>) {
    let Some(state) = result.value() else {
        println!("The pipeline failed: {:?}", result.error());
        return;
    };

    let Some(params) = state.params else {
        println!("The pipeline produced no parameters.");
        return;
    };

    println!("Summary");
    println!("  M_W from the theory   {:>12.6} GeV", lower(state.w_mass));
    println!(
        "  M_W from experiment   {:>12.6} GeV",
        lower(params.w_mass())
    );
    println!(
        "  deviation             {:>12.3} MeV",
        lower(w_mass_deviation_mev(state, params.w_mass()))
    );
    println!(
        "  peak cross-section    {:>12.6} nb",
        lower(state.z_peak_cross_section)
    );
    println!("  d_rho                 {:>12.9}", lower(state.delta_rho));
    println!();
    println!("  Fix three measured numbers and the rest of the electroweak sector is predicted.");
    println!("  The tree relation misses the W mass by about 1.5 GeV; adding one loop brings it");
    println!("  to within a few MeV of a measurement good to a part in ten thousand. What is");
    println!("  left over is where the two-loop terms sit, and where anything beyond the");
    println!("  Standard Model would have to show up.");
}
