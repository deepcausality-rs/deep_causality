/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the hyperlens sweep.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use crate::model::{
    AXIS_X, AXIS_Y, AXIS_Z, WAVELENGTH_NM, ZERO, free_space_wavenumber, permittivity,
};
use deep_causality_metric::Metric;
use deep_causality_num::lower;
use deep_causality_tensor::CausalTensor;

pub fn print_header() {
    println!("=== Hyperbolic metamaterial lens: the metric is the material ===\n");
    println!("Precision:   {}", core::any::type_name::<FloatType>());
    println!("Wavelength:  {:.0} nm", lower(WAVELENGTH_NM));
    println!(
        "k_0 = 2pi/lambda:  {:.6} rad/nm\n",
        lower(free_space_wavenumber())
    );
}

/// The two materials, showing that each principal permittivity comes from the metric signature.
pub fn print_materials(vacuum: &Metric, lens: &Metric) {
    println!("Materials, as metric signatures");
    println!("  material               metric          eps_x   eps_y   eps_z");

    for (name, metric) in [("vacuum", vacuum), ("Type I metamaterial", lens)] {
        println!(
            "  {:<21}  {:<14}  {:>+5.1}   {:>+5.1}   {:>+5.1}",
            name,
            format!("{metric}"),
            lower(permittivity(metric, AXIS_X)),
            lower(permittivity(metric, AXIS_Y)),
            lower(permittivity(metric, AXIS_Z))
        );
    }
    println!();
    println!("  The optics reads every sign through `Metric::sign_of_sq`, so the signature alone");
    println!("  decides whether the dispersion surface is a sphere or a hyperboloid.\n");
}

/// The sweep: one row per object period, with the outcome under each material.
pub fn print_sweep(
    periods: &CausalTensor<FloatType>,
    in_vacuum: &CausalTensor<FloatType>,
    in_lens: &CausalTensor<FloatType>,
) {
    println!("Dispersion sweep");
    println!("  period     vacuum k_z^2   outcome       lens k_z^2     outcome");
    println!("   (nm)       (rad/nm)^2                  (rad/nm)^2");

    for ((&period, &vacuum), &lens) in periods
        .as_slice()
        .iter()
        .zip(in_vacuum.as_slice())
        .zip(in_lens.as_slice())
    {
        println!(
            "  {:>6.0}   {:>+13.3e}   {:<12}  {:>+11.3e}   {}",
            lower(period),
            lower(vacuum),
            outcome(vacuum),
            lower(lens),
            outcome(lens)
        );
    }
    println!();
}

/// What a squared wavenumber means for the wave that carries it.
fn outcome(k_z_squared: FloatType) -> &'static str {
    if k_z_squared < ZERO {
        "evanescent"
    } else {
        "propagates"
    }
}

pub fn print_verdict(vacuum_limit: Option<FloatType>, lens_limit: Option<FloatType>) {
    println!("Resolution limit, the finest period that still propagates");

    match vacuum_limit {
        Some(limit) => println!(
            "  vacuum                {:.0} nm, which is the wavelength itself",
            lower(limit)
        ),
        None => println!("  vacuum                nothing in this sweep propagates"),
    }

    match lens_limit {
        Some(limit) => println!(
            "  Type I metamaterial   {:.0} nm, the finest period probed",
            lower(limit)
        ),
        None => println!("  Type I metamaterial   nothing in this sweep propagates"),
    }

    println!();
    println!(
        "In vacuum a period below the wavelength drives k_z^2 negative and the detail decays."
    );
    println!(
        "Flipping one permittivity sign keeps k_z^2 positive at every period in the sweep, so"
    );
    println!("the limit is set by how fine an object is probed rather than by the optics.");
}
