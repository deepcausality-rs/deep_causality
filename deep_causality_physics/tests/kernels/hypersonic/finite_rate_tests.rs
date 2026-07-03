/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Pointwise validation of the finite-rate ionization network kernels:
//! detailed balance through the RP-1232 Table II pairs, equilibrium recovery
//! from both sides, frozen limits at low temperature, and purity.

use deep_causality_physics::{
    ElectronTemperature, EquilibriumConstant, RP1232_NO_DR_EXPONENT, RP1232_NO_DR_PREFACTOR,
    Temperature, arrhenius_rate_kernel, dissociation_equilibrium_fraction_kernel,
    electron_impact_ionization_n_rate_kernel, electron_impact_ionization_o_rate_kernel,
    finite_rate_ionization_fixed_point_kernel, n2_dissociation_equilibrium_kernel,
    no_dissociative_recombination_rate_kernel, o2_dissociation_equilibrium_kernel,
};

// ── Dissociative recombination (Table II, reaction 7 backward) ────────────

#[test]
fn dr_rate_matches_the_table_form_and_is_barrier_free() {
    // k_b = 1.80e19 · T⁻¹, no activation barrier: the rate *rises* as the
    // electrons cool, which is what makes it the exit mechanism.
    let cold =
        no_dissociative_recombination_rate_kernel(ElectronTemperature::new(2_000.0_f64).unwrap())
            .unwrap()
            .value();
    let hot =
        no_dissociative_recombination_rate_kernel(ElectronTemperature::new(8_000.0_f64).unwrap())
            .unwrap()
            .value();
    assert!(cold > hot, "barrier-free DR grows as T_e falls");
    let expected = RP1232_NO_DR_PREFACTOR * 2_000.0_f64.powf(RP1232_NO_DR_EXPONENT);
    assert!((cold - expected).abs() / expected < 1e-12);
}

#[test]
fn dr_pairs_with_the_forward_rate_as_a_finite_equilibrium_constant() {
    // K_eq = k_f / k_b from the same table row (RP-1232 eq. 5a) must be
    // finite and positive across the tabulated range.
    for t in [2_000.0_f64, 4_000.0, 6_000.0, 8_000.0, 12_000.0] {
        let kf = arrhenius_rate_kernel(Temperature::new(t).unwrap(), 9.03e9_f64, 0.5, 32_400.0)
            .unwrap()
            .value();
        let kb = no_dissociative_recombination_rate_kernel(ElectronTemperature::new(t).unwrap())
            .unwrap()
            .value();
        let k_eq = kf / kb;
        assert!(k_eq.is_finite() && k_eq > 0.0, "K_eq at {t} K: {k_eq}");
    }
}

// ── Electron impact (Table II, reactions 8 and 9 forward) ─────────────────

#[test]
fn electron_impact_is_thresholded_and_frozen_when_cold() {
    // The 1.58e5 / 1.69e5 K activation temperatures freeze both channels at
    // sheath electron temperatures of a few thousand kelvin.
    let n_cold =
        electron_impact_ionization_n_rate_kernel(ElectronTemperature::new(2_000.0_f64).unwrap())
            .unwrap()
            .value();
    let o_cold =
        electron_impact_ionization_o_rate_kernel(ElectronTemperature::new(2_000.0_f64).unwrap())
            .unwrap()
            .value();
    // Frozen in the operational sense: at sheath concentrations (~1e-6
    // mol/cm³) these rates give per-electron ionization frequencies below
    // 1e-18 s⁻¹, nothing over a 2e-5 s residence time.
    assert!(n_cold < 1e-12, "N impact frozen at 2000 K: {n_cold}");
    assert!(o_cold < 1e-12, "O impact frozen at 2000 K: {o_cold}");

    let n_hot =
        electron_impact_ionization_n_rate_kernel(ElectronTemperature::new(15_000.0_f64).unwrap())
            .unwrap()
            .value();
    assert!(n_hot > n_cold * 1e6, "the threshold opens with T_e");
}

// ── Dissociation equilibria (Table II, reactions 1 and 2 pairs) ───────────

#[test]
fn dissociation_equilibrium_constants_grow_with_temperature() {
    let k_n2_cold = n2_dissociation_equilibrium_kernel(Temperature::new(4_000.0_f64).unwrap())
        .unwrap()
        .value();
    let k_n2_hot = n2_dissociation_equilibrium_kernel(Temperature::new(8_000.0_f64).unwrap())
        .unwrap()
        .value();
    assert!(k_n2_hot > k_n2_cold, "hotter air dissociates more");

    // O2 dissociates far more readily than N2 at the same temperature
    // (activation 5.94e4 K vs 1.131e5 K).
    let k_o2 = o2_dissociation_equilibrium_kernel(Temperature::new(4_000.0_f64).unwrap())
        .unwrap()
        .value();
    assert!(k_o2 > k_n2_cold, "O2 leads N2 in dissociation");
}

#[test]
fn dissociation_fraction_recovers_both_limits() {
    let n_nuclei = 1.0e-6_f64; // mol/cm³ basis
    // K >> n: fully dissociated.
    let full = dissociation_equilibrium_fraction_kernel(
        EquilibriumConstant::new(1.0e3_f64).unwrap(),
        n_nuclei,
    )
    .unwrap()
    .value();
    assert!(full > 0.999, "K >> n dissociates fully: {full}");
    // K << n: bound.
    let bound = dissociation_equilibrium_fraction_kernel(
        EquilibriumConstant::new(1.0e-20_f64).unwrap(),
        n_nuclei,
    )
    .unwrap()
    .value();
    assert!(bound < 1e-3, "K << n stays molecular: {bound}");
    // The exact quadratic: [A]²/[A2] = K must hold at the returned fraction.
    let k = 1.0e-7_f64;
    let x =
        dissociation_equilibrium_fraction_kernel(EquilibriumConstant::new(k).unwrap(), n_nuclei)
            .unwrap()
            .value();
    let atoms = x * n_nuclei;
    let molecules = (n_nuclei - atoms) / 2.0;
    assert!(
        ((atoms * atoms / molecules) - k).abs() / k < 1e-9,
        "the closed form satisfies the equilibrium it solves"
    );
}

#[test]
fn dissociation_fraction_rejects_a_nonpositive_pool() {
    let err = dissociation_equilibrium_fraction_kernel(
        EquilibriumConstant::new(1.0_f64).unwrap(),
        0.0_f64,
    );
    assert!(err.is_err());
}

// ── The fixed point ────────────────────────────────────────────────────────

#[test]
fn fixed_point_balances_production_and_loss_exactly() {
    let (p, k_lin, beta) = (2.0e-9_f64, 3.0e2_f64, 5.0e11_f64);
    let x = finite_rate_ionization_fixed_point_kernel(p, k_lin, beta)
        .unwrap()
        .value();
    let residual = beta * x * x - k_lin * x - p;
    assert!(
        residual.abs() < 1e-9 * (beta * x * x),
        "quadratic residual vanishes at the fixed point"
    );
}

#[test]
fn fixed_point_is_approached_consistently_from_both_sides() {
    // Equilibrium recovery from both sides: an under-ionized and an
    // over-ionized population relax toward the same fixed point (here the
    // fixed point itself is closed-form; both sides bracket it).
    let (p, k_lin, beta) = (1.0e-10_f64, 0.0_f64, 1.0e12_f64);
    let x = finite_rate_ionization_fixed_point_kernel(p, k_lin, beta)
        .unwrap()
        .value();
    let below = x * 0.1;
    let above = x * 10.0;
    // Net rate sign: production − loss must push both toward x*.
    let net_below = p + k_lin * below - beta * below * below;
    let net_above = p + k_lin * above - beta * above * above;
    assert!(
        net_below > 0.0,
        "under-ionized grows toward the fixed point"
    );
    assert!(
        net_above < 0.0,
        "over-ionized decays toward the fixed point"
    );
}

#[test]
fn fixed_point_without_production_is_zero() {
    // Frozen forward channels with nothing carried: the fixed point is zero
    // and stays there (the LER frozen-chemistry contract upstream).
    let x = finite_rate_ionization_fixed_point_kernel(0.0_f64, 0.0, 1.0e12)
        .unwrap()
        .value();
    assert_eq!(x, 0.0);
}

#[test]
fn fixed_point_guards_its_domain() {
    assert!(finite_rate_ionization_fixed_point_kernel(-1.0_f64, 0.0, 1.0).is_err());
    assert!(finite_rate_ionization_fixed_point_kernel(1.0_f64, 0.0, 0.0).is_err());
}

// ── Purity (two states, two outputs) ──────────────────────────────────────

#[test]
fn kernels_are_pure_and_dynamic() {
    let a =
        no_dissociative_recombination_rate_kernel(ElectronTemperature::new(3_000.0_f64).unwrap())
            .unwrap()
            .value();
    let b =
        no_dissociative_recombination_rate_kernel(ElectronTemperature::new(6_000.0_f64).unwrap())
            .unwrap()
            .value();
    assert_ne!(a, b);
    let c = n2_dissociation_equilibrium_kernel(Temperature::new(5_000.0_f64).unwrap())
        .unwrap()
        .value();
    let d = n2_dissociation_equilibrium_kernel(Temperature::new(9_000.0_f64).unwrap())
        .unwrap()
        .value();
    assert_ne!(c, d);
}

// ── The revision channels (Zeldovich + the dissociation controller) ───────

#[test]
fn zeldovich_opens_three_decades_before_direct_dissociation() {
    use deep_causality_physics::{
        RP1232_N2_DISS_ACTIVATION_TEMP, RP1232_N2_DISS_EXPONENT, RP1232_N2_DISS_PREFACTOR,
        zeldovich_exchange_rate_kernel,
    };
    let t = Temperature::new(6_000.0_f64).unwrap();
    let k_z = zeldovich_exchange_rate_kernel(t).unwrap().value();
    let k_d = arrhenius_rate_kernel(
        t,
        RP1232_N2_DISS_PREFACTOR,
        RP1232_N2_DISS_EXPONENT,
        RP1232_N2_DISS_ACTIVATION_TEMP,
    )
    .unwrap()
    .value();
    assert!(
        k_z > k_d * 1.0e3,
        "the 37,500 K barrier beats the 113,100 K one: {k_z} vs {k_d}"
    );
}

#[test]
fn the_dissociation_controller_sits_between_its_temperatures() {
    use deep_causality_physics::{PARK_DISSOCIATION_Q, park_controlling_temperature_kernel};
    let t_tr = Temperature::new(8_044.0_f64).unwrap();
    let t_ve = Temperature::new(4_000.0_f64).unwrap();
    let t_q = park_controlling_temperature_kernel(t_tr, t_ve, PARK_DISSOCIATION_Q)
        .unwrap()
        .value();
    assert!(t_q > 4_000.0 && t_q < 8_044.0);
    // q = 0.7 weights translation harder than the geometric mean.
    let geo = (8_044.0_f64 * 4_000.0).sqrt();
    assert!(
        t_q > geo,
        "Park's classic exponent runs hotter: {t_q} vs {geo}"
    );
    // Domain guard.
    assert!(park_controlling_temperature_kernel(t_tr, t_ve, 1.5).is_err());
}
