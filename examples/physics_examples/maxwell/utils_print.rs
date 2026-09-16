/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the Maxwell chain.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::model::{MaxwellState, PlaneWaveConfig};
use crate::{FloatType, Verification};
use deep_causality_num::lower;

pub fn print_header() {
    println!("=== Maxwell's Unification: E and B as one bivector ===");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

pub fn print_config(c: &PlaneWaveConfig) {
    println!("Plane wave  A_x(t, z) = cos(omega (t - z))");
    println!(
        "Observed at omega = {}, t = {}, z = {}\n",
        lower(c.omega),
        lower(c.t),
        lower(c.z)
    );
}

pub fn print_fields(s: &MaxwellState, blades: (FloatType, FloatType)) {
    println!("--- The potential and the field it generates ---");
    println!("  phase   omega (t - z)   = {:.6}", lower(s.phase));
    println!("  A_x     cos(phase)      = {:.6}", lower(s.potential_ax));
    println!("  E_x     -dA_x/dt        = {:.6}   [AD]", lower(s.e_field));
    println!("  B_y      dA_x/dz        = {:.6}   [AD]", lower(s.b_field));

    println!("\n--- Both fields in one Cl(1,3) bivector F ---");
    println!("  F on e_t ^ e_x  (E_x)   = {:.6}", lower(blades.0));
    println!("  F on e_z ^ e_x  (B_y)   = {:.6}", lower(blades.1));

    println!("\n--- Gauge and flux ---");
    println!("  d_mu A^mu               = {:.3e}", lower(s.divergence));
    println!("  |S| = |E x B|           = {:.6}", lower(s.poynting_flux));
}

pub fn print_verification(v: &Verification) {
    println!("\n--- Identities for a source-free plane wave ---");
    println!(
        "  tolerance               = {:.2e}   ({} rounding steps of the working type)",
        lower(v.tolerance),
        crate::TOLERANCE_ULPS
    );
    println!(
        "  |d_mu A^mu|             = {:.2e}   (Lorenz gauge)",
        lower(v.gauge_residual)
    );
    println!(
        "  |E| - |B|               = {:.2e}   (the wave travels at c)",
        lower(v.field_balance)
    );
    println!(
        "  |S| - |E||B|            = {:.2e}   (E and B are orthogonal)",
        lower(v.flux_residual)
    );
    println!(
        "  E_x - omega sin(phase)  = {:.2e}   (AD against the closed form)",
        lower(v.closed_form_residual)
    );
    println!(
        "  => {}",
        if v.holds {
            "all four hold to tolerance"
        } else {
            "AN IDENTITY FAILED"
        }
    );
}
