/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The gate's powered-descent burn axes: the dynamic C_T cap, the throttle window, the ignition
//! dynamic-pressure window, and the M4 live-enforcement refusals.

use super::{ctx, field, gate};
use deep_causality_cfd::{
    BurnEnvelope, CoupledField, CyberneticCorrect, PhysicsStage, SafetyEnvelope,
};
use deep_causality_haft::LogSize;

// Burn envelope: throttle ∈ [0.1, 0.9], max_ct 2.0, ignition q window [1000, 5000],
// propellant floor 10 kg, max descent rate 100 m/s. Sensing: thrust_ref 2000 N, S_ref 0.785 m².
// The dynamic cap is ct_ceiling = max_ct·q·S_ref/thrust_ref = q·7.85e-4.
fn burn_gate() -> CyberneticCorrect<f64> {
    let envelope = SafetyEnvelope::new(1.0e6, 12.0, 0.5).with_burn(BurnEnvelope::new(
        0.1, 0.9, 2.0, 1000.0, 5000.0, 10.0, 100.0,
    ));
    CyberneticCorrect::new(envelope).with_burn_sensing(
        "q_inf",
        "propellant",
        "descent_rate",
        2000.0,
        0.785,
    )
}

#[test]
fn dynamic_ct_cap_binds_below_the_static_ceiling() {
    let mut f = field();
    f.set_scalar("q_inf", vec![1_000.0]); // ct_ceiling = 0.785 < static 0.9
    f.set_scalar("propellant", vec![100.0]);
    f.set_throttle_action(0.85); // inside the static ceiling, above the dynamic cap
    burn_gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(
        f.throttle_action(),
        Some(0.785),
        "throttle capped by the C_T ceiling"
    );
    assert_eq!(f.log().len(), 1, "the dynamic cap is logged");
}

#[test]
fn static_ceiling_binds_when_dynamic_pressure_is_high() {
    let mut f = field();
    f.set_scalar("q_inf", vec![2_000.0]); // ct_ceiling = 1.57 > static 0.9
    f.set_scalar("propellant", vec![100.0]);
    f.set_throttle_action(1.2); // above the static ceiling
    burn_gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(
        f.throttle_action(),
        Some(0.9),
        "throttle capped by the static ceiling"
    );
}

#[test]
fn throttle_below_the_floor_clamps_up() {
    let mut f = field();
    f.set_scalar("q_inf", vec![2_000.0]);
    f.set_scalar("propellant", vec![100.0]);
    f.set_throttle_action(0.05); // below the 0.1 floor
    burn_gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(f.throttle_action(), Some(0.1));
}

#[test]
fn propellant_floor_breach_refuses_not_clamps() {
    let mut f = field();
    f.set_scalar("q_inf", vec![2_000.0]);
    f.set_scalar("propellant", vec![5.0]); // at/below the 10 kg floor
    f.set_throttle_action(0.5); // positive throttle commanded
    let result = burn_gate().apply(&ctx(0), &mut f);
    assert!(result.is_err(), "a propellant-floor breach short-circuits");
    assert!(!f.log().is_empty(), "the breach is logged");
}

#[test]
fn descent_rate_bound_breach_returns_entropy() {
    let mut f = field();
    f.set_scalar("q_inf", vec![2_000.0]);
    f.set_scalar("propellant", vec![100.0]);
    f.set_scalar("descent_rate", vec![150.0]); // above the 100 m/s bound
    f.set_throttle_action(0.5);
    assert!(burn_gate().apply(&ctx(0), &mut f).is_err());
}

#[test]
fn burn_none_leaves_the_throttle_channel_untouched() {
    // With `burn: None` the gate never reads or writes the throttle channel: a throttle written
    // upstream survives verbatim and no burn-related provenance appears (the corridor gate path).
    let mut f = field();
    f.set_throttle_action(5.0); // an absurd command the burn gate would clamp — but burn is None
    f.set_control_action(0.2); // inside the bank envelope
    gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(
        f.throttle_action(),
        Some(5.0),
        "throttle untouched with burn: None"
    );
    assert_eq!(f.control_action(), Some(0.2));
    assert!(f.log().is_empty(), "no burn log traffic");
}

#[test]
fn burn_axes_without_a_throttle_command_are_inert() {
    // Burn axes present, but no throttle write: the throttle channel stays absent and no
    // burn-related log appears (only the ordinary bank path runs).
    let mut f = field();
    f.set_scalar("q_inf", vec![2_000.0]);
    f.set_control_action(0.2);
    burn_gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(
        f.throttle_action(),
        None,
        "no throttle command ⇒ no throttle write"
    );
    assert!(f.log().is_empty(), "no burn log traffic without a command");
}

// ---------------------------------------------------------------------------
// CyberneticCorrect — M4 live enforcement (powered-descent-envelope)
// ---------------------------------------------------------------------------

/// A field inside every burn axis, with the engine not yet lit.
fn burn_field() -> CoupledField<f64> {
    let mut f = field();
    f.set_scalar("q_inf", vec![2_000.0]);
    f.set_scalar("propellant", vec![100.0]);
    f
}

fn log_text(f: &CoupledField<f64>) -> String {
    f.log().messages().collect::<Vec<_>>().join(" | ")
}

#[test]
fn ignition_outside_the_dynamic_pressure_window_refuses() {
    let mut f = burn_field();
    f.set_scalar("q_inf", vec![200.0]); // below the 1000 Pa window floor
    f.set_throttle_action(0.5); // a throttle rising from zero: the engine is not lit
    let err = burn_gate().apply(&ctx(0), &mut f).unwrap_err();
    assert!(format!("{err:?}").contains("dynamic-pressure window"));
    assert!(log_text(&f).contains("ignition dynamic pressure"));
}

#[test]
fn ignition_inside_the_window_is_admitted() {
    let mut f = burn_field();
    f.set_scalar("q_inf", vec![1_500.0]);
    f.set_throttle_action(0.5);
    burn_gate()
        .apply(&ctx(0), &mut f)
        .expect("inside the window");
}

#[test]
fn the_window_does_not_bound_a_burn_already_under_way() {
    // The engine is lit, so `q∞` leaving the window as the vehicle decelerates is not an ignition
    // decision and must not refuse — the running axes bound a burn in progress, not this one.
    let mut f = burn_field();
    f.set_scalar("q_inf", vec![200.0]);
    f.set_scalar("ignited", vec![1.0]);
    f.set_throttle_action(0.5);
    burn_gate()
        .apply(&ctx(0), &mut f)
        .expect("a running burn is not re-gated on the ignition window");
}

#[test]
fn an_absent_dynamic_pressure_sensor_does_not_trip_the_window() {
    // Absent sensors read as zero and stay safe; the producer-side gap is the flight-sensor
    // stage's business, not a reason to fail closed here.
    let mut f = field();
    f.set_scalar("propellant", vec![100.0]);
    f.set_throttle_action(0.5);
    burn_gate()
        .apply(&ctx(0), &mut f)
        .expect("absent q is safe");
}

#[test]
fn a_crossed_throttle_window_refuses_rather_than_choosing_a_bound() {
    // ct_ceiling = q·7.85e-4; at q = 100 Pa that is 0.0785, below the 0.1 floor.
    let mut f = burn_field();
    f.set_scalar("q_inf", vec![100.0]);
    f.set_scalar("ignited", vec![1.0]); // isolate the crossed window from the ignition window
    f.set_throttle_action(0.5); // at/above the floor: the pre-change code clamped DOWN past it
    let err = burn_gate().apply(&ctx(0), &mut f).unwrap_err();
    assert!(format!("{err:?}").contains("throttle window crossed"));
}

#[test]
fn neither_crossed_window_branch_emits_an_out_of_envelope_throttle() {
    // The pre-change clamp tested its lower bound first, so a command BETWEEN the crossed bounds
    // was pushed UP past the C_T cap while one at or above the floor was pushed DOWN below the
    // floor. Both must now refuse, and neither may leave a bounded command behind.
    for commanded in [0.09_f64, 0.5] {
        let mut f = burn_field();
        f.set_scalar("q_inf", vec![100.0]);
        f.set_scalar("ignited", vec![1.0]);
        f.set_throttle_action(commanded);
        assert!(
            burn_gate().apply(&ctx(0), &mut f).is_err(),
            "commanded {commanded} must refuse on a crossed window"
        );
        assert_eq!(
            f.throttle_action(),
            Some(commanded),
            "a refused step leaves no bounded command"
        );
    }
}

#[test]
fn simultaneous_breaches_are_all_logged_and_the_first_is_returned() {
    // A heat breach used to return before the burn block was reached, so the propellant breach on
    // the same step was never logged. Both must appear; the error names the first in axis order.
    let mut f = burn_field();
    f.set_scalar("heat_flux", vec![2.0e6]); // above the 1e6 ceiling
    f.set_scalar("propellant", vec![5.0]); // at/below the 10 kg floor
    f.set_scalar("ignited", vec![1.0]);
    f.set_throttle_action(0.5);

    let err = burn_gate().apply(&ctx(0), &mut f).unwrap_err();
    let log = log_text(&f);
    assert!(log.contains("no recoverable bank correction"), "log: {log}");
    assert!(log.contains("propellant"), "log: {log}");
    assert!(format!("{err:?}").contains("no recoverable bank correction"));
}

#[test]
fn the_returned_error_is_deterministic_across_runs() {
    let build = || {
        let mut f = burn_field();
        f.set_scalar("heat_flux", vec![2.0e6]);
        f.set_scalar("descent_rate", vec![150.0]);
        f.set_scalar("ignited", vec![1.0]);
        f.set_throttle_action(0.5);
        f
    };
    let mut a = build();
    let mut b = build();
    let ea = burn_gate().apply(&ctx(0), &mut a).unwrap_err();
    let eb = burn_gate().apply(&ctx(0), &mut b).unwrap_err();
    assert_eq!(format!("{ea:?}"), format!("{eb:?}"));
}

#[test]
fn scalar_driven_thrust_with_burn_axes_attached_refuses() {
    // The propulsion stages honour the published scalar; the gate senses the channel alone. A world
    // driving only the scalar would otherwise fly its full propulsion path unenforced.
    let mut f = burn_field();
    f.set_scalar("commanded_throttle", vec![0.7]);
    // No `set_throttle_action`: the channel is absent.
    let err = burn_gate().apply(&ctx(0), &mut f).unwrap_err();
    assert!(format!("{err:?}").contains("not on the throttle channel"));
    assert!(log_text(&f).contains("cannot enforce a throttle it cannot see"));
}

#[test]
fn the_gate_never_writes_the_channel_from_a_scalar_source() {
    // Writing the channel here would outrank the world's published constant on every later step,
    // freezing a counterfactual branch at its first clamped value.
    let mut f = burn_field();
    f.set_scalar("commanded_throttle", vec![0.7]);
    let _ = burn_gate().apply(&ctx(0), &mut f);
    assert_eq!(
        f.throttle_action(),
        None,
        "the counterfactual seam must survive the gate"
    );
}

#[test]
fn a_zero_published_scalar_with_burn_axes_stays_silent() {
    // Neither seam driven: the inactive-axes guarantee holds.
    let mut f = burn_field();
    f.set_scalar("commanded_throttle", vec![0.0]);
    f.set_control_action(0.2);
    burn_gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert!(f.log().is_empty(), "no new log traffic: {}", log_text(&f));
}

#[test]
fn burn_axes_without_burn_sensing_refuse_rather_than_silently_disabling_the_cap() {
    let envelope = SafetyEnvelope::new(1.0e6, 12.0, 0.5).with_burn(BurnEnvelope::new(
        0.1, 0.9, 2.0, 1000.0, 5000.0, 10.0, 100.0,
    ));
    // No `with_burn_sensing`: thrust_ref and s_ref stay zero, so the dynamic C_T cap cannot bind.
    let gate = CyberneticCorrect::new(envelope);
    let mut f = burn_field();
    f.set_scalar("ignited", vec![1.0]);
    f.set_throttle_action(0.5);

    let err = gate.apply(&ctx(0), &mut f).unwrap_err();
    assert!(format!("{err:?}").contains("with_burn_sensing"));
}

#[test]
fn a_commanded_shutdown_is_not_clamped_up_to_the_throttle_floor() {
    // The floor is a stability constraint for a running engine — below it the central-nozzle
    // jet-penetration flow is unsteady — so it bounds how softly a burn may run, not whether one
    // must run at all. Clamping a commanded zero up to the floor would light the engine.
    let mut f = burn_field();
    f.set_throttle_action(0.0);
    burn_gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(f.throttle_action(), Some(0.0));
    assert!(f.log().is_empty(), "a shutdown needs no bounding entry");
}

#[test]
fn a_positive_throttle_below_the_floor_still_clamps_up() {
    // The shutdown carve-out must not weaken the floor for a genuinely commanded burn.
    let mut f = burn_field();
    f.set_throttle_action(0.02);
    burn_gate().apply(&ctx(0), &mut f).expect("gate applies");
    assert_eq!(f.throttle_action(), Some(0.1));
}
