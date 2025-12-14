/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Laser Resonator Stability Analysis
//!
//! Propagates a Gaussian beam through an optical cavity using ABCD matrices and Causal Monads.
//! Checks stability by verifying if the beam parameter q remains physical (Im(q) > 0).

use deep_causality_core::{CausalEffectPropagationProcess, EffectValue};
use deep_causality_num::{Complex, DivisionAlgebra};
use deep_causality_physics::{
    AbcdMatrix, ComplexBeamParameter, IndexOfRefraction, PhysicsError, Wavelength, beam_spot_size,
    gaussian_q_propagation, lens_maker,
};
use deep_causality_tensor::CausalTensor;

fn main() -> Result<(), PhysicsError> {
    println!("=== Laser Resonator Stability Analysis ===\n");

    let wavelength = Wavelength::new(1064e-9)?; // YAG laser
    println!("Wavelength: {:.1} nm", wavelength.value() * 1e9);

    // Initial Beam: Waist w0 = 1mm at z=0 (Plane wavefront R=inf)
    // q = z + i zR. At waist z=0 -> q = i zR.
    // zR = pi w0^2 / lambda
    let w0 = 1e-3;
    let z_r = std::f64::consts::PI * w0 * w0 / wavelength.value();
    let q_initial = ComplexBeamParameter::new(Complex::new(0.0, z_r))?;

    println!("Initial Beam: w0 = {:.2} mm, zR = {:.2} m", w0 * 1e3, z_r);

    // Causal Chain: Round Trip in Cavity
    // Start -> Drift L1 -> Lens(f) -> Drift L2 -> Mirror -> Drift L2 -> Lens(f) -> Drift L1 -> End

    // Cavity Config
    let l1 = 0.5; // m
    let l2 = 0.5; // m
    // Thermal Lens focal length (dynamic)
    let r_lens = 0.5; // m (Radius of curvature)
    let n_lens = IndexOfRefraction::new(1.5)?;

    // We execute the propagation steps monadically
    let process = CausalEffectPropagationProcess::with_state(
        CausalEffectPropagationProcess::pure(q_initial),
        (),
        Some(wavelength),
    )
    .bind(|q_eff, _, _| {
        // --- Step 1: Drift L1 ---
        let q = q_eff.into_value().unwrap();
        // ABCD for Free Space: [1, L; 0, 1]
        let mat_data = vec![1.0, l1, 0.0, 1.0];
        let mat = AbcdMatrix::new(CausalTensor::new(mat_data, vec![2, 2]).unwrap());

        println!("\n[1] Propagating Drift L1 ({} m)...", l1);
        let next_q_eff = gaussian_q_propagation(q, &mat);
        report_beam(
            next_q_eff.value().clone().into_value().unwrap(),
            wavelength,
            "After Drift L1",
        );

        // Pass forward
        CausalEffectPropagationProcess::pure(next_q_eff.value().clone().into_value().unwrap())
    })
    .bind(|q_eff, _, _| {
        // --- Step 2: Thermal Lens ---
        let q = q_eff.into_value().unwrap();

        // Calculate focal length via Lens Maker
        let power_eff = lens_maker(n_lens, r_lens, -r_lens); // Biconvex
        let power = power_eff.value().clone().into_value().unwrap().value();
        let f = 1.0 / power;

        println!("\n[2] Transmitting Lens (f = {:.2} m)...", f);

        // ABCD for Thin Lens: [1, 0; -1/f, 1]
        let mat_data = vec![1.0, 0.0, -1.0 / f, 1.0];
        let mat = AbcdMatrix::new(CausalTensor::new(mat_data, vec![2, 2]).unwrap());

        let next_q_eff = gaussian_q_propagation(q, &mat);
        report_beam(
            next_q_eff.value().clone().into_value().unwrap(),
            wavelength,
            "After Lens",
        );

        CausalEffectPropagationProcess::pure(next_q_eff.value().clone().into_value().unwrap())
    })
    .bind(|q_eff, _, _| {
        // --- Step 3: Drift L2 ---
        let q = q_eff.into_value().unwrap();
        let mat_data = vec![1.0, l2, 0.0, 1.0];
        let mat = AbcdMatrix::new(CausalTensor::new(mat_data, vec![2, 2]).unwrap());

        println!("\n[3] Propagating Drift L2 ({} m)...", l2);
        let next_q_eff = gaussian_q_propagation(q, &mat);
        report_beam(
            next_q_eff.value().clone().into_value().unwrap(),
            wavelength,
            "At Mirror",
        );

        CausalEffectPropagationProcess::pure(next_q_eff.value().clone().into_value().unwrap())
    })
    .bind(|q_eff, _, _| {
        // --- Step 4: Reflection (Flat Mirror) ---
        let q = q_eff.into_value().unwrap();
        if q.value().im > 0.0 {
            println!("\n[Status] Beam is CONFINED (Im(q) > 0).");
            CausalEffectPropagationProcess::pure(q)
        } else {
            println!("\n[Status] Beam UNSTABLE (Diffracted away).");
            CausalEffectPropagationProcess::from_error(deep_causality_core::CausalityError::new(
                deep_causality_core::CausalityErrorEnum::Custom("Unstable Resonator".into()),
            ))
        }
    });

    if let EffectValue::Value(final_q) = process.value() {
        println!("\n=== Final State ===");
        report_beam(*final_q, wavelength, "Round Trip Half-Way");
    } else {
        println!("\n=== Simulation Failed (Unstable) ===");
    }

    Ok(())
}

fn report_beam(q: ComplexBeamParameter, lambda: Wavelength, label: &str) {
    let w_eff = beam_spot_size(q, lambda);
    if let EffectValue::Value(w) = w_eff.value() {
        println!(
            "    {}: Spot Size w = {:.3} mm, Curvature R = {:.2} m",
            label,
            w.value() * 1e3,
            q.value().norm_sqr() / q.value().re
        );
    }
}
