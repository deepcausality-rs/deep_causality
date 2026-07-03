/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The finite-rate ionization network stage: two-way relaxation toward the
//! uncalibrated network fixed point, the recombination exit mechanism, the
//! frozen limit, per-cell evolved inputs, and the renewal toggle.

use deep_causality_cfd::{
    Ambient, CoupledField, FiniteRateIonizationStage, PhysicsStage, StepContext,
};

fn ctx(step: usize) -> StepContext<'static, 2, f64> {
    StepContext::<2, f64>::qtt(2.0e-5, step)
}

fn field_at(t_tr: f64, t_ve: f64) -> CoupledField<f64> {
    let mut f = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    f.set_scalar("T_tr", vec![t_tr]);
    f.set_scalar("T_ve", vec![t_ve]);
    f
}

const N_PEAK: f64 = 2.645e22;

#[test]
fn stage_is_a_noop_without_the_controller_field() {
    let stage = FiniteRateIonizationStage::new(N_PEAK);
    let mut f = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    stage.apply(&ctx(1), &mut f).expect("applies");
    assert!(f.scalar("n_e").is_none());
}

#[test]
fn the_fixed_point_is_approached_from_both_sides() {
    // Relax an under-ionized and an over-ionized carried fraction at the same
    // hot state: both must move toward the same network fixed point.
    let stage = FiniteRateIonizationStage::new(N_PEAK);

    let mut low = field_at(6_000.0, 6_000.0);
    low.set_scalar("alpha", vec![0.0_f64]);
    let mut high = field_at(6_000.0, 6_000.0);
    high.set_scalar("alpha", vec![0.9_f64]);

    // March both for many steps so they converge.
    for s in 0..20_000 {
        stage.apply(&ctx(s + 1), &mut low).expect("applies");
        stage.apply(&ctx(s + 1), &mut high).expect("applies");
    }
    let a_low = low.scalar("alpha").unwrap()[0];
    let a_high = high.scalar("alpha").unwrap()[0];
    assert!(a_low > 0.0, "under-ionized grew: {a_low}");
    assert!(a_high < 0.9, "over-ionized decayed: {a_high}");
    assert!(
        (a_low - a_high).abs() / a_low < 0.05,
        "both sides converge to one fixed point: {a_low} vs {a_high}"
    );
}

#[test]
fn recombination_decays_a_hot_fraction_in_a_cold_cell() {
    // The exit mechanism: a carried electron population entering cold dense
    // air must decay, which the forward-only surrogate could never do.
    let stage = FiniteRateIonizationStage::new(2.0e24);
    let mut f = field_at(2_500.0, 2_500.0);
    f.set_scalar("alpha", vec![1.0e-4_f64]);
    let a0 = 1.0e-4_f64;
    for s in 0..200 {
        stage.apply(&ctx(s + 1), &mut f).expect("applies");
    }
    let a = f.scalar("alpha").unwrap()[0];
    assert!(a < a0 * 0.5, "the carried fraction decays in the cold: {a}");
}

#[test]
fn the_frozen_limit_leaves_an_empty_population_untouched() {
    // Cold, thin, and nothing carried: every channel is frozen and the
    // fraction stays at zero (the LER frozen-chemistry contract).
    let stage = FiniteRateIonizationStage::new(1.0e19);
    let mut f = field_at(300.0, 300.0);
    for s in 0..50 {
        stage.apply(&ctx(s + 1), &mut f).expect("applies");
    }
    let a = f.scalar("alpha").unwrap()[0];
    assert!(a < 1.0e-30, "frozen and empty stays empty: {a}");
}

#[test]
fn per_cell_density_and_temperature_drive_per_cell_electrons() {
    let stage = FiniteRateIonizationStage::new(1.0_f64)
        .with_density_field("n_tot")
        .with_sheath_renewal(2.0e-5);
    let mut f = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    f.set_scalar("T_tr", vec![6_000.0_f64, 6_000.0, 3_000.0]);
    f.set_scalar("T_ve", vec![6_000.0_f64, 6_000.0, 3_000.0]);
    f.set_scalar("n_tot", vec![1.0e20_f64, N_PEAK, N_PEAK]);
    stage.apply(&ctx(1), &mut f).expect("applies");
    let ne = f.scalar("n_e").unwrap();
    assert_eq!(ne.len(), 3);
    assert!(
        ne[1] > ne[0],
        "denser cell ionizes more: {} vs {}",
        ne[1],
        ne[0]
    );
    assert!(
        ne[1] > ne[2],
        "hotter cell ionizes more: {} vs {}",
        ne[1],
        ne[2]
    );
}

#[test]
fn the_lagged_pool_sits_below_its_equilibrium() {
    // The atom pool must lag its dissociation equilibrium over one residence
    // time (the D3 amendment): the pool fractions are far below equilibrium
    // at post-shock conditions, not at it.
    let stage = FiniteRateIonizationStage::new(N_PEAK).with_sheath_renewal(2.0e-5);
    let mut f = field_at(8_000.0, 4_000.0);
    stage.apply(&ctx(1), &mut f).expect("applies");
    let x_n = f.scalar("atom_frac_n").unwrap()[0];
    let x_o = f.scalar("atom_frac_o").unwrap()[0];
    assert!((0.0..=1.0).contains(&x_n));
    assert!(x_o > x_n, "O2 dissociates ahead of N2: {x_o} vs {x_n}");
    // N2 dissociation at 8000 K is slow against a 2e-5 s residence: the lag
    // must hold the pool visibly below the equilibrium fraction.
    assert!(x_n < 0.5, "the N pool lags its equilibrium: {x_n}");
}

#[test]
fn renewal_mode_is_stateless_per_step() {
    let renewed = FiniteRateIonizationStage::new(N_PEAK).with_sheath_renewal(2.0e-5);
    let mut f = field_at(7_000.0, 4_000.0);
    let mut first = 0.0;
    for s in 0..50 {
        renewed.apply(&ctx(s + 1), &mut f).expect("applies");
        if s == 0 {
            first = f.scalar("n_e").unwrap()[0];
        }
    }
    let last = f.scalar("n_e").unwrap()[0];
    assert_eq!(
        first, last,
        "renewal caps the exposure at one residence time"
    );
    assert!(last > 0.0, "one residence time still ionizes");
}

#[test]
fn electron_temperature_field_falls_back_to_the_controller() {
    let stage = FiniteRateIonizationStage::new(N_PEAK)
        .with_electron_temperature_field("missing_te")
        .with_sheath_renewal(2.0e-5);
    let mut with_fallback = field_at(6_000.0, 4_000.0);
    stage.apply(&ctx(1), &mut with_fallback).expect("applies");

    let explicit = FiniteRateIonizationStage::new(N_PEAK)
        .with_electron_temperature_field("T_e_same")
        .with_sheath_renewal(2.0e-5);
    let mut same = field_at(6_000.0, 4_000.0);
    same.set_scalar("T_e_same", vec![6_000.0_f64]);
    explicit.apply(&ctx(1), &mut same).expect("applies");

    assert_eq!(
        with_fallback.scalar("n_e").unwrap()[0],
        same.scalar("n_e").unwrap()[0],
        "an absent electron-temperature field means the controller value"
    );
}
