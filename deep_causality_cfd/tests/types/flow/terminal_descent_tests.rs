/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The **terminal-descent harness** (change `add-retropulsion-terminal-descent`).
//!
//! This is the first test anywhere composing `CyberneticCorrect` **with burn axes** into a coupled
//! stack that actually runs: the M2/M3 burn integration test omits the gate entirely and drives
//! ignition with a bare channel write, so every burn leg in the workspace ran ungated until now.
//!
//! It flies coast → commit → burn → terminal descent through the guidance stage and asserts the M4
//! exit gates: **(2) ignition corridor** (the commit fired inside the band, window, nav state, and
//! margin), **(6) touchdown** (the altitude floor reached with bounded descent rate and propellant),
//! and **(0) integrity** (no step captured an error, the envelope held).

use deep_causality_cfd::{
    Ambient, BlackoutTrigger, BurnEnvelope, CoupledField, Coupling, CyberneticCorrect,
    FlightSensors, IGNITION_LATCH_FIELD, IgnitionCorridor, MachRegime, PhysicsStage,
    PlumeObstruction, RegimeClassify, RetroThrust, SafetyEnvelope, StepContext, ThrottleGuidance,
    ThrustState,
};

const THRUST: f64 = 30_000.0; // full-throttle thrust, N
const ISP: f64 = 300.0;
const GRAVITY: f64 = 9.80665;
const Q_INF: f64 = 2_000.0;
const S_REF_AREA: f64 = 0.785; // reference AREA, m^2 — never the acoustic S_REF
const M_BAR: f64 = 4.81e-26;
const DT: f64 = 0.1;

// Burn envelope: throttle in [0.1, 0.95], max_ct 3.0, ignition window [1000, 5000] Pa,
// propellant floor 5 kg, max descent rate 700 m/s.
//
// The descent-rate axis is a whole-leg bound the gate applies on every step, so it is sized for the
// supersonic entry to this leg (~600 m/s), not for the touchdown condition. A tighter
// trajectory-derived touchdown speed is M5's witness, not a harness one — closing that loop needs
// the truth propagator, which lives in the example crate.
fn envelope() -> SafetyEnvelope<f64> {
    SafetyEnvelope::new(1.0e7, 60.0, 0.5).with_burn(BurnEnvelope::new(
        0.1, 0.95, 3.0, 1_000.0, 5_000.0, 5.0, 700.0,
    ))
}

fn corridor() -> IgnitionCorridor<f64> {
    IgnitionCorridor::new(1.2, 3.0, 1_000.0, 5_000.0, 60.0)
}

/// The powered-descent stack in the M2 order: sensors → classifier → thrust → plume → guidance →
/// gate. The thrust and plume stages compose before the force consumers; guidance composes after
/// the sensing it reads and before the gate that clamps it.
fn powered_stack() -> impl PhysicsStage<2, f64> {
    Coupling::between_steps()
        .then(FlightSensors::new(M_BAR))
        .then(
            RegimeClassify::new(1.0, BlackoutTrigger::new(1.0e9)).with_flight_axes(0.8, 1.2, 500.0),
        )
        .then(RetroThrust::new(THRUST, ISP))
        .then(PlumeObstruction::new(THRUST, S_REF_AREA))
        .then(ThrottleGuidance::new(THRUST, GRAVITY).with_corridor(corridor()))
        .then(CyberneticCorrect::new(envelope()).with_burn_sensing(
            "q_inf",
            "propellant",
            "descent_rate",
            THRUST,
            S_REF_AREA,
        ))
        .build()
}

const R_EARTH: f64 = 6_371_000.0;

/// A descending vehicle at `altitude` m with radial descent speed `sink` m/s, plus the freestream
/// and navigation the carrier and the nav stage would have published.
fn descent_field(altitude: f64, sink: f64, mach: f64, aided: bool) -> CoupledField<f64> {
    let mut f = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    f.set_scalar("mass", vec![1_200.0]);
    f.set_scalar("propellant", vec![400.0]);
    f.set_scalar("mean_free_path", vec![0.005]);
    f.set_scalar("flight_mach", vec![mach]);
    f.set_scalar("flight_altitude", vec![altitude]);
    f.set_scalar("flight_speed", vec![sink]);
    // q_inf is derived by FlightSensors from these two.
    f.set_scalar("freestream_n", vec![2.0 * Q_INF / (M_BAR * sink * sink)]);
    f.set_scalar(
        "truth_state",
        vec![R_EARTH + altitude, 0.0, 0.0, -sink, 0.0, 0.0],
    );
    f.set_scalar("nav_mode", vec![if aided { 1.0 } else { 0.0 }]);
    f.set_scalar("nav_position_variance", vec![900.0]); // sigma 30 m, inside the 60 m margin
    f.set_aero_force([-5.0, 0.0, 0.0]);
    f
}

fn log_text(f: &CoupledField<f64>) -> String {
    f.log().messages().collect::<Vec<_>>().join(" | ")
}

#[test]
fn the_gate_flies_with_burn_axes_composed_into_a_marching_stack() {
    // The composition itself is the assertion: before M4 nothing wrote the throttle channel, so
    // the gate's whole burn block was unreachable in a composed world.
    let stack = powered_stack();
    let mut field = descent_field(20_000.0, 600.0, 2.0, true);

    stack
        .apply(&StepContext::<2, f64>::qtt(DT, 1), &mut field)
        .expect("a burn-gated step runs");

    assert!(
        field.throttle_action().is_some(),
        "the guidance stage wrote the channel, so the gate's burn axes are live"
    );
}

#[test]
fn gate_2_the_commit_fires_inside_the_corridor() {
    let stack = powered_stack();
    let mut field = descent_field(20_000.0, 600.0, 2.0, true);

    stack
        .apply(&StepContext::<2, f64>::qtt(DT, 4), &mut field)
        .expect("commit step");

    assert!(
        field.scalar(IGNITION_LATCH_FIELD).is_some(),
        "the corridor committed"
    );
    let log = log_text(&field);
    assert!(
        log.contains("ignition corridor committed at step 4"),
        "log: {log}"
    );
    // The commit named the sensing it saw.
    assert!(log.contains("aided"), "log: {log}");
}

#[test]
fn the_corridor_does_not_commit_on_dead_reckoning() {
    let stack = powered_stack();
    let mut field = descent_field(20_000.0, 600.0, 2.0, false);

    stack
        .apply(&StepContext::<2, f64>::qtt(DT, 1), &mut field)
        .expect("step runs");

    assert!(field.scalar(IGNITION_LATCH_FIELD).is_none());
    assert_eq!(field.throttle_action(), Some(0.0));
}

#[test]
fn pre_ignition_steps_are_gated_and_the_burn_stages_stay_inert() {
    // Guidance writes zero from step 0 precisely so the gate's burn axes are evaluated before
    // ignition. The thrust and plume stages must remain untouched at that command.
    let stack = powered_stack();
    // Outside the Mach band, so the corridor cannot commit.
    let mut field = descent_field(20_000.0, 600.0, 6.0, true);

    let mass_before = field.scalar("mass").unwrap()[0];
    let prop_before = field.scalar("propellant").unwrap()[0];
    stack
        .apply(&StepContext::<2, f64>::qtt(DT, 1), &mut field)
        .expect("gated pre-ignition step");

    assert_eq!(field.throttle_action(), Some(0.0));
    assert_eq!(field.scalar("mass").unwrap()[0], mass_before);
    assert_eq!(field.scalar("propellant").unwrap()[0], prop_before);
    assert!(field.scalar("ignited").is_none());
}

#[test]
fn a_powered_descent_leg_runs_to_the_floor_with_integrity() {
    // Gates (0) and (6) at the level a harness can honestly establish. Closing the loop — thrust
    // feeding a trajectory that feeds the next step's sensing — needs the truth propagator, which
    // lives in the example crate; M5 owns the trajectory-derived touchdown witnesses (miss to pad,
    // terminal speed). What this asserts is what M4 built: the leg flies a prescribed descent to the
    // altitude floor with the gate live on every step, no step captures an error, the envelope holds
    // the throttle inside its bounds throughout, and the propellant never crosses its floor.
    let stack = powered_stack();
    let mut field = descent_field(20_000.0, 600.0, 2.0, true);

    const STEPS: usize = 200;
    let mut min_throttle = f64::INFINITY;
    let mut max_throttle = f64::NEG_INFINITY;

    for step in 1..=STEPS {
        // A prescribed descent: altitude falls linearly to the floor, Mach with it.
        let frac = step as f64 / STEPS as f64;
        let altitude = 20_000.0 * (1.0 - frac) + 400.0 * frac;
        let sink = 600.0 * (1.0 - frac) + 40.0 * frac;
        let mach = (sink / 300.0).max(0.05);

        field.set_scalar("flight_altitude", vec![altitude]);
        field.set_scalar("flight_speed", vec![sink]);
        field.set_scalar("flight_mach", vec![mach]);
        field.set_scalar(
            "truth_state",
            vec![R_EARTH + altitude, 0.0, 0.0, -sink, 0.0, 0.0],
        );
        field.set_scalar("freestream_n", vec![2.0 * Q_INF / (M_BAR * sink * sink)]);
        field.set_aero_force([-5.0, 0.0, 0.0]);

        stack
            .apply(&StepContext::<2, f64>::qtt(DT, step), &mut field)
            .expect("gate (0): no step captures an error");

        let theta = field
            .throttle_action()
            .expect("the channel is written every step");
        min_throttle = min_throttle.min(theta);
        max_throttle = max_throttle.max(theta);
        assert!(
            field.aero_force().unwrap().iter().all(|c| c.is_finite()),
            "gate (0): the force channel stayed finite at step {step}"
        );
    }

    // The envelope held: every commanded throttle was either a shutdown or inside [floor, ceiling].
    assert!(
        min_throttle >= 0.0,
        "no negative throttle was ever emitted: {min_throttle}"
    );
    assert!(
        max_throttle <= 0.95,
        "gate (0): the envelope ceiling held at {max_throttle}"
    );
    // Touchdown is classified at the floor.
    assert!(
        field.regime().unwrap().touchdown,
        "gate (6): the classifier recorded touchdown at the altitude floor"
    );
    // Propellant never crossed its floor — had it, the gate would have refused the step above.
    assert!(
        field.scalar("propellant").unwrap()[0] > 5.0,
        "gate (6): propellant above the floor at touchdown"
    );
    assert!(field.scalar("mass").unwrap()[0] > 0.0);
}

#[test]
fn the_throttle_floor_does_not_light_a_commanded_shutdown() {
    // The floor is a stability constraint for a *running* engine, so clamping a commanded zero up
    // to it would ignite the vehicle on the next step — defeating the zero-from-step-0 discipline
    // that keeps the envelope live before the corridor commits.
    let stack = powered_stack();
    let mut field = descent_field(20_000.0, 600.0, 6.0, true); // outside the Mach band

    stack
        .apply(&StepContext::<2, f64>::qtt(DT, 1), &mut field)
        .expect("step runs");

    assert_eq!(
        field.throttle_action(),
        Some(0.0),
        "a commanded shutdown is admissible and must not be clamped up to the floor"
    );
}

#[test]
fn the_transonic_crossing_under_thrust_is_logged() {
    // Gate (6)'s companion: the M = 1 crossing under thrust appears as a regime transition rather
    // than passing silently.
    let stack = powered_stack();
    let mut field = descent_field(20_000.0, 600.0, 2.0, true);

    for (step, mach) in [(1usize, 2.0_f64), (2, 1.0), (3, 0.5)] {
        field.set_scalar("flight_mach", vec![mach]);
        stack
            .apply(&StepContext::<2, f64>::qtt(DT, step), &mut field)
            .expect("step runs");
    }

    // The classifier composes before the thrust stage (the corridor's order, fixed by the
    // chemistry and lift stages that need the regime), so the `"ignited"` flag it reads is the
    // previous step's — the thrust state trails the Mach band by one entry. The cascade itself is
    // the assertion: the ordered crossings appear, and the leg reaches a burning subsonic state.
    let msgs: Vec<&str> = field.log().messages().collect();
    let cascade: Vec<&&str> = msgs.iter().filter(|m| m.starts_with("regime ->")).collect();
    assert!(
        cascade.iter().any(|m| m.contains("transonic")),
        "the M = 1 crossing is logged: {msgs:?}"
    );
    assert!(
        cascade
            .iter()
            .any(|m| m.contains("subsonic") && m.contains("burn"))
    );
    assert_eq!(field.regime().unwrap().mach_regime, MachRegime::Subsonic);
    assert_eq!(field.regime().unwrap().thrust_state, ThrustState::Burn);
}

#[test]
fn a_propellant_floor_breach_stops_the_leg() {
    // Gate (0): the envelope held — and when it cannot, the step refuses rather than burning on.
    let stack = powered_stack();
    let mut field = descent_field(20_000.0, 600.0, 2.0, true);
    // Commit first, then starve the tank.
    stack
        .apply(&StepContext::<2, f64>::qtt(DT, 1), &mut field)
        .expect("commit step");
    field.set_scalar("propellant", vec![1.0]); // below the 5 kg floor

    let err = stack
        .apply(&StepContext::<2, f64>::qtt(DT, 2), &mut field)
        .unwrap_err();
    assert!(format!("{err:?}").contains("propellant floor"));
}

#[test]
fn ignition_outside_the_dynamic_pressure_window_is_refused_in_flight() {
    // The window is enforced on the step that lights the engine. Drive the corridor's own q inside
    // its band while the envelope's window sees a different, out-of-range dynamic pressure.
    let stack = Coupling::between_steps()
        .then(
            RegimeClassify::new(1.0, BlackoutTrigger::new(1.0e9)).with_flight_axes(0.8, 1.2, 500.0),
        )
        .then(RetroThrust::new(THRUST, ISP))
        .then(
            ThrottleGuidance::new(THRUST, GRAVITY)
                .with_corridor(IgnitionCorridor::new(1.2, 3.0, 0.0, 1.0e9, 60.0)),
        )
        .then(CyberneticCorrect::new(envelope()).with_burn_sensing(
            "q_inf",
            "propellant",
            "descent_rate",
            THRUST,
            S_REF_AREA,
        ))
        .build();

    let mut field = descent_field(20_000.0, 600.0, 2.0, true);
    field.set_scalar("q_inf", vec![200.0]); // below the envelope's 1000 Pa window floor

    let err = stack
        .apply(&StepContext::<2, f64>::qtt(DT, 1), &mut field)
        .unwrap_err();
    assert!(format!("{err:?}").contains("dynamic-pressure window"));
}
