/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The [`RegimeClassify`] governing-model selector: the Knudsen bands, comms denial, and the
//! opt-in powered-descent flight axes.

use super::{ctx, denying_trigger, field};
use deep_causality_cfd::{
    GoverningModel, MachRegime, PhysicsStage, REGIME_TRANSITIONS_FIELD, RegimeClassify, ThrustState,
};
use deep_causality_haft::LogSize;

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
// RegimeClassify — powered-descent flight axes (flight-regime-classifier)
// ---------------------------------------------------------------------------

/// A classifier with explicit flight bands: subsonic ≤ 0.8, supersonic ≥ 1.2, touchdown ≤ 10 m.
fn flight_classifier() -> RegimeClassify<f64> {
    RegimeClassify::new(1.0, denying_trigger()).with_flight_axes(0.8, 1.2, 10.0)
}

#[test]
fn each_flight_axis_reads_its_published_scalar() {
    let cases = [
        (
            2.5_f64,
            1.0_f64,
            50_000.0_f64,
            MachRegime::Supersonic,
            ThrustState::Burn,
            false,
        ),
        (
            1.0,
            0.0,
            50_000.0,
            MachRegime::Transonic,
            ThrustState::Coast,
            false,
        ),
        (
            0.5,
            0.0,
            5.0,
            MachRegime::Subsonic,
            ThrustState::Coast,
            true,
        ),
    ];
    for (mach, ignited, alt, want_mach, want_thrust, want_touchdown) in cases {
        let mut f = field();
        f.set_scalar("mean_free_path", vec![0.005]);
        f.set_scalar("flight_mach", vec![mach]);
        f.set_scalar("ignited", vec![ignited]);
        f.set_scalar("flight_altitude", vec![alt]);
        flight_classifier().apply(&ctx(0), &mut f).expect("applies");
        let c = f.regime().expect("classified");
        assert_eq!(c.mach_regime, want_mach, "mach {mach}");
        assert_eq!(c.thrust_state, want_thrust, "ignited {ignited}");
        assert_eq!(c.touchdown, want_touchdown, "altitude {alt}");
    }
}

#[test]
fn the_corridor_classification_is_unchanged_without_the_opt_in() {
    // The compressible carrier publishes "flight_mach" every step, so neutrality cannot depend on
    // the scalar being absent: the flight axes are **opt-in**. A classifier built without
    // `with_flight_axes` ignores the published flight scalars entirely, so the regime key reduces
    // to today's (model, gnss_denied) pair and the logged message is exactly the pre-change text.
    let mut f = field();
    f.set_scalar("mean_free_path", vec![0.005]);
    f.set_scalar("n_e", vec![0.0]);
    f.set_scalar("flight_mach", vec![2.5]); // published, as the carrier really does
    f.set_scalar("ignited", vec![1.0]);
    f.set_scalar("flight_altitude", vec![5.0]);
    RegimeClassify::new(1.0, denying_trigger())
        .apply(&ctx(0), &mut f)
        .expect("applies");

    let c = f.regime().expect("classified");
    assert_eq!(c.mach_regime, MachRegime::Unknown);
    assert_eq!(c.thrust_state, ThrustState::Unknown);
    assert!(!c.touchdown);
    assert_eq!(f.log().len(), 1);
    let msg: Vec<&str> = f.log().messages().collect();
    assert!(
        msg[0].starts_with("regime -> continuum (GNSS-available), Kn="),
        "pre-change message text preserved: {}",
        msg[0]
    );
    assert!(
        !msg[0].contains("mach-unknown"),
        "no flight-phase suffix when the axes are neutral: {}",
        msg[0]
    );
}

#[test]
fn a_mach_crossing_under_thrust_logs_once() {
    let stage = flight_classifier();
    let mut f = field();
    f.set_scalar("mean_free_path", vec![0.005]);
    f.set_scalar("ignited", vec![1.0]);
    f.set_scalar("flight_mach", vec![2.5]); // supersonic
    stage.apply(&ctx(0), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1);

    // Same band on the next step: nothing new.
    stage.apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1, "an unchanged band is not re-logged");

    // Cross into transonic: one new entry.
    f.set_scalar("flight_mach", vec![1.0]);
    stage.apply(&ctx(2), &mut f).expect("applies");
    assert_eq!(f.log().len(), 2, "a Mach crossing is a regime change");
}

#[test]
fn a_burn_to_coast_transition_logs() {
    let stage = flight_classifier();
    let mut f = field();
    f.set_scalar("mean_free_path", vec![0.005]);
    f.set_scalar("flight_mach", vec![2.5]);
    f.set_scalar("ignited", vec![1.0]); // burn
    stage.apply(&ctx(0), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1);

    f.set_scalar("ignited", vec![0.0]); // cutoff → coast, same Mach band
    stage.apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(f.log().len(), 2, "burn↔coast is a regime change");
}

#[test]
fn a_touchdown_logs_and_appears_in_the_message() {
    let stage = flight_classifier();
    let mut f = field();
    f.set_scalar("mean_free_path", vec![0.005]);
    f.set_scalar("flight_mach", vec![0.5]);
    f.set_scalar("ignited", vec![0.0]);
    f.set_scalar("flight_altitude", vec![100.0]); // above the floor
    stage.apply(&ctx(0), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1);

    f.set_scalar("flight_altitude", vec![5.0]); // at/below the 10 m floor
    stage.apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(f.log().len(), 2, "touchdown is a regime change");
    let msg: Vec<&str> = f.log().messages().collect();
    assert!(
        msg[1].contains("touchdown"),
        "the phase suffix names it: {}",
        msg[1]
    );
}

// ── The typed transition counter (change `fix-retropulsion-measurement-integrity`) ────────────

#[test]
fn the_transition_counter_increments_once_per_genuine_change() {
    // The classifier already decides whether the regime key changed, in order to log it. That
    // decision is now also published, so a consumer reads a number instead of counting "regime ->"
    // substrings in a rendered log.
    let stage = RegimeClassify::new(1.0, denying_trigger());
    let mut f = field();
    let count = |f: &deep_causality_cfd::CoupledField<f64>| {
        f.scalar(REGIME_TRANSITIONS_FIELD)
            .and_then(|s| s.first().copied())
    };

    assert_eq!(
        count(&f),
        None,
        "nothing published before the first classify"
    );

    // First classification is a change from "no regime".
    f.set_scalar("mean_free_path", vec![0.005]);
    stage.apply(&ctx(0), &mut f).expect("applies");
    assert_eq!(count(&f), Some(1.0));

    // Same band again: not a transition.
    stage.apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(
        count(&f),
        Some(1.0),
        "an unchanged regime is not a transition"
    );

    // A band crossing.
    f.set_scalar("mean_free_path", vec![0.05]);
    stage.apply(&ctx(2), &mut f).expect("applies");
    assert_eq!(count(&f), Some(2.0));

    // And holding in the new band adds nothing.
    stage.apply(&ctx(3), &mut f).expect("applies");
    assert_eq!(count(&f), Some(2.0));
}

#[test]
fn the_transition_counter_matches_the_logged_entries() {
    // The counter and the prose must agree, since the counter exists to replace counting the prose.
    let stage = RegimeClassify::new(1.0, denying_trigger());
    let mut f = field();
    for (step, mfp) in [(0, 0.005_f64), (1, 0.005), (2, 0.05), (3, 1.0), (4, 1.0)] {
        f.set_scalar("mean_free_path", vec![mfp]);
        stage.apply(&ctx(step), &mut f).expect("applies");
    }
    let logged = f
        .log()
        .messages()
        .filter(|m| m.contains("regime ->"))
        .count();
    let counted = f
        .scalar(REGIME_TRANSITIONS_FIELD)
        .and_then(|s| s.first().copied())
        .expect("published");
    assert_eq!(counted, logged as f64);
}
