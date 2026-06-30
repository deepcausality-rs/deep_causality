/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B Stage 4 gates: the fitted normal shock (`FittedNormalShock`) — exact Rankine–Hugoniot jump, the
//! nonequilibrium ionization lag, and the `O(1)`-rank post-shock relaxation profile.

use deep_causality_cfd::FittedNormalShock;
use deep_causality_tensor::Truncation;

fn tr() -> Truncation<f64> {
    Truncation::by_tol(1e-10).unwrap()
}

#[test]
fn rejects_gamma_at_or_below_one() {
    assert!(FittedNormalShock::<f64>::new(1.0).is_err());
    assert!(FittedNormalShock::<f64>::new(0.9).is_err());
    assert!(FittedNormalShock::<f64>::new(1.4).is_ok());
}

#[test]
fn post_shock_ratios_match_exact_rh() {
    // Closed-form normal-shock ratios at M = 5, γ = 1.4.
    let (mach, gamma) = (5.0f64, 1.4f64);
    let m2 = mach * mach;
    let rho_ratio_exact = (gamma + 1.0) * m2 / ((gamma - 1.0) * m2 + 2.0);
    let p_ratio_exact = (2.0 * gamma * m2 - (gamma - 1.0)) / (gamma + 1.0);

    let shock = FittedNormalShock::<f64>::new(gamma).unwrap();
    let post = shock.post_shock(250.0, 1.0e22, mach).unwrap();

    assert!((post.rho_ratio - rho_ratio_exact).abs() < 1e-10);
    assert!((post.p_ratio - p_ratio_exact).abs() < 1e-10);
    assert!((post.u_ratio - 1.0 / rho_ratio_exact).abs() < 1e-12);
    // Number density scales with the density ratio.
    assert!((post.n_tot2 - 1.0e22 * rho_ratio_exact).abs() / post.n_tot2 < 1e-12);
}

#[test]
fn reacting_gamma_lands_t2_in_the_10k_band() {
    // Effective reacting-air γ ≈ 1.1 lands T₂ in the realistic ~10⁴ K post-shock band at M ≈ 25.
    let shock = FittedNormalShock::<f64>::new(1.1).unwrap();
    let post = shock.post_shock(250.0, 1.3e21, 25.0).unwrap();
    assert!(
        post.t2 > 5_000.0 && post.t2 < 15_000.0,
        "T2 should be ~10^4 K, got {}",
        post.t2
    );
}

#[test]
fn nonequilibrium_lag_sits_below_saha_equilibrium() {
    // The grounded ionization lag must pull the peak n_e below the Saha-equilibrium upper bound.
    let shock = FittedNormalShock::<f64>::new(1.1).unwrap();
    let post = shock.post_shock(250.0, 1.3e21, 25.0).unwrap();
    let comms = 9.4e9;
    let residence_time = 2.0e-5;

    let equilibrium = shock.stagnation_blackout(&post, comms).unwrap();
    let lagged = shock
        .stagnation_line_blackout(&post, residence_time, comms)
        .unwrap();

    assert!(
        lagged.electron_density < equilibrium.electron_density,
        "the lag must reduce n_e below equilibrium: lagged {:.3e} vs eq {:.3e}",
        lagged.electron_density,
        equilibrium.electron_density
    );
    assert!(lagged.electron_density > 0.0);
    // Both ionize enough to black out the comms band at this flight condition.
    assert!(lagged.blackout && equilibrium.blackout);
}

#[test]
fn ramc_peak_ne_in_order_of_magnitude_band() {
    // The milestone gate: peak n_e within ~2 decades of the RAM-C II anchor (order-of-magnitude surrogate).
    let shock = FittedNormalShock::<f64>::new(1.1).unwrap();
    let post = shock.post_shock(250.0, 1.3e21, 25.0).unwrap();
    let u2 = 7650.0 * post.u_ratio;
    let residence_time = 0.0076 / u2;
    let out = shock
        .stagnation_line_blackout(&post, residence_time, 9.4e9)
        .unwrap();
    assert!(
        out.electron_density > 1.0e17 && out.electron_density < 1.0e21,
        "peak n_e {:.3e} should be within ~2 decades of RAM-C II (1e19)",
        out.electron_density
    );
}

#[test]
fn relaxation_profile_is_low_rank() {
    let shock = FittedNormalShock::<f64>::new(1.1).unwrap();
    let post = shock.post_shock(250.0, 1.3e21, 25.0).unwrap();
    let (bond, peak) = shock
        .relaxation_profile_bond(&post, 10, 0.2, &tr())
        .unwrap();
    assert!(
        bond <= 4,
        "smooth post-shock profile should be O(1) rank, got {bond}"
    );
    assert!(peak > 0.0);
}
