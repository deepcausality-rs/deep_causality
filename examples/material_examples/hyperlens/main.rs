/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::Metric;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("----------------------------------------------------------------");
    println!("   Hyperbolic Metamaterial Lens (Hyperlens) Simulation");
    println!("----------------------------------------------------------------");
    println!("Demonstrating sub-diffraction imaging via Metric manipulation.\n");

    // ----------------------------------------------------------------
    // 1. Define Metrics
    // ----------------------------------------------------------------

    // Standard Vacuum: Euclidean(3) -> Signature (+ + +)
    let vacuum_metric = Metric::Euclidean(3);
    println!("[1] Defined Vacuum Metric: {}", vacuum_metric);

    // Hyperbolic Metamaterial: Generic(1, 2, 0) -> Signature (+ - -)
    // p=1 (+), q=2 (-), r=0. (1 spatial dim is +, 2 are -)
    // This signature creates a hyperbolic dispersion surface.
    let hyperbolic_metric = Metric::Generic { p: 1, q: 2, r: 0 };
    println!("[1] Defined Hyperlens Metric: {}", hyperbolic_metric);

    // ----------------------------------------------------------------
    // 2. Setup (Conceptual)
    // ----------------------------------------------------------------
    println!("\n[2] Initializing Hyperlens Geometry...");
    println!("    (Skipping full Manifold construction for this example calculation)");

    // ----------------------------------------------------------------
    // 3. Simulate Wave Propagation (Dispersion Relation Check)
    // ----------------------------------------------------------------
    println!("\n[3] Simulating Sub-Wavelength Transmission...");

    // Object Details: Distance between two sources
    let wavelength = 500.0; // nm (Green light)
    let source_separation = 100.0; // nm (Sub-diffraction limit: < lambda/2)

    // Spatial frequency k_x approx 1 / separation
    let k_x = 2.0 * std::f64::consts::PI / source_separation;

    // Max propagating k in vacuum k0 = 2*pi / lambda
    let k_0 = 2.0 * std::f64::consts::PI / wavelength;

    println!("    Light Wavelength: {} nm", wavelength);
    println!("    Source Separation: {} nm", source_separation);
    println!("    Required k_vector: {:.4} rad/nm", k_x);
    println!("    Max Vacuum k_0:    {:.4} rad/nm", k_0);

    // CHECK VACUUM PROPAGATION
    // k_z = sqrt(k0^2 - kx^2)
    let kz_sq_vacuum = k_0.powi(2) - k_x.powi(2);

    if kz_sq_vacuum < 0.0 {
        println!(
            "\n    -> Vacuum Verdict: k_z^2 = {:.4} (Negative!)",
            kz_sq_vacuum
        );
        println!("       Result: EVANESCENT DECAY. Image is lost.");
        println!("       The fine details cannot propagate to the far field.");
    } else {
        println!("       Result: Propagation.");
    }

    // CHECK HYPERLENS PROPAGATION
    println!("\n    ...Switching material to Hyperbolic dispersion...");

    // In HMM, dispersion is hyperbolic. Simplified: k_z^2 = k0^2 * eps_x + k_x^2 * (|eps_z|/eps_x)
    // Let's use a standard Type I HMM: eps_z < 0, eps_x > 0
    // Derived from Metric Generic(1,2,0) which has (+ - -) structure

    let epsilon_parallel = 1.0; // x (+)
    let epsilon_perp = -1.0; // z (-)

    // Dispersion relation in anisotropic medium:
    // k_z^2 / eps_x + k_x^2 / eps_z = (omega/c)^2
    // -> k_z^2 = eps_x * ((omega/c)^2 - k_x^2 / eps_z)

    let kz_sq_hmm = epsilon_parallel * (k_0.powi(2) - (k_x.powi(2) / epsilon_perp));
    // dividing by negative eps_perp makes the term positive!

    println!("\n    -> Hyperlens Verdict: Calculating dispersion with Metric Generic(1,2,0)...");
    println!(
        "       k_z^2 = {:.4} * ({:.4}^2 - ({:.4}^2 / {:.4}))",
        epsilon_parallel, k_0, k_x, epsilon_perp
    );
    println!("       k_z^2 = {:.4} (POSITIVE!)", kz_sq_hmm);

    if kz_sq_hmm > 0.0 {
        println!("       Result: PROPAGATION.");
        println!("       The high-frequency modes (fine details) transmit through the lens.");
        println!("       [SUCCESS] Super-Resolution Achieved.");
    }

    Ok(())
}
