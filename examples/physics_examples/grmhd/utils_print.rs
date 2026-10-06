/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the GRMHD chain.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::model::{
    CENTRAL_MASS, COLUMN_LENGTH, CURRENT_DENSITY, GrmhdContext, GrmhdState, MAGNETIC_FIELD,
    ORBIT_RADIUS, TIDAL_THRESHOLD, curvature_radius, orbit_radius, read, schwarzschild_radius,
    tidal_acceleration_si,
};
use crate::{FloatType, TOLERANCE_ULPS, Verification};
use deep_causality_core::CausalityError;
use deep_causality_num::lower;

pub fn print_header() {
    println!("=== GRMHD: General Relativistic Magnetohydrodynamics ===");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!("Units: geometric (G = c = 1), so a mass is a length and an acceleration is 1/m\n");
}

pub fn print_config(world: &GrmhdContext) -> Result<(), CausalityError> {
    let r_s = schwarzschild_radius(world)?;
    let central_mass = read(world, CENTRAL_MASS)?;
    let r = orbit_radius(world)?;
    let orbit_radii = read(world, ORBIT_RADIUS)?;
    let column_length = read(world, COLUMN_LENGTH)?;
    let current_density = read(world, CURRENT_DENSITY)?;
    let magnetic_field = read(world, MAGNETIC_FIELD)?;

    println!("Central body and plasma:");
    println!(
        "  Schwarzschild radius r_s = {:.4e} m  ({:.0} solar masses)",
        lower(r_s),
        lower(central_mass)
    );
    println!(
        "  Plasma radius r          = {:.4e} m  ({:.0} r_s, equatorial plane)",
        lower(r),
        lower(orbit_radii)
    );
    println!(
        "  Column length L          = {:.4e} m",
        lower(column_length)
    );
    println!(
        "  Current J, field B       = {:.2}, {:.2}  (as the static observer measures them)\n",
        lower(current_density),
        lower(magnetic_field)
    );
    Ok(())
}

pub fn print_report(s: &GrmhdState, world: &GrmhdContext) -> Result<(), CausalityError> {
    let tidal_threshold = read(world, TIDAL_THRESHOLD)?;

    println!("[1] GR solver: curvature from the Schwarzschild solution");
    println!(
        "      M = r_s / 2               = {:.4e} m",
        lower(s.mass_geometric)
    );
    println!("      lapse 1 - r_s/r           = {:.4}", lower(s.lapse));
    println!(
        "      Ricci scalar R            = {:.4e}        (vacuum exterior, so zero)",
        lower(s.ricci_scalar)
    );
    println!(
        "      Kretschmann K = 48M^2/r^6 = {:.4e} 1/m^4  (non-zero where Ricci is not)",
        lower(s.kretschmann)
    );
    println!(
        "      curvature radius K^-1/4   = {:.4e} m",
        lower(curvature_radius(s.kretschmann))
    );
    println!(
        "      tidal stretch 2ML/r^3     = {:.4e} 1/m    (= {:.4e} m/s^2 across the column)",
        lower(s.tidal_acceleration),
        lower(tidal_acceleration_si(s.tidal_acceleration))
    );

    println!("\n[2] Coupling: the computed tide selects the algebra");
    println!(
        "      tide {:.2e} vs threshold {:.2e}  (both 1/m)",
        lower(s.tidal_acceleration),
        lower(tidal_threshold)
    );
    println!("      selected metric           = {}", s.metric_label);

    println!("\n[3] MHD solver: Lorentz force in that algebra");
    println!(
        "      F = J ^ B on e_1 ^ e_2    = {:.4}",
        lower(s.lorentz_force)
    );

    println!("\n[4] Feedback: EM stress-energy on the Schwarzschild metric");
    println!("      g_uv                      = diag(-lapse, 1/lapse, r^2, r^2)");
    println!(
        "      F^(r theta) = B sqrt(lapse)/r = {:.4e}   (the observer's B, in coordinates)",
        lower(s.em_tensor_component)
    );
    println!(
        "      T^tt in coordinates       = {:.4}",
        lower(s.em_energy_coordinate)
    );
    println!(
        "      rho_EM = lapse T^tt       = {:.4}     (what the observer measures)",
        lower(s.em_energy_density)
    );

    println!("\n[5] Analysis: two curvatures, both in 1/m^2");
    println!(
        "      8 pi rho_EM               = {:.4e}  (sourced by the plasma)",
        lower(s.plasma_curvature)
    );
    println!(
        "      sqrt(K)                   = {:.4e}  (imposed by the hole)",
        lower(s.tidal_curvature)
    );
    println!("      {}", s.status);

    println!("\nData flow: spacetime geometry -> coupling -> plasma physics -> gravity feedback");
    Ok(())
}

pub fn print_verification(v: &Verification) {
    println!("\n--- Check: the observer's energy density against its closed form ---");
    println!(
        "  |rho_EM - B^2/2| / (B^2/2)  = {:.2e}   (tolerance {:.2e}, {} rounding steps)",
        lower(v.energy_residual),
        lower(v.tolerance),
        TOLERANCE_ULPS
    );
    println!(
        "  => {}",
        if v.holds {
            "the frame projection recovers B^2/2"
        } else {
            "THE CHECK FAILED"
        }
    );
}
