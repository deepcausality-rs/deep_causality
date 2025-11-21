/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::Applicative;
use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

// --- THE COUPLING ---

fn main() {
    println!("============================================================");
    println!("   GRMHD: General Relativistic Magnetohydrodynamics");
    println!("============================================================");

    // Step 1: The GR Solver (Tensor Monad)
    println!("\n[Step 1] GR Solver: Calculating Spacetime Curvature...");
    let g_uv = calculate_spacetime_metric();
    let g_tensor = calculate_einstein_tensor(&g_uv);

    // Extract a local metric property to feed into MHD.
    // For example, the time component g_00 tells us about gravitational time dilation.
    // A "proper" coupling would map the full tensor to the Clifford metric.
    let time_dilation = g_tensor.data()[0].abs();
    println!("   -> Local Curvature Intensity: {:.4}", time_dilation);

    // Step 2: The Coupling Logic (Causal Glue)
    // We determine the effective metric signature for the MHD solver based on GR.
    // If curvature is high, we must use Minkowski(4). If flat, Euclidean(3).
    // This is a dynamic type/value decision driven by the physics.
    println!("\n[Step 2] Causal Coupling: Configuring MHD Solver...");
    let (metric_sig, label) = if time_dilation > 0.05 {
        (Metric::Minkowski(4), "Relativistic (Minkowski 4D)")
    } else {
        (Metric::Euclidean(3), "Classical (Euclidean 3D)")
    };
    println!("   -> Selected Metric: {}", label);

    // Step 3: The MHD Solver (MultiVector Monad)
    println!("\n[Step 3] MHD Solver: Calculating Plasma Confinement...");
    let current_j = 10.0;
    let field_b = 2.0;

    // The physics calculation happens here, using the metric decided by Step 2.
    let force = calculate_lorentz_force(current_j, field_b, metric_sig);

    println!("   -> Lorentz Force Density: {:.4}", force);

    // Step 4: Feedback Loop (The "Cycle")
    println!("\n[Step 4] Analysis:");
    if force < 0.0 {
        println!("   STATUS: Relativistic Reversal Detected!");
        println!("   Action: Adjusting containment field to compensate for frame dragging.");
    } else {
        println!("   STATUS: Standard Confinement.");
    }

    println!("\n============================================================");
    println!("CONCLUSION:");
    println!("We successfully coupled a Tensor-based GR solver with a");
    println!("MultiVector-based MHD solver in a single executable.");
    println!("Data flowed from Spacetime Geometry -> Coupling -> Plasma Physics.");
    println!("============================================================");
}

// --- GENERAL RELATIVITY (Tensor Engine) ---

/// Calculates the Metric Tensor for a simplified Schwarzschild-like spacetime.
/// In a real simulation, this would be dynamic.
fn calculate_spacetime_metric() -> CausalTensor<f64> {
    // Minkowski metric signature (- + + +) perturbed by gravity
    // g_00 = -(1 - 2GM/rc^2)
    // For simulation, we use normalized units.
    let g_00 = -0.9; // Time dilation
    let g_11 = 1.1; // Radial stretching
    let g_22 = 1.0;
    let g_33 = 1.0;

    let metric_data = vec![
        g_00, 0.0, 0.0, 0.0, 0.0, g_11, 0.0, 0.0, 0.0, 0.0, g_22, 0.0, 0.0, 0.0, 0.0, g_33,
    ];
    CausalTensor::new(metric_data, vec![4, 4]).unwrap()
}

/// Calculates the Einstein Tensor G_uv given a metric.
/// (Simplified for demonstration: G_uv ~ curvature * metric)
fn calculate_einstein_tensor(g_uv: &CausalTensor<f64>) -> CausalTensor<f64> {
    // Assume a scalar curvature R driven by mass
    let curvature = 0.1;

    // G_uv ~ R * g_uv (Simplified EFE LHS)
    // Use HKT to scale the metric
    let scale_fn = |x: f64| x * curvature;
    let fn_tensor = <CausalTensorWitness as Applicative<CausalTensorWitness>>::pure(scale_fn);

    <CausalTensorWitness as Applicative<CausalTensorWitness>>::apply(fn_tensor, g_uv.clone())
}

// --- MAGNETOHYDRODYNAMICS (MultiVector Engine) ---

/// Calculates the Lorentz Force Density F = J . B
/// Adapted for the local spacetime metric.
fn calculate_lorentz_force(
    current_density: f64,
    magnetic_field: f64,
    metric_signature: Metric,
) -> f64 {
    // 1. Setup Plasma Current (J)
    // Current flowing Toroidally (X-axis/Dim 1)
    let idx_current = 1 << 1;
    let mut j_data = vec![0.0; 1 << metric_signature.dimension()];
    j_data[idx_current] = current_density;
    let j_vec = CausalMultiVector::new(j_data, metric_signature).unwrap();

    // 2. Setup Magnetic Field (B)
    // Field applied Poloidally (Y-axis plane/Dim 1^2)
    let idx_field_plane = (1 << 1) | (1 << 2);
    let mut b_data = vec![0.0; 1 << metric_signature.dimension()];
    b_data[idx_field_plane] = magnetic_field;
    let b_field = CausalMultiVector::new(b_data, metric_signature).unwrap();

    // 3. Compute Force: F = J . B
    let force = j_vec.inner_product(&b_field);

    // Extract force component in Poloidal direction (Dim 2)
    let idx_force = 1 << 2;
    *force.get(idx_force).unwrap()
}
