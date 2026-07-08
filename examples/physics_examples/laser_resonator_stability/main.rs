/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Laser Resonator Stability Analysis
//!
//! Propagates a Gaussian beam through an optical cavity using ABCD matrices. Each
//! optical element is a named stage; the round trip composes into one `CausalFlow`
//! pipeline that threads the complex beam parameter q and fails the flow if the beam
//! becomes unphysical (Im(q) <= 0).

use deep_causality_algebra::DivisionAlgebra;
use deep_causality_core::{CausalEffect, CausalFlow, PropagatingEffect, PropagatingProcess};
use deep_causality_num_complex::Complex;
use deep_causality_physics::{
    AbcdMatrix, ComplexBeamParameter, IndexOfRefraction, PhysicsError, Wavelength, beam_spot_size,
    gaussian_q_propagation, lens_maker,
};
use deep_causality_tensor::CausalTensor;

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision.
pub type FloatType = f64;

// Cavity configuration.
const L1: f64 = 0.5; // m, first drift
const L2: f64 = 0.5; // m, second drift
const R_LENS: f64 = 0.5; // m, thermal-lens radius of curvature

/// YAG laser wavelength (1064 nm).
fn laser_wavelength() -> Wavelength<FloatType> {
    Wavelength::<FloatType>::new(1064e-9).unwrap()
}
/// Thermal-lens index of refraction.
fn n_lens() -> IndexOfRefraction<FloatType> {
    IndexOfRefraction::new(1.5).unwrap()
}

fn main() -> Result<(), PhysicsError> {
    println!("=== Laser Resonator Stability Analysis ===\n");

    let wavelength = laser_wavelength();
    println!("Wavelength: {:.1} nm", wavelength.value() * 1e9);

    // Initial Beam: Waist w0 = 1mm at z=0 (Plane wavefront R=inf)
    // q = z + i zR. At waist z=0 -> q = i zR; zR = pi w0^2 / lambda.
    let w0 = 1e-3;
    let z_r = std::f64::consts::PI * w0 * w0 / wavelength.value();
    let q_initial = ComplexBeamParameter::new(Complex::new(0.0, z_r))?;

    println!("Initial Beam: w0 = {:.2} mm, zR = {:.2} m", w0 * 1e3, z_r);

    // Round trip in the cavity as one CausalFlow pipeline: the beam parameter q seeds
    // the value channel, then each optical element binds the propagated q.
    let process = CausalFlow::value(q_initial)
        .bind(stage_drift_l1)
        .bind(stage_thermal_lens)
        .bind(stage_drift_l2)
        .bind(stage_mirror)
        .into_process();

    if let Some(final_q) = process.value() {
        println!("\n=== Final State ===");
        report_beam(*final_q, wavelength, "Round Trip Half-Way");
    } else {
        println!("\n=== Simulation Failed (Unstable) ===");
    }

    Ok(())
}

/// Step 1: drift through free space L1. ABCD = [1, L; 0, 1].
fn stage_drift_l1(
    value: CausalEffect<ComplexBeamParameter<FloatType>>,
    _state: (),
    _ctx: Option<()>,
) -> PropagatingProcess<ComplexBeamParameter<FloatType>, (), ()> {
    let q = value.into_value().unwrap();
    let mat_data = vec![1.0, L1, 0.0, 1.0];
    let mat = AbcdMatrix::new(CausalTensor::new(mat_data, vec![2, 2]).unwrap());

    println!("\n[1] Propagating Drift L1 ({} m)...", L1);
    let next_q_eff = gaussian_q_propagation(q, &mat);
    report_beam(
        next_q_eff.value_cloned().unwrap(),
        laser_wavelength(),
        "After Drift L1",
    );

    PropagatingEffect::pure(next_q_eff.value_cloned().unwrap())
}

/// Step 2: thermal lens. Focal length from the lens-maker equation; ABCD = [1, 0; -1/f, 1].
fn stage_thermal_lens(
    value: CausalEffect<ComplexBeamParameter<FloatType>>,
    _state: (),
    _ctx: Option<()>,
) -> PropagatingProcess<ComplexBeamParameter<FloatType>, (), ()> {
    let q = value.into_value().unwrap();

    let power_eff = lens_maker(n_lens(), R_LENS, -R_LENS); // Biconvex
    let power = power_eff.value_cloned().unwrap().value();
    let f = 1.0 / power;

    println!("\n[2] Transmitting Lens (f = {:.2} m)...", f);

    let mat_data = vec![1.0, 0.0, -1.0 / f, 1.0];
    let mat = AbcdMatrix::new(CausalTensor::new(mat_data, vec![2, 2]).unwrap());

    let next_q_eff = gaussian_q_propagation(q, &mat);
    report_beam(
        next_q_eff.value_cloned().unwrap(),
        laser_wavelength(),
        "After Lens",
    );

    PropagatingEffect::pure(next_q_eff.value_cloned().unwrap())
}

/// Step 3: drift through free space L2. ABCD = [1, L; 0, 1].
fn stage_drift_l2(
    value: CausalEffect<ComplexBeamParameter<FloatType>>,
    _state: (),
    _ctx: Option<()>,
) -> PropagatingProcess<ComplexBeamParameter<FloatType>, (), ()> {
    let q = value.into_value().unwrap();
    let mat_data = vec![1.0, L2, 0.0, 1.0];
    let mat = AbcdMatrix::new(CausalTensor::new(mat_data, vec![2, 2]).unwrap());

    println!("\n[3] Propagating Drift L2 ({} m)...", L2);
    let next_q_eff = gaussian_q_propagation(q, &mat);
    report_beam(
        next_q_eff.value_cloned().unwrap(),
        laser_wavelength(),
        "At Mirror",
    );

    PropagatingEffect::pure(next_q_eff.value_cloned().unwrap())
}

/// Step 4: reflection off a flat mirror. The beam is confined iff Im(q) > 0; otherwise the
/// flow enters the error channel as an unstable resonator.
fn stage_mirror(
    value: CausalEffect<ComplexBeamParameter<FloatType>>,
    _state: (),
    _ctx: Option<()>,
) -> PropagatingProcess<ComplexBeamParameter<FloatType>, (), ()> {
    let q = value.into_value().unwrap();
    if q.value().im > 0.0 {
        println!("\n[Status] Beam is CONFINED (Im(q) > 0).");
        PropagatingEffect::pure(q)
    } else {
        println!("\n[Status] Beam UNSTABLE (Diffracted away).");
        PropagatingEffect::from_error(deep_causality_core::CausalityError::new(
            deep_causality_core::CausalityErrorEnum::Custom("Unstable Resonator".into()),
        ))
    }
}

fn report_beam(q: ComplexBeamParameter<FloatType>, lambda: Wavelength<FloatType>, label: &str) {
    let w_eff = beam_spot_size(q, lambda);
    if let Some(w) = w_eff.value() {
        println!(
            "    {}: Spot Size w = {:.3} mm, Curvature R = {:.2} m",
            label,
            w.value() * 1e3,
            q.value().norm_sqr() / q.value().re
        );
    }
}
