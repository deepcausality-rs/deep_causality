/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Event Horizon Probe
//!
//! A space probe approaching a black hole, switching between Newtonian and relativistic physics
//! through a causal monad. Three DeepCausality pillars appear together:
//!
//! - **The tangent functor.** The local gravitational field is read straight off the potential:
//!   the acceleration is `g = −dΦ/dr` and the tidal gradient is `−d²Φ/dr²`, both produced by
//!   `derivative` / `second_derivative` of `Φ(r) = −GM/r` (no hand-coded `GM/r²`).
//! - **Precision as a parameter.** One `FloatType` alias re-runs the whole simulation, the autodiff
//!   field included, at `f32`, `f64`, or `Float106`; the physics kernels are generic over the
//!   `RealField` scalar.
//! - **The causal monad.** `CausalEffectPropagationProcess` carries the probe state and the black
//!   hole mass through each regime-switching step.

use deep_causality_calculus::{DifferentiableArrow, DifferentiateExt, Scalar};
use deep_causality_core::CausalFlow;
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num::FromPrimitive;
use deep_causality_physics::{Length, Mass, NEWTONIAN_CONSTANT_OF_GRAVITATION, PhysicsError};
use deep_causality_physics::{escape_velocity, schwarzschild_radius, time_dilation_angle};

/// Switch this alias to `f32` for low precision, `f64` for standard precision, or `Float106` for
/// high precision (the latter also needs `use deep_causality_num::Float106;`). The whole
/// simulation, the autodiff gravitational field included, re-runs at the chosen precision.
pub type FloatType = f64;

/// Sagittarius A* (about 4 million solar masses), in kilograms.
const M_KG: FloatType = 4.0e6 * 1.989e30;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Event Horizon Probe Simulation ===\n");

    // 1. Setup: Supermassive Black Hole (Sagittarius A* approx)
    let black_hole_mass =
        Mass::<FloatType>::new(ft(M_KG)).map_err(|e: PhysicsError| e.to_string())?;
    let rs_effect = schwarzschild_radius(&black_hole_mass);
    let r_s = rs_effect.value_cloned().unwrap().value();

    // Φ(r) = −GM/r; the gravitational acceleration and tidal force are its derivatives.
    let potential = NewtonianPotential {
        gm: NEWTONIAN_CONSTANT_OF_GRAVITATION * M_KG,
    };

    println!("Target: Supermassive Black Hole");
    println!("Mass: {:.2e} kg", black_hole_mass.value());
    println!("Schwarzschild Radius (Rs): {:.2e} m\n", r_s);

    // 2. Initial State: Probe far away
    let initial_state = ProbeState {
        distance: r_s * ft(100.0), // 100x Rs
        velocity: ft(0.0),         // Starting from rest (freefall)
        mass: ft(1000.0),          // 1000 kg probe
        status: "Approaching".to_string(),
    };

    // 3. Causal Chain: Fall and Evaluate
    let steps = 5;
    let mut current_state = initial_state;

    for t in 0..steps {
        println!("---\nStep {} ---", t);
        let dist_ratio = current_state.distance / r_s;
        println!(
            "Distance: {:.2e} m ({:.1} Rs)",
            current_state.distance, dist_ratio
        );

        // The gravitational field, straight from the tangent functor: g = −dΦ/dr, tidal = −d²Φ/dr².
        let r = current_state.distance;
        let g = -potential.derivative(r);
        let tidal = -potential.second_derivative(r);
        println!("  [AD] gravity  g = −dΦ/dr    = {:.3e} m/s²", g);
        println!("  [AD] tidal gradient −d²Φ/dr² = {:.3e} 1/s²", tidal);

        // Define the physics context based on state
        let regime_check = if dist_ratio > ft(10.0) {
            "Newtonian"
        } else {
            "Relativistic"
        };
        println!("Physics Regime: {}", regime_check);

        // Run the regime-switching step as a stateful CausalFlow: the probe state is the flow's
        // state and the black-hole mass its context; the step returns the next probe state.
        let next = CausalFlow::process(current_state.clone())
            .context(black_hole_mass)
            .try_step_with(
                |_unit: (), state: &ProbeState, ctx: Option<&Mass<FloatType>>| {
                    let bh_mass = *ctx.expect("context holds the BH mass");
                    let r = Length::<FloatType>::new(state.distance).unwrap();

                    // A. Calculate expected orbital/escape velocities (Context assessment)
                    let v_esc_effect = escape_velocity(&bh_mass, &r);
                    let v_esc = v_esc_effect.value_cloned().unwrap().value();

                    println!("  Escape Velocity required: {:.2e} m/s", v_esc);

                    // Cross-check: the autodiff gravity equals v_esc²/(2r) (both are GM/r²).
                    let g_from_vesc = v_esc * v_esc / (ft(2.0) * state.distance);
                    println!("  [check] v_esc²/(2r)         = {:.3e} m/s²", g_from_vesc);

                    // B. Regime-Specific Logic
                    if state.distance / r_s > ft(10.0) {
                        // --- Newtonian Regime ---
                        // Simple freefall approximation v = sqrt(2GM/r) (which is v_esc)
                        let new_vel = v_esc;
                        let new_dist = state.distance * ft(0.5); // Simulate falling

                        Ok(ProbeState {
                            distance: new_dist,
                            velocity: new_vel,
                            status: "Freefall (Newtonian)".to_string(),
                            mass: state.mass,
                        })
                    } else {
                        // --- Relativistic Regime ---
                        // Calculate Time Dilation effects, metric (+---)
                        let metric = Metric::Minkowski(4);

                        // Probe 4-velocity (approx): a static observer e_t
                        let mut static_vec = vec![ft(0.0); 16];
                        static_vec[1] = ft(1.0);
                        let t_static = CausalMultiVector::new(static_vec, metric).unwrap();

                        // Falling probe vector (gamma, gamma*v, 0, 0)
                        let v_rel = ft(0.9);
                        let gamma = ft(1.0) / fsqrt(ft(1.0) - v_rel * v_rel);
                        let mut probe_vec = vec![ft(0.0); 16];
                        probe_vec[1] = gamma;
                        probe_vec[2] = gamma * v_rel;
                        let t_probe = CausalMultiVector::new(probe_vec, metric).unwrap();

                        let dilation_effect = time_dilation_angle(&t_static, &t_probe);
                        let rapidity = dilation_effect.value_cloned().unwrap().value();

                        println!("  [GR] Relativistic Rapidity: {:.4}", rapidity);
                        println!("  [GR] Time Dilation Factor: {:.2}", fcosh(rapidity));

                        // Check Horizon crossing
                        if state.distance <= r_s * ft(1.1) {
                            Ok(ProbeState {
                                distance: state.distance * ft(0.1),
                                velocity: ft(2.99e8), // c
                                status: "EVENT HORIZON CROSSED".to_string(),
                                mass: state.mass,
                            })
                        } else {
                            Ok(ProbeState {
                                distance: state.distance * ft(0.5),
                                velocity: v_esc,
                                status: "Relativistic Plunge".to_string(),
                                mass: state.mass,
                            })
                        }
                    }
                },
            )
            .finish();

        // Update State
        if let Ok(s) = next {
            current_state = s;
            println!("  -> New Status: {}", current_state.status);
            println!("  -> Current Velocity: {:.2e} m/s", current_state.velocity);

            if current_state.status == "EVENT HORIZON CROSSED" {
                println!("\n!!! SIGNAL LOST !!! Probe has crossed the event horizon.");
                break;
            }
        }
        println!();
    }

    Ok(())
}

/// Lift an exact `f64` constant into the working precision. The fully-qualified call forces the
/// `FromPrimitive` trait method; some scalars (e.g. `Float106`) carry an inherent `from_f64` that
/// would otherwise shadow it.
fn ft(x: f64) -> FloatType {
    <FloatType as FromPrimitive>::from_f64(x).expect("constant lifts into FloatType")
}

/// `sqrt` / `cosh` at the working precision through the `Real` elementary functions, so the call
/// stays precision-generic instead of binding a concrete-type inherent method.
fn fsqrt<S: Scalar>(x: S) -> S {
    x.sqrt()
}
fn fcosh<S: Scalar>(x: S) -> S {
    x.cosh()
}

/// The Newtonian gravitational potential `Φ(r) = −GM/r`, written once over the working scalar so
/// the tangent functor can differentiate it. The first derivative is the gravitational
/// acceleration, the second is the radial tidal gradient.
struct NewtonianPotential {
    gm: f64,
}

impl DifferentiableArrow for NewtonianPotential {
    fn run<S: Scalar>(&self, r: S) -> S {
        let gm = S::from_f64(self.gm).expect("GM lifts into the working scalar");
        -(gm / r)
    }
}

#[derive(Debug, Clone, Default)]
struct ProbeState {
    distance: FloatType, // Meters from singularity
    velocity: FloatType, // m/s (radial)
    mass: FloatType,     // kg
    status: String,
}
