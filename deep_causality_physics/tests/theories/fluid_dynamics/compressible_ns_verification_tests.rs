/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reference-solution verification for the compressible Navier-Stokes
//! regime evaluators.
//!
//! Each test cites an external reference for the setup and the expected RHS.
//!
//! References:
//!   - Landau & Lifshitz, "Fluid Mechanics" (2nd ed., 1987), §64
//!     "The energy and momentum of sound waves", and Ch. VIII generally.
//!   - A. D. Pierce, "Acoustics: An Introduction to Its Physical Principles
//!     and Applications" (3rd ed., ASA Press, 2019), §1.3 "Linear acoustic
//!     equations and the wave equation".
//!   - Anderson, "Modern Compressible Flow" (3rd ed., 2003), §3.4 "Speed of
//!     sound", Eq. (3.18) for c² = γ p₀/ρ₀.
//!   - Batchelor (1967), §1.6 for the linearised compressible equations.

use deep_causality_physics::{
    AccelerationVector, Density, Velocity3, VelocityGradient,
    compressible_ns_continuity_rhs_kernel, compressible_ns_momentum_rhs_kernel,
};

// =============================================================================
// Verification: 1D linear (small-amplitude) acoustic wave
//
// Source: Landau & Lifshitz (1987) §64; Pierce (2019) §1.3 Eqs. (1-3.1) to
//         (1-3.5); Anderson (2003) §3.4 Eq. (3.18).
//
// Linearised compressible Euler equations around a quiescent base state
// (ρ₀, p₀, u = 0) for a barotropic fluid admit a plane-wave solution
//
//   u(x,t)  =  u₀ sin(k x − ω t)
//   ρ(x,t)  =  ρ₀ + (u₀/c) ρ₀ sin(k x − ω t)
//   p(x,t)  =  p₀ + (u₀ ρ₀ c) sin(k x − ω t)
//
// with dispersion relation ω = c k and c² = γ p₀ / ρ₀ (Anderson Eq. 3.18).
//
// Sampling at (x, t) = (0, 0):
//
//   u            =  0
//   ρ            =  ρ₀
//   p            =  p₀
//   ∂u/∂x        =  u₀ k        (so grad_u[0][0] = u₀ k, all others = 0)
//   ∂ρ/∂x        =  ρ₀ u₀ k / c
//   ∂p/∂x        =  ρ₀ c u₀ k
//   div u        =  u₀ k
//
// Reference time derivatives (from the analytic plane-wave solution):
//
//   ∂ρ/∂t |_{(0,0)} = − (u₀/c) ρ₀ · ω · cos(0) = − ρ₀ u₀ k  (since ω = c k)
//   ∂u/∂t |_{(0,0)} = − u₀ ω cos(0)            = − u₀ c k
//
// Inviscid (∇·τ = 0) momentum-RHS check:
//   −(u·∇)u    =  0
//   −(1/ρ) ∇p = −(1/ρ₀) · ρ₀ c u₀ k = −c u₀ k     ✓ matches ∂u/∂t.
//
// Continuity-RHS check:
//   −u·∇ρ − ρ·(∇·u) = 0 − ρ₀ u₀ k = −ρ₀ u₀ k       ✓ matches ∂ρ/∂t.
// =============================================================================

const TOL: f64 = 1e-12;

struct AcousticReference {
    rho0: f64,
    c: f64,
    u0: f64,
    k: f64,
}

impl AcousticReference {
    fn air_at_stp() -> Self {
        // STP air: ρ₀ = 1.225 kg/m³, γ = 1.4, p₀ = 101 325 Pa
        // ⇒ c = sqrt(γ p₀/ρ₀) ≈ 340.29 m/s (Anderson §3.4 worked example).
        let rho0 = 1.225_f64;
        let gamma = 1.4_f64;
        let p0 = 101_325.0_f64;
        let c = (gamma * p0 / rho0).sqrt();
        Self {
            rho0,
            c,
            u0: 0.01_f64, // small-amplitude perturbation, 1 cm/s
            k: 0.5_f64,   // wavenumber rad/m
        }
    }
}

#[test]
fn test_compressible_ns_acoustic_wave_continuity() {
    let r = AcousticReference::air_at_stp();
    // Sample at (x, t) = (0, 0): u = 0, ρ = ρ₀.
    let rho = Density::<f64>::new(r.rho0).unwrap();
    let u = Velocity3::<f64>::new([0.0; 3]).unwrap();
    let grad_rho = [r.rho0 * r.u0 * r.k / r.c, 0.0, 0.0];
    let div_u = r.u0 * r.k;

    let rhs = compressible_ns_continuity_rhs_kernel(&rho, &u, &grad_rho, div_u);
    // Reference (Landau & Lifshitz §64 / Pierce §1.3): ∂ρ/∂t = −ρ₀ u₀ k.
    let expected = -r.rho0 * r.u0 * r.k;
    assert!(
        (rhs - expected).abs() < TOL * expected.abs().max(1.0),
        "continuity RHS = {}, expected {}",
        rhs,
        expected
    );
}

#[test]
fn test_compressible_ns_acoustic_wave_momentum() {
    let r = AcousticReference::air_at_stp();
    let rho = Density::<f64>::new(r.rho0).unwrap();
    let u = Velocity3::<f64>::new([0.0; 3]).unwrap();
    let grad_u =
        VelocityGradient::<f64>::new([[r.u0 * r.k, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]])
            .unwrap();
    let grad_p = [r.rho0 * r.c * r.u0 * r.k, 0.0, 0.0];
    let div_tau = [0.0_f64; 3]; // inviscid (Euler limit of compressible NS)
    let body = AccelerationVector::<f64>::new([0.0; 3]).unwrap();

    let rhs = compressible_ns_momentum_rhs_kernel(&u, &grad_u, &grad_p, &div_tau, &rho, &body)
        .unwrap()
        .into_inner();
    // Reference (Anderson §3.4 Eq. 3.18; Landau & Lifshitz §64):
    // ∂u/∂t = −c u₀ k along the wave direction; transverse components 0.
    let expected_x = -r.c * r.u0 * r.k;
    assert!(
        (rhs[0] - expected_x).abs() < 1e-10 * expected_x.abs().max(1.0),
        "acoustic momentum RHS_x = {}, expected {}",
        rhs[0],
        expected_x
    );
    assert!(rhs[1].abs() < TOL);
    assert!(rhs[2].abs() < TOL);
}

// =============================================================================
// Verification: Speed-of-sound consistency
//
// Source: Anderson, "Modern Compressible Flow" (2003), §3.4 Eq. (3.18):
//         c² = γ p₀ / ρ₀ for an ideal gas. Combining the two acoustic-wave
//         RHS checks above yields, on elimination of u₀, ρ₀, k:
//
//           |∂u/∂t| / |∂ρ/∂t / ρ₀| = c
//
// i.e. the ratio of the momentum and continuity RHS magnitudes recovers the
// speed of sound. This is the *physical* content of the dispersion relation
// ω = c k applied to the kernel outputs and is independent of the absolute
// scale of the perturbation amplitude u₀.
// =============================================================================

#[test]
fn test_compressible_ns_acoustic_dispersion_recovers_sound_speed() {
    let r = AcousticReference::air_at_stp();
    let rho = Density::<f64>::new(r.rho0).unwrap();
    let u = Velocity3::<f64>::new([0.0; 3]).unwrap();

    // Continuity at (0, 0).
    let grad_rho = [r.rho0 * r.u0 * r.k / r.c, 0.0, 0.0];
    let div_u = r.u0 * r.k;
    let drho_dt = compressible_ns_continuity_rhs_kernel(&rho, &u, &grad_rho, div_u);

    // Momentum at (0, 0).
    let grad_u =
        VelocityGradient::<f64>::new([[r.u0 * r.k, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]])
            .unwrap();
    let grad_p = [r.rho0 * r.c * r.u0 * r.k, 0.0, 0.0];
    let div_tau = [0.0_f64; 3];
    let body = AccelerationVector::<f64>::new([0.0; 3]).unwrap();
    let du_dt = compressible_ns_momentum_rhs_kernel(&u, &grad_u, &grad_p, &div_tau, &rho, &body)
        .unwrap()
        .into_inner();

    // Reference: c = |∂u/∂t| / |∂ρ/∂t / ρ₀|, recovered to 1e-10 relative.
    let c_recovered = du_dt[0].abs() / (drho_dt.abs() / r.rho0);
    assert!(
        (c_recovered - r.c).abs() < 1e-10 * r.c,
        "speed of sound recovered from kernel outputs = {}, expected {} m/s",
        c_recovered,
        r.c
    );
}
