/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, CoMonad, Functor, Monad};
use deep_causality_tensor::CausalTensor;
use deep_causality_tensor::CausalTensorWitness;

fn main() {
    print_header();

    // Constants
    // kappa = 8 * pi * G / c^4
    let kappa = 8.0 * std::f64::consts::PI;
    let lambda = 1e-5;

    println!("Constants:");
    println!("  Kappa (κ)  : {:.4}", kappa);
    println!("  Lambda (Λ) : {:.4}", lambda);
    println!("--------------------------------------------------");

    // 1. Define the Metric Tensor g_uv (4x4)
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
    let scalar_curvature_val = 0.05;

    // 4. Calculate Einstein Tensor G_uv = R_uv - 0.5 * R * g_uv
    // We demonstrate Applicative for scaling and addition?
    // Scaling is Functor.
    // Addition is Applicative (combining two contexts).

    println!("Calculating Einstein Tensor G_uv...");
    println!("Formula: G_uv = R_uv + (-0.5 * R * g_uv)");

    let scaling_factor = -0.5 * scalar_curvature_val;
    let term_2 = <CausalTensorWitness as Functor<CausalTensorWitness>>::fmap(g_uv.clone(), |x| {
        x * scaling_factor
    });

    // G_uv = R_uv + term_2
    // HKT Applicative: lifting addition over two tensors.
    // We curried addition: add_func = fmap(r_uv, |a| move |b| a + b)
    // result = apply(add_func, term_2)
    let add_func_tensor =
        <CausalTensorWitness as Functor<CausalTensorWitness>>::fmap(r_uv.clone(), |a: f64| {
            Box::new(move |b: f64| a + b) as Box<dyn Fn(f64) -> f64>
        });
    let g_tensor =
        <CausalTensorWitness as Applicative<CausalTensorWitness>>::apply(add_func_tensor, term_2);

    print_tensor("Einstein Tensor (G_uv)", &g_tensor);

    // 5. Add Cosmological Term: + Lambda * g_uv
    let lambda_term =
        <CausalTensorWitness as Functor<CausalTensorWitness>>::fmap(g_uv.clone(), |x| x * lambda);

    // LHS = G_uv + Lambda * g_uv
    // Using standard addition for simplicity here, mixing styles to show interoperability.
    let lhs = &g_tensor + &lambda_term;

    print_tensor("LHS (G_uv + Λ * g_uv)", &lhs);

    // 6. Calculate Stress-Energy Tensor T_uv
    let t_uv = <CausalTensorWitness as Functor<CausalTensorWitness>>::fmap(lhs, |x| x / kappa);

    print_tensor("Stress-Energy Tensor (T_uv) Result", &t_uv);

    // 7. Advanced HKT: CoMonad (Local Analysis)
    println!("--------------------------------------------------");
    println!("7. Advanced HKT: CoMonad for Local Field Analysis");
    // ... extend logic ...
    let anomaly_map = <CausalTensorWitness as CoMonad<CausalTensorWitness>>::extend(
        &t_uv,
        |view: &CausalTensor<f64>| {
            let data = view.data();
            let center = data[0];
            let neighbor = if data.len() > 1 { data[1] } else { center };
            (center - neighbor).abs()
        },
    );

    print_tensor("Anomaly Map (Local Gradients)", &anomaly_map);

    // 8. Advanced HKT: Monad (Quantization / Expansion)
    println!("--------------------------------------------------");
    println!("8. Advanced HKT: Monad for Quantum Fluctuations");
    println!("   Monad `bind` allows us to replace each value with a new structure (sub-tensor)");
    println!("   and flatten the result. We use this to simulate splitting energy levels.");

    // Function: "Split each energy value E into [E - delta, E, E + delta]"
    let fluctuation_fn = |energy: f64| {
        let delta = energy * 0.1; // 10% fluctuation
        // Return a tensor of 3 values
        CausalTensor::new(vec![energy - delta, energy, energy + delta], vec![3]).unwrap()
    };

    // Bind: Apply fluctuation to every element of T_uv (flattened)
    // For a 4x4 tensor (16 elements), this produces 16 * 3 = 48 elements.
    let quantum_foam =
        <CausalTensorWitness as Monad<CausalTensorWitness>>::bind(t_uv.clone(), fluctuation_fn);

    println!("Original Elements: {}", t_uv.len());
    println!("Quantum Foam Elements: {}", quantum_foam.len());
    println!("First few fluctuations:");
    let foam_data = quantum_foam.data();
    for val in foam_data.iter().take(6) {
        print!("{:.4} ", val);
    }
    println!("...");
}

pub(crate) fn print_header() {
    println!("============================================================");
    println!("   Einstein Field Equations with CausalTensor HKT");
    println!("============================================================");
    println!("Demonstrating Functor, Applicative, Monad, and CoMonad.");
    println!("============================================================");
    println!();
}

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
