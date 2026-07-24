/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Measure + verify for the Park-2T blackout slice: the six LER acceptance gates, the blackout
//! observables, and the published reference cross-references (with Tier-A disclaimers).

use crate::FloatType;
use crate::config;
use deep_causality_cfd::{EvidenceClass, Report, ler_step};
use deep_causality_physics::{
    PARK_NO_IONIZATION_ACTIVATION_TEMP, PARK_NO_IONIZATION_EXPONENT, PARK_NO_IONIZATION_PREFACTOR,
    Temperature, arrhenius_rate_kernel, park2t_ionization_surrogate_kernel,
    rankine_hugoniot_temperature_kernel,
};

/// (i) Unconditional stability at stiffness: `τ = Δt/1000` stays bounded/monotone where an explicit
/// Euler rate step diverges.
fn gate_stability_at_stiffness() -> bool {
    let (dt, x_eq) = (1.0_f64, 7000.0_f64);
    let tau = dt / 1000.0;
    let mut x = 300.0_f64;
    for _ in 0..50 {
        let nx = ler_step(x, x_eq, tau, dt);
        if nx < x - 1e-6 || nx > x_eq + 1e-6 {
            return false;
        }
        x = nx;
    }
    let euler = 300.0 + (dt / tau) * (x_eq - 300.0);
    (x - x_eq).abs() < 1.0 && euler > x_eq * 100.0
}

/// Number of sub-steps in the independent reference integration of `dx/dt = (x_eq − x)/τ`.
const REFERENCE_SUBSTEPS: usize = 1_000_000;
/// Tolerance on `|ler_step − reference| / (x_eq − x₀)`.
///
/// Sized from the reference's own truncation error, not from the measurement. Forward Euler with
/// `N` sub-steps gives `(1 − a/N)^N` against the exact `exp(−a)` with `a = Δt/τ`, a relative error
/// of `a²/(2N)`. At `a = 0.3` and `N = 10⁶` that is `4.5e-8`, so `1e-6` clears the reference's own
/// error by ~30× while still failing any real defect (a sign flip or a mis-scaled `τ` moves this
/// by `O(1)`).
const LER_REFERENCE_TOL: f64 = 1.0e-6;

/// (ii) The closed-form relaxation kernel agrees with an **independently-derived** reference: a
/// converged sub-stepped integration of `dx/dt = (x_eq − x)/τ`.
///
/// This gate previously compared `ler_step(..)` against a re-transcription of its own body
/// (`x_eq − (x_eq − x)·exp(−Δt/τ)`) with `==`. Both sides were the same expression at the same
/// monomorphization, so they were bit-identical by construction and no input could make the gate
/// fail. Integrating the ODE numerically is a genuinely separate derivation of the same quantity.
///
/// BREAKING CONDITION: flip the sign of the exponent in `ler_step`, or scale `τ`, and the two
/// disagree by `O(1)` relative — far outside `LER_REFERENCE_TOL`.
fn gate_exponential_exactness() -> bool {
    let (x0, x_eq, tau, dt) = (300.0_f64, 7000.0_f64, 0.01_f64, 0.003_f64);

    // Independent reference: sub-stepped forward Euler on the relaxation ODE. It never calls
    // `ler_step`, and it converges to the same limit from a different algorithm.
    let h = dt / REFERENCE_SUBSTEPS as f64;
    let mut reference = x0;
    for _ in 0..REFERENCE_SUBSTEPS {
        reference += h * (x_eq - reference) / tau;
    }

    let closed_form = ler_step(x0, x_eq, tau, dt);
    let rel = (closed_form - reference).abs() / (x_eq - x0).abs();
    rel < LER_REFERENCE_TOL
}

/// (iii) The mandatory Rankine–Hugoniot jump lands peak `T_post` in the ~10⁴ K band at `M ≈ 25`.
fn gate_rh_temperature_band() -> bool {
    let t_inf = match Temperature::new(config::T_INF) {
        Ok(t) => t,
        Err(_) => return false,
    };
    match rankine_hugoniot_temperature_kernel(t_inf, config::MACH, config::GAMMA) {
        Ok(t_post) => t_post.value() > 1.0e4 && t_post.value() < 1.0e5,
        Err(_) => false,
    }
}

/// (iv) The ionization lag is real and `τ_ion` is grounded in the dominant rate (it varies with `T`).
///
/// The former "Saha limit recovered as `τ → 0`" conjunct has been removed. It called
/// `ler_step(0, α_eq, 0, Δt)`, which takes an explicit early `return x_eq` when `τ <= 0`, then
/// asserted the result equalled `α_eq` — a check of a hard-coded return statement, true for every
/// input. The two surviving conjuncts are falsifiable:
///
/// BREAKING CONDITIONS: invert the sign of the Arrhenius activation exponent and `grounded` fails;
/// make `ler_step` jump straight to equilibrium and `lagged < α_eq` fails.
fn gate_lag_and_saha_limit() -> bool {
    let n = config::NUMBER_DENSITY;
    let t = match Temperature::new(8000.0_f64) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let alpha_eq = match park2t_ionization_surrogate_kernel(t, n) {
        Ok(a) => a.value(),
        Err(_) => return false,
    };
    // τ_ion grounded in the dominant associative-ionization rate — increases with T (not constant).
    let rate = |temp: f64| {
        arrhenius_rate_kernel(
            Temperature::new(temp).unwrap(),
            PARK_NO_IONIZATION_PREFACTOR,
            PARK_NO_IONIZATION_EXPONENT,
            PARK_NO_IONIZATION_ACTIVATION_TEMP,
        )
        .unwrap()
        .value()
    };
    let grounded = rate(9000.0) > rate(6000.0);
    // Lag: a fast ramp (small Δt) leaves α below equilibrium.
    let lagged = ler_step(0.0, alpha_eq, 0.01, 1.0e-5);
    grounded && alpha_eq > 0.0 && lagged < alpha_eq
}

/// (v) Counterfactual path-dependence: two temperature histories reaching the same final target carry
/// different ionization (the LER memory).
fn gate_path_dependence() -> bool {
    let n = config::NUMBER_DENSITY;
    let eq = |temp: f64| {
        park2t_ionization_surrogate_kernel(Temperature::new(temp).unwrap(), n)
            .unwrap()
            .value()
    };
    let (tau, dt) = (0.01_f64, 0.005_f64);
    // History A: hot (8000 K) then settle to 7000 K. History B: cool (5000 K) then 7000 K.
    let a = ler_step(ler_step(0.0, eq(8000.0), tau, dt), eq(7000.0), tau, dt);
    let b = ler_step(ler_step(0.0, eq(5000.0), tau, dt), eq(7000.0), tau, dt);
    (a - b).abs() > 1e-6
}

/// (vi) Ionized species present: the marched electron density is strictly positive somewhere.
fn gate_electrons_produced(report: &Report<FloatType>) -> bool {
    report
        .series("n_e")
        .map(|s| s.iter().any(|&x| x > 0.0))
        .unwrap_or(false)
}

/// Run all six gates, printing a labeled PASS/FAIL line for each; returns `true` only if all pass.
pub fn verify(report: &Report<FloatType>) -> bool {
    // Evidence class per gate. Only (ii) compares against a derivation independent of this
    // codebase (a sub-stepped integration of the relaxation ODE); the rest are internal
    // invariants or loosely-stated physical bands, so they detect regression only.
    let gates: [(&str, EvidenceClass, bool); 6] = [
        (
            "(i)   stability at stiffness (τ=Δt/1000)",
            EvidenceClass::Tripwire,
            gate_stability_at_stiffness(),
        ),
        (
            "(ii)  relaxation kernel vs independent sub-stepped reference",
            EvidenceClass::Reference,
            gate_exponential_exactness(),
        ),
        (
            "(iii) RH jump peak T_post in ~10⁴ K band",
            EvidenceClass::Tripwire,
            gate_rh_temperature_band(),
        ),
        (
            "(iv)  ionization lag real + rate grounded in T",
            EvidenceClass::Tripwire,
            gate_lag_and_saha_limit(),
        ),
        (
            "(v)   counterfactual path-dependence",
            EvidenceClass::Tripwire,
            gate_path_dependence(),
        ),
        (
            "(vi)  ionized species present (n_e>0)",
            EvidenceClass::Tripwire,
            gate_electrons_produced(report),
        ),
    ];
    println!("\n--- LER acceptance gates ---");
    let mut all = true;
    for (label, evidence, pass) in gates {
        println!(
            "  [{}] [{evidence}] {label}",
            if pass { "PASS" } else { "FAIL" }
        );
        all &= pass;
    }
    all
}

/// Print the blackout observables and the published reference cross-references (with disclaimers).
pub fn render(report: &Report<FloatType>) {
    let ne_peak = report
        .series("n_e")
        .and_then(|s| {
            s.iter()
                .copied()
                .fold(None, |m: Option<f64>, x| Some(m.map_or(x, |a| a.max(x))))
        })
        .unwrap_or(0.0);
    let wp_peak = report
        .series("plasma_frequency")
        .and_then(|s| {
            s.iter()
                .copied()
                .fold(None, |m: Option<f64>, x| Some(m.map_or(x, |a| a.max(x))))
        })
        .unwrap_or(0.0);
    let dwell = report
        .series("blackout_dwell")
        .and_then(|s| s.first().copied())
        .unwrap_or(0.0);

    println!("\n--- Blackout observables (Tier-A, incompressible rollout) ---");
    println!("  peak electron density n_e   : {ne_peak:.3e} m^-3");
    println!("  peak plasma frequency ω_p   : {wp_peak:.3e} rad/s");
    println!(
        "  comms band (config)         : {:.3e} rad/s",
        config::COMMS_BAND_RAD_S
    );
    println!("  blackout dwell              : {dwell:.4e} s");

    println!("\n--- Published reference cross-references (Tier-A disclaimers) ---");
    println!(
        "  RAM-C II peak n_e (~71 km)  : ~{:.1e} m^-3 [order-of-magnitude anchor]",
        config::RAMC_NE_REFERENCE
    );
    println!(
        "  DISCLAIMER: Tier-A rides the INCOMPRESSIBLE rollout; T_tr is a recovery-temperature"
    );
    println!("  reconstruction (Rankine–Hugoniot jump + ½|u|²/c_p), NOT a true post-shock path.");
    println!(
        "  Two-temperature (T_ve=T_e) lumping over-predicts peak n_e ~2x (Farbar–Boyd–Martin 2013);"
    );
    println!("  the operator split is first-order Lie. No absolute coupled-CFD match is claimed.");
    println!(
        "  Anchors: RAM-C II (NASA TN), Fluid Dynamics 2022, Aiken–Carter–Boyd 2025, Park 2-T, Saha."
    );
}

/// Print the success summary.
pub fn summary() {
    println!("\n=== All six LER gates passed — Gap-2 Tier-A slice verified. ===");
}
