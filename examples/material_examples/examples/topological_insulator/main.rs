/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use model::QWZModel;
use std::f64::consts::PI;

mod model;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("----------------------------------------------------------------");
    println!("   Topological Insulator Analysis (Berry Curvature)");
    println!("----------------------------------------------------------------");
    println!("Calculating Chern Number for Hamiltonian Manifold.\n");

    let n = 100; // Grid resolution (higher = more accurate)
    let dk = 2.0 * PI / (n as f64);

    // Test both phases
    let test_params = vec![
        (3.0, "TRIVIAL (u > 2)"),
        (1.0, "TOPOLOGICAL (-2 < u < 0 or 0 < u < 2)"),
        (-1.0, "TOPOLOGICAL (-2 < u < 0 or 0 < u < 2)"),
    ];

    for (u, expected) in test_params {
        println!(
            "\n[...] Analyzing Material: u = {:.1} (Expected: {})",
            u, expected
        );
        let model = QWZModel::new(u);
        let mut total_flux = 0.0;

        // Iterate over discretized Brillouin Zone [-pi, pi] x [-pi, pi]
        for i in 0..n {
            for j in 0..n {
                let kx = -PI + (i as f64) * dk;
                let ky = -PI + (j as f64) * dk;

                // Get spinors at 4 corners of plaquette
                let psi_00 = model.lower_band_spinor(kx, ky);
                let psi_10 = model.lower_band_spinor(kx + dk, ky);
                let psi_11 = model.lower_band_spinor(kx + dk, ky + dk);
                let psi_01 = model.lower_band_spinor(kx, ky + dk);

                // Link variables around the plaquette (counterclockwise)
                // U_x(k) = <psi(k)|psi(k+x)>
                let u_01_00 = model::overlap(psi_00, psi_10); // right
                let u_02_01 = model::overlap(psi_10, psi_11); // up
                let u_03_02 = model::overlap(psi_11, psi_01); // left
                let u_00_03 = model::overlap(psi_01, psi_00); // down

                // Wilson loop = U1 * U2 * U3 * U4
                let wilson = u_01_00 * u_02_01 * u_03_02 * u_00_03;

                // Berry flux = Im(ln(Wilson))
                // Using atan2 for proper branch handling
                let flux = wilson.im.atan2(wilson.re);
                total_flux += flux;
            }
        }

        let chern = total_flux / (2.0 * PI);
        println!("      Total Berry Flux: {:.4} rad", total_flux);
        println!("      Chern Number: {:.2}", chern);

        let c_rounded = chern.round() as i32;
        if c_rounded == 0 {
            println!("      Verdict: TRIVIAL Insulator (C = 0)");
        } else {
            println!("      Verdict: TOPOLOGICAL Insulator (C = {})", c_rounded);
            println!("      -> Protected edge states exist!");
        }
    }

    println!("\n----------------------------------------------------------------");
    println!("   Phase Diagram Summary");
    println!("----------------------------------------------------------------");
    println!("   |u| > 2  : Trivial (C = 0)");
    println!("   0 < u < 2: Topological (C = -1)");
    println!("  -2 < u < 0: Topological (C = +1)");

    Ok(())
}
