/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{CoMonad, Functor};
use deep_causality_tensor::CausalTensor;
use deep_causality_tensor::CausalTensorWitness;

fn main() {
    print_header();

    // Constants
    // kappa = 8 * pi * G / c^4
    // For simplicity in this example, we use normalized units where G=1, c=1.
    let kappa = 8.0 * std::f64::consts::PI;
    // Cosmological constant (small positive value for accelerating expansion)
    let lambda = 1e-5;

    println!("Constants:");
    println!("  Kappa (κ)  : {:.4}", kappa);
    println!("  Lambda (Λ) : {:.4}", lambda);
    println!("--------------------------------------------------");

    // 1. Define the Metric Tensor g_uv (4x4)
    // We'll use a Minkowski metric signature (- + + +) as a base,
    // slightly perturbed to make it interesting (not just flat space).
    let metric_data = vec![
        -1.0, 0.01, 0.0, 0.0, // t
        0.01, 1.0, 0.0, 0.0, // x
        0.0, 0.0, 1.0, 0.0, // y
        0.0, 0.0, 0.0, 1.0, // z
    ];
    let metric_shape = vec![4, 4];
    let g_uv =
        CausalTensor::new(metric_data, metric_shape).expect("Failed to create metric tensor");

    print_tensor("Metric Tensor (g_uv)", &g_uv);

    // 2. Define the Ricci Tensor R_uv (4x4)
    // The Ricci tensor represents the amount by which the volume of a geodesic ball
    // deviates from that in Euclidean space. Calculating it from the metric involves
    // complex derivatives (Christoffel symbols). Here we assume it's given.
    let ricci_data = vec![
        0.1, 0.005, 0.0, 0.0, // t
        0.005, 0.05, 0.0, 0.0, // x
        0.0, 0.0, 0.05, 0.0, // y
        0.0, 0.0, 0.0, 0.05, // z
    ];
    let ricci_shape = vec![4, 4];
    let r_uv = CausalTensor::new(ricci_data, ricci_shape).expect("Failed to create Ricci tensor");

    print_tensor("Ricci Tensor (R_uv)", &r_uv);

    // 3. Define Scalar Curvature R (Scalar)
    // Trace of Ricci tensor w.r.t metric (R = g^uv R_uv).
    // For this example, we just pick a value consistent with the tensors above.
    let scalar_curvature_val = 0.05;

    println!("Scalar Curvature (R): {:.4}", scalar_curvature_val);
    println!("--------------------------------------------------");

    // 4. Calculate Einstein Tensor G_uv = R_uv - 0.5 * R * g_uv
    // We use HKT Functor to scale g_uv.

    println!("Calculating Einstein Tensor G_uv...");
    println!("Formula: G_uv = R_uv + (-0.5 * R * g_uv)");

    // Calculate scaling factor: -0.5 * R
    let scaling_factor = -0.5 * scalar_curvature_val;

    // Scale g_uv by this factor using HKT Functor
    // HKT Power: `fmap` transforms the structure without us writing a loop.
    let term_2 = <CausalTensorWitness as Functor<CausalTensorWitness>>::fmap(g_uv.clone(), |x| {
        x * scaling_factor
    });

    // G_uv = R_uv + term_2
    // We use the standard `Add` implementation for tensor-tensor addition.
    let g_tensor = &r_uv + &term_2;

    print_tensor("Einstein Tensor (G_uv)", &g_tensor);

    // 5. Add Cosmological Term: + Lambda * g_uv
    println!("Adding Cosmological Term...");
    println!("Formula: LHS = G_uv + (Λ * g_uv)");

    // Use HKT Functor to scale g_uv by Lambda
    let lambda_term =
        <CausalTensorWitness as Functor<CausalTensorWitness>>::fmap(g_uv.clone(), |x| x * lambda);

    // LHS = G_uv + Lambda * g_uv
    let lhs = &g_tensor + &lambda_term;

    print_tensor("LHS (G_uv + Λ * g_uv)", &lhs);

    // 6. Calculate Stress-Energy Tensor T_uv
    // LHS = kappa * T_uv  =>  T_uv = LHS / kappa
    println!("Solving for Stress-Energy Tensor T_uv...");
    println!("Formula: T_uv = LHS / κ");

    // HKT Power: `fmap` again allows us to simply divide every element by kappa.
    let t_uv = <CausalTensorWitness as Functor<CausalTensorWitness>>::fmap(lhs, |x| x / kappa);

    println!("--------------------------------------------------");
    print_tensor("Stress-Energy Tensor (T_uv) Result", &t_uv);

    println!("--------------------------------------------------");
    println!("Verification:");
    println!("Shape of T_uv: {:?}", t_uv.shape());
    assert_eq!(t_uv.shape(), &[4, 4]);
    println!("Calculation completed successfully.");

    // 7. Advanced HKT: CoMonad
    println!("--------------------------------------------------");
    println!("7. Advanced HKT: CoMonad for Local Field Analysis");
    println!("   CoMonad `extend` allows us to perform operations where each element's");
    println!("   new value depends on its 'neighborhood' (context).");
    println!(
        "   This is ideal for calculating field gradients, smoothness, or detecting anomalies."
    );

    // Example: Detect "High Energy" regions relative to neighbors.
    // We'll define a "smoothness" check: abs(center - neighbor).
    // Since the tensor is flattened, we check the immediate neighbor in memory for simplicity.
    let anomaly_map = <CausalTensorWitness as CoMonad<CausalTensorWitness>>::extend(
        &t_uv,
        |view: &CausalTensor<f64>| {
            let data = view.data();
            let center = data[0];
            // Check next neighbor (wrapping)
            let neighbor = if data.len() > 1 { data[1] } else { center };
            // Calculate gradient/difference
            (center - neighbor).abs()
        },
    );

    print_tensor("Anomaly Map (Local Gradients)", &anomaly_map);
}

pub(crate) fn print_header() {
    println!("============================================================");
    println!("   Einstein Field Equations with CausalTensor & HKT");
    println!("============================================================");
    println!("This example demonstrates solving the Einstein Field Equations (EFE):");
    println!("  G_uv + Λ * g_uv = κ * T_uv");
    println!();
    println!("Where:");
    println!("  G_uv = R_uv - 0.5 * R * g_uv  (Einstein Tensor)");
    println!("  R_uv                          (Ricci Curvature Tensor)");
    println!("  R                             (Scalar Curvature)");
    println!("  g_uv                          (Metric Tensor)");
    println!("  Λ                             (Cosmological Constant)");
    println!("  κ                             (Einstein Constant)");
    println!("  T_uv                          (Stress-Energy Tensor)");
    println!();
    println!("Value of HKT (Higher-Kinded Types) in this context:");
    println!("1. Abstraction: We treat tensors as abstract contexts (Functors).");
    println!("2. Safety: Operations are lifted into the context, handling shapes implicitly.");
    println!("3. Composability: We chain operations (map) without manual loops.");
    println!("4. Clarity: The code mirrors the mathematical equation structure.");
    println!("============================================================");
    println!();
}

/// Helper function to pretty print a 2D tensor
pub(crate) fn print_tensor(name: &str, tensor: &CausalTensor<f64>) {
    println!("{}:", name);
    let shape = tensor.shape();
    if shape.len() != 2 {
        println!("{:?}", tensor);
        return;
    }
    let rows = shape[0];
    let cols = shape[1];
    let data = tensor.data();

    println!("[");
    for i in 0..rows {
        print!("  [");
        for j in 0..cols {
            let val = data[i * cols + j];
            if val.abs() < 1e-10 {
                print!("{:>10.4}", 0.0);
            } else {
                print!("{:>10.4}", val);
            }
            if j < cols - 1 {
                print!(", ");
            }
        }
        println!("  ],");
    }
    println!("]");
    println!();
}
