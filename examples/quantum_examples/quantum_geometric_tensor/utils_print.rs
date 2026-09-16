/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the quantum geometric tensor.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use crate::model::{AXIS_NAMES, AXIS_PAIRS, TwoBandModel, lattice_constant_nm};
use deep_causality_num::lower;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

pub fn print_header() {
    println!("=== The quantum geometric tensor, and the transport it forces ===\n");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!("Units:     energies in meV, lengths in nm, velocities in meV*nm");
    println!("System:    a flat band and one remote band, as in magic-angle TBG\n");
}

/// The band structure the tensor is computed from.
pub fn print_bands(model: &TwoBandModel) {
    let energies = model.energies.as_slice();

    println!("Bands");
    println!("  band 0 (flat)     {:>8.3} meV", lower(energies[0]));
    println!("  band 1 (remote)   {:>8.3} meV", lower(energies[1]));
    println!(
        "  gap               {:>8.3} meV",
        lower(energies[1] - energies[0])
    );
    println!(
        "  lattice constant  {:>8.3} nm\n",
        lower(lattice_constant_nm())
    );
}

/// The tensor and its two parts, one row per axis pair.
pub fn print_tensor(
    qgt: &CausalTensor<Complex<FloatType>>,
    metric: &CausalTensor<FloatType>,
    curvature: &CausalTensor<FloatType>,
) {
    println!("Q_ij for the flat band, and its decomposition");
    println!("  pair      Re(Q_ij)      Im(Q_ij)      g_ij          Omega_ij");
    println!("                                        (nm^2)        (nm^2)");

    // The tensor was built by mapping over `AXIS_PAIRS`, so walking the pairs alongside the cells
    // pairs each component with the axes it belongs to.
    for (cell, (&(i, j), q)) in AXIS_PAIRS.iter().zip(qgt.as_slice()).enumerate() {
        println!(
            "  Q_{}{}    {:>11.6}   {:>11.6}   {:>11.6}   {:>11.6}",
            AXIS_NAMES[i],
            AXIS_NAMES[j],
            lower(q.re),
            lower(q.im),
            lower(metric.as_slice()[cell]),
            lower(curvature.as_slice()[cell])
        );
    }
    println!();
}

/// The symmetry each part is forced to have, checked against what came out.
pub fn print_symmetry(metric: &CausalTensor<FloatType>, curvature: &CausalTensor<FloatType>) {
    let g_xy = metric.as_slice()[1];
    let g_yx = metric.as_slice()[2];
    let omega_xx = curvature.as_slice()[0];
    let omega_xy = curvature.as_slice()[1];
    let omega_yx = curvature.as_slice()[2];

    println!("The symmetry the decomposition forces");
    println!(
        "  g_xy - g_yx            {:>12.3e}   the metric is symmetric",
        lower(g_xy - g_yx)
    );
    println!(
        "  Omega_xy + Omega_yx    {:>12.3e}   the curvature is antisymmetric",
        lower(omega_xy + omega_yx)
    );
    println!(
        "  Omega_xx               {:>12.3e}   so the diagonal of it vanishes",
        lower(omega_xx)
    );
    println!();
    println!("  Those three are checks rather than results. A Hermitian tensor has a real");
    println!("  symmetric part and an imaginary antisymmetric one, so a non-zero Omega_xx would");
    println!("  be reporting an arithmetic error and not a discovery.\n");
}

/// The transport the geometry forces, against the transport the band alone would give.
pub fn print_transport(
    trace: FloatType,
    g_xx: FloatType,
    reduced: FloatType,
    gap: FloatType,
    conventional: FloatType,
    geometric: FloatType,
) {
    println!("From geometry to transport");
    println!("  tr g = g_xx + g_yy        {:>12.6} nm^2", lower(trace));
    println!("  g_xx                      {:>12.6} nm^2", lower(g_xx));
    println!(
        "  g~_xx = g_xx / a^2        {:>12.6}        dimensionless",
        lower(reduced)
    );
    println!("  E_gap                     {:>12.6} meV", lower(gap));
    println!();
    println!("  D = (D_conv + g~_xx * E_gap) * a^2");
    println!(
        "    flat band alone         {:>12.6} meV*nm^2",
        lower(conventional)
    );
    println!(
        "    with the geometry       {:>12.6} meV*nm^2",
        lower(geometric)
    );
    println!();
    println!("  The flat band has no dispersion, so its conventional weight is zero and it");
    println!("  should not conduct. Every bit of the second number came from the quantum metric");
    println!("  computed above. That is the geometric lower bound, and it is why magic-angle");
    println!("  twisted bilayer graphene is metallic where band theory alone says it cannot be.");
}
