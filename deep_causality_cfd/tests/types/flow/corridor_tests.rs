/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage 3 corridor composition stages: the [`RegimeClassify`] governing-model selector, the
//! [`BranchAccumulator`]/[`BranchOutcome`] counterfactual branch vocabulary, and the
//! [`CyberneticCorrect`] bounded-correction gate.

use deep_causality_cfd::{
    Ambient, BankCorrection, BlackoutTrigger, BranchAccumulator, CoupledField, CyberneticCorrect,
    GoverningModel, PhysicsStage, RegimeClassify, SafetyEnvelope, StepContext,
};
use deep_causality_haft::LogSize;

fn field() -> CoupledField<f64> {
    CoupledField::new(Ambient::new(0.01_f64, 0.0, None))
}

fn ctx(step: usize) -> StepContext<'static, 2, f64> {
    StepContext::<2, f64>::qtt(0.1, step)
}

// A band that denies the link for any real plasma (ω_p ≫ 1 rad/s for any positive n_e).
fn denying_trigger() -> BlackoutTrigger<f64> {
    BlackoutTrigger::new(1.0)
}

// ---------------------------------------------------------------------------
// GoverningModel
// ---------------------------------------------------------------------------

#[test]
fn governing_model_names_are_stable() {
    assert_eq!(GoverningModel::Continuum.name(), "continuum");
    assert_eq!(GoverningModel::Slip.name(), "slip");
    assert_eq!(GoverningModel::Transitional.name(), "transitional");
    assert_eq!(GoverningModel::FreeMolecular.name(), "free-molecular");
}

// ---------------------------------------------------------------------------
// RegimeClassify (3.1)
// ---------------------------------------------------------------------------

#[test]
fn classify_is_a_noop_without_mean_free_path() {
    let stage = RegimeClassify::new(1.0, denying_trigger());
    let mut f = field();
    stage.apply(&ctx(0), &mut f).expect("applies");
    assert!(f.regime().is_none(), "no classification without λ");
    assert!(f.log().is_empty(), "nothing logged");
}

#[test]
fn classify_selects_each_knudsen_band() {
    // L = 1 m, so Kn = λ directly. One representative λ per band.
    let cases = [
        (0.005_f64, GoverningModel::Continuum),
        (0.05, GoverningModel::Slip),
        (1.0, GoverningModel::Transitional),
        (20.0, GoverningModel::FreeMolecular),
    ];
    for (lambda, expected) in cases {
        let stage = RegimeClassify::new(1.0, denying_trigger());
        let mut f = field();
        f.set_scalar("mean_free_path", vec![lambda * 0.5, lambda]); // peak is `lambda`
        stage.apply(&ctx(0), &mut f).expect("applies");
        let class = f.regime().expect("classified");
        assert_eq!(class.model, expected, "λ={lambda}");
        assert!((class.knudsen - lambda).abs() < 1e-12);
    }
}

#[test]
fn classify_uses_configured_thresholds() {
    // Push the slip band up to 1.0: a Kn of 0.5 is now still continuum.
    let stage = RegimeClassify::new(1.0, denying_trigger()).with_thresholds(1.0, 5.0, 50.0);
    let mut f = field();
    f.set_scalar("mean_free_path", vec![0.5]);
    stage.apply(&ctx(0), &mut f).expect("applies");
    assert_eq!(f.regime().unwrap().model, GoverningModel::Continuum);
}

#[test]
fn classify_flags_gnss_denial_from_electron_density() {
    let stage = RegimeClassify::new(1.0, denying_trigger());

    // Dense plasma → denied.
    let mut denied = field();
    denied.set_scalar("mean_free_path", vec![0.005]);
    denied.set_scalar("n_e", vec![1.0e18]);
    stage.apply(&ctx(0), &mut denied).expect("applies");
    let c = denied.regime().unwrap();
    assert!(c.gnss_denied, "dense plasma denies the link");
    assert!(c.plasma_frequency > 0.0);

    // No plasma → available.
    let mut avail = field();
    avail.set_scalar("mean_free_path", vec![0.005]);
    avail.set_scalar("n_e", vec![0.0]);
    stage.apply(&ctx(0), &mut avail).expect("applies");
    let c = avail.regime().unwrap();
    assert!(!c.gnss_denied, "no plasma leaves the link available");
    assert_eq!(c.plasma_frequency, 0.0);
}

#[test]
fn classify_logs_only_genuine_regime_changes() {
    let stage = RegimeClassify::new(1.0, denying_trigger());
    let mut f = field();

    // First classification is always a change (None -> Continuum): one entry.
    f.set_scalar("mean_free_path", vec![0.005]);
    stage.apply(&ctx(0), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1);

    // Re-applying with the same regime logs nothing new.
    stage.apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1, "unchanged regime is not re-logged");

    // A band change (Continuum -> FreeMolecular) logs a second entry.
    f.set_scalar("mean_free_path", vec![20.0]);
    stage.apply(&ctx(2), &mut f).expect("applies");
    assert_eq!(f.log().len(), 2, "a genuine transition is logged");
}

#[test]
fn classify_logs_a_comms_denial_transition() {
    let stage = RegimeClassify::new(1.0, denying_trigger());
    let mut f = field();
    f.set_scalar("mean_free_path", vec![0.005]);
    f.set_scalar("n_e", vec![0.0]);
    stage.apply(&ctx(0), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1);

    // Same flow band, but now denied: a regime change (the key includes comms denial).
    f.set_scalar("n_e", vec![1.0e18]);
    stage.apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(f.log().len(), 2);
}

// ---------------------------------------------------------------------------
// BranchAccumulator / BranchOutcome (3.2)
// ---------------------------------------------------------------------------

#[test]
fn branch_accumulator_folds_peak_load_and_dwell() {
    let mut acc = BranchAccumulator::<f64>::new(0.35);
    acc.observe(100.0, false, 0.1); // heat 100, comms up
    acc.observe(300.0, true, 0.1); // heat 300 (new peak), denied
    acc.observe(200.0, true, 0.2); // heat 200, denied
    let out = acc.finish(42.0);

    assert_eq!(out.bank_angle, 0.35);
    assert_eq!(out.peak_heat_flux, 300.0);
    // thermal load = 100*0.1 + 300*0.1 + 200*0.2 = 10 + 30 + 40 = 80.
    assert!((out.thermal_load - 80.0).abs() < 1e-9);
    // dwell = 0.1 + 0.2 = 0.3 (only the denied steps).
    assert!((out.blackout_dwell - 0.3).abs() < 1e-9);
    assert_eq!(out.miss_distance, 42.0);
}

#[test]
fn branch_accumulator_with_no_steps_is_zero() {
    let out = BranchAccumulator::<f64>::new(0.0).finish(5.0);
    assert_eq!(out.peak_heat_flux, 0.0);
    assert_eq!(out.thermal_load, 0.0);
    assert_eq!(out.blackout_dwell, 0.0);
    assert_eq!(out.miss_distance, 5.0);
}

// ---------------------------------------------------------------------------
// CyberneticCorrect (3.3)
// ---------------------------------------------------------------------------

fn envelope() -> SafetyEnvelope<f64> {
    // heat ≤ 1e6 W/m², g ≤ 12, |bank| ≤ 0.5 rad.
    SafetyEnvelope::new(1.0e6, 12.0, 0.5)
}

fn gate() -> CyberneticCorrect<f64> {
    CyberneticCorrect::new(envelope())
}

#[test]
fn correction_clamps_bank_into_the_envelope() {
    let mut f = field();
    f.set_scalar("heat_flux", vec![1.0e5]); // within
    f.set_scalar("g_load", vec![3.0]); // within
    f.set_control_action(1.2); // desired bank beyond the 0.5 cap

    gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(f.control_action(), Some(0.5), "bank clamped to +max");
    assert_eq!(f.log().len(), 1, "the bounding is logged");
}

#[test]
fn correction_clamps_negative_bank() {
    let mut f = field();
    f.set_control_action(-3.0);
    gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(f.control_action(), Some(-0.5), "bank clamped to -max");
}

#[test]
fn in_envelope_command_passes_through_unchanged() {
    let mut f = field();
    f.set_scalar("heat_flux", vec![2.0e5]);
    f.set_control_action(0.2); // already inside [-0.5, 0.5]
    gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(f.control_action(), Some(0.2));
    assert!(f.log().is_empty(), "an unchanged command is not logged");
}

#[test]
fn heat_breach_returns_entropy_and_logs() {
    let mut f = field();
    f.set_scalar("heat_flux", vec![2.0e6]); // above the 1e6 ceiling
    f.set_control_action(0.1);
    let result = gate().apply(&ctx(0), &mut f);
    assert!(result.is_err(), "an unrecoverable breach short-circuits");
    assert_eq!(f.log().len(), 1);
}

#[test]
fn g_load_breach_returns_entropy() {
    let mut f = field();
    f.set_scalar("g_load", vec![20.0]); // above the 12 g ceiling
    f.set_control_action(0.1);
    assert!(gate().apply(&ctx(0), &mut f).is_err());
}

#[test]
fn absent_sensor_fields_are_treated_as_zero_and_safe() {
    let mut f = field();
    f.set_control_action(0.3);
    // No heat_flux / g_load fields → sensed as zero → inside envelope.
    gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(f.control_action(), Some(0.3));
}

#[test]
fn gate_is_deterministic() {
    let build = || {
        let mut f = field();
        f.set_scalar("heat_flux", vec![5.0e5]);
        f.set_scalar("g_load", vec![6.0]);
        f.set_control_action(0.9);
        f
    };
    let mut a = build();
    let mut b = build();
    gate().apply(&ctx(0), &mut a).expect("applies");
    gate().apply(&ctx(0), &mut b).expect("applies");
    assert_eq!(a.control_action(), b.control_action());
    assert_eq!(a.control_action(), Some(0.5));
}

#[test]
fn bank_correction_value_equality() {
    assert_eq!(
        BankCorrection::Clamped(0.5_f64),
        BankCorrection::Clamped(0.5)
    );
    assert_ne!(
        BankCorrection::Clamped(0.5_f64),
        BankCorrection::NoSafeAction
    );
}
