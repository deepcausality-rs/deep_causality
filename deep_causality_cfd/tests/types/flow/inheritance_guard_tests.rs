/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Prong B of the corridor-inheritance guard (`corridor-inheritance-guard`): the inert
//! `PropulsionStub` is a no-op in a multi-step coupled march at zero throttle, extending the landed
//! marcher-path bit-identity pattern (`unforced_carrier_matches_the_bare_marcher_bit_for_bit`) to
//! the stage layer. This is the tested meaning of "strictly inert at zero throttle": the burn-phase
//! stack can carry the propulsion stages from the start, and ignition stays a published-command
//! event rather than a stack swap.

use deep_causality_cfd::{
    AeroForceCoupling, Ambient, CoupledField, Coupling, CyberneticCorrect, PhysicsError,
    PhysicsStage, PropulsionStub, SafetyEnvelope, StepContext, ThermalRelax, ViscosityArrhenius,
};

/// A corridor-class between-step coupling: thermal relaxation → temperature-driven viscosity →
/// a flow-derived aero force → the cybernetic bank gate. Representative of the corridor stack's
/// shape (scalars + ambient + force channel + gate), without the full example wiring.
fn corridor_stack() -> impl PhysicsStage<2, f64> {
    Coupling::between_steps()
        .then(ThermalRelax::new(0.5, 400.0))
        .then(ViscosityArrhenius::new(0.01, 300.0, 2.0))
        .then(AeroForceCoupling::new(1.2, 3.0e-4))
        .then(CyberneticCorrect::new(SafetyEnvelope::new(
            1.0e9, 100.0, 0.5,
        )))
        .build()
}

/// The seed carried state: a temperature field, a flow speed, and the propulsion scalars a
/// burn-phase world rides (so the stub has real state it must not touch), plus a bank command.
fn seed_field() -> CoupledField<f64> {
    let mut f = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    f.set_scalar("temperature", vec![300.0, 320.0, 280.0]);
    f.set_scalar("speed", vec![1_500.0, 1_400.0]);
    f.set_scalar("mass", vec![1_000.0]);
    f.set_scalar("propellant", vec![200.0]);
    f.set_control_action(0.2);
    f
}

/// March a coupling stack `steps` times over the seed field (the per-step loop `run_coupled` runs).
fn march<S: PhysicsStage<2, f64>>(stack: &S, steps: usize) -> CoupledField<f64> {
    let mut field = seed_field();
    for step in 0..steps {
        let ctx = StepContext::<2, f64>::qtt(0.1, step);
        stack.apply(&ctx, &mut field).expect("coupled step");
    }
    field
}

/// Assert two coupled fields are bit-identical across every artifact the guard compares.
fn assert_bit_identical(a: &CoupledField<f64>, b: &CoupledField<f64>) {
    assert_eq!(a.scalars().len(), b.scalars().len(), "scalar count");
    for ((na, va), (nb, vb)) in a.scalars().iter().zip(b.scalars()) {
        assert_eq!(na, nb, "scalar name order");
        assert_eq!(va.len(), vb.len(), "scalar '{na}' length");
        for (x, y) in va.iter().zip(vb) {
            assert_eq!(x.to_bits(), y.to_bits(), "scalar '{na}' bit-identical");
        }
    }
    assert_eq!(a.aero_force(), b.aero_force(), "force channel");
    assert_eq!(a.control_action(), b.control_action(), "bank channel");
    assert_eq!(a.throttle_action(), b.throttle_action(), "throttle channel");
    assert_eq!(a.regime(), b.regime(), "regime");
    assert_eq!(a.log(), b.log(), "provenance log");
}

#[test]
fn stub_at_zero_throttle_is_invisible_over_a_coupled_march() {
    let steps = 5;
    let plain = march(&corridor_stack(), steps);

    // The same stack with the propulsion stub composed at zero throttle (no throttle command).
    let with_stub = Coupling::between_steps()
        .then(ThermalRelax::new(0.5, 400.0))
        .then(ViscosityArrhenius::new(0.01, 300.0, 2.0))
        .then(AeroForceCoupling::new(1.2, 3.0e-4))
        .then(CyberneticCorrect::new(SafetyEnvelope::new(
            1.0e9, 100.0, 0.5,
        )))
        .then(PropulsionStub::new(2_000.0, 250.0, 2_800.0, 0.785))
        .build();
    let stubbed = march(&with_stub, steps);

    assert_bit_identical(&plain, &stubbed);
}

/// A deliberately non-inert stage: writes a scalar every step regardless of throttle.
struct NoisyStub;

impl PhysicsStage<2, f64> for NoisyStub {
    fn apply(
        &self,
        _ctx: &StepContext<'_, 2, f64>,
        field: &mut CoupledField<f64>,
    ) -> Result<(), PhysicsError> {
        field.set_scalar("ignited", alloc_vec_one());
        Ok(())
    }
}

fn alloc_vec_one() -> Vec<f64> {
    vec![1.0]
}

#[test]
fn the_guard_detects_a_non_inert_stage() {
    // The negative control: a stage that writes at zero throttle diverges from the plain stack, so
    // the guard's bit-identity comparison would fail — proving it detects a non-inert regression.
    let steps = 3;
    let plain = march(&corridor_stack(), steps);

    let noisy = Coupling::between_steps()
        .then(ThermalRelax::new(0.5, 400.0))
        .then(ViscosityArrhenius::new(0.01, 300.0, 2.0))
        .then(AeroForceCoupling::new(1.2, 3.0e-4))
        .then(CyberneticCorrect::new(SafetyEnvelope::new(
            1.0e9, 100.0, 0.5,
        )))
        .then(NoisyStub)
        .build();
    let noisy_field = march(&noisy, steps);

    // The discriminating artifact the guard checks differs: plain never sets "ignited".
    assert_eq!(plain.scalar("ignited"), None);
    assert_eq!(noisy_field.scalar("ignited"), Some([1.0].as_slice()));
}

// ---------------------------------------------------------------------------
// M3 burn-leg cascade harness (flight-regime-classifier, exit gate)
// ---------------------------------------------------------------------------

use deep_causality_cfd::{
    BlackoutTrigger, MachRegime, PlumeObstruction, RegimeClassify, RetroThrust, ThrustState,
};

const BURN_THRUST: f64 = 2_000.0;
const BURN_ISP: f64 = 250.0;
const BURN_Q_INF: f64 = 2_800.0;
const BURN_S_REF: f64 = 0.785;

/// The burn-phase coupling: thrust → plume (A0 decrement) → the flight-regime classifier.
fn burn_stack() -> impl PhysicsStage<2, f64> {
    Coupling::between_steps()
        .then(RetroThrust::new(BURN_THRUST, BURN_ISP))
        .then(PlumeObstruction::new(BURN_THRUST, BURN_S_REF))
        .then(
            RegimeClassify::new(1.0, BlackoutTrigger::new(1.0e9)).with_flight_axes(0.8, 1.2, 10.0),
        )
        .build()
}

/// A burn-phase field: propulsion state plus the carrier-published flight scalars.
fn burn_field(mach: f64, altitude: f64) -> CoupledField<f64> {
    let mut f = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    f.set_scalar("mass", vec![1_000.0]);
    f.set_scalar("propellant", vec![200.0]);
    f.set_scalar("mean_free_path", vec![0.005]);
    f.set_scalar("flight_mach", vec![mach]);
    f.set_scalar("flight_altitude", vec![altitude]);
    f.set_aero_force([-5.0, 0.0, 0.0]);
    // Retro thrust is aimed against the carried flight velocity.
    f.set_scalar("truth_state", vec![6.4e6, 0.0, 0.0, 800.0, 0.0, 0.0]);
    // The plume stage senses its freestream each step rather than carrying a constant.
    f.set_scalar("q_inf", vec![BURN_Q_INF]);
    f.set_scalar("p_inf", vec![1_000.0]);
    f
}

#[test]
fn a_burn_leg_emits_the_ordered_regime_cascade_with_integrity() {
    // Coast (supersonic, unlit) → light the engine → decelerate through the Mach bands. Every
    // transition must land in provenance in order, and no step may capture an error.
    let stack = burn_stack();
    let mut field = burn_field(2.5, 50_000.0);

    // Step 0 — coasting, supersonic: the stages are inert (no throttle), the classifier records.
    stack
        .apply(&StepContext::<2, f64>::qtt(0.1, 0), &mut field)
        .expect("coast step");
    let coast = field.regime().expect("classified");
    assert_eq!(
        coast.thrust_state,
        ThrustState::Unknown,
        "no propulsion state before ignition"
    );
    assert_eq!(coast.mach_regime, MachRegime::Supersonic);

    // Step 1 — ignition: a published throttle lights the burn (RetroThrust sets "ignited").
    field.set_throttle_action(0.5);
    stack
        .apply(&StepContext::<2, f64>::qtt(0.1, 1), &mut field)
        .expect("ignition step");
    assert_eq!(field.regime().unwrap().thrust_state, ThrustState::Burn);

    // Steps 2-3 — decelerate under thrust across the Mach bands.
    field.set_scalar("flight_mach", vec![1.0]);
    stack
        .apply(&StepContext::<2, f64>::qtt(0.1, 2), &mut field)
        .expect("transonic step");
    field.set_scalar("flight_mach", vec![0.5]);
    stack
        .apply(&StepContext::<2, f64>::qtt(0.1, 3), &mut field)
        .expect("subsonic step");

    // The ordered cascade: supersonic-coast → burn → transonic → subsonic.
    let msgs: Vec<&str> = field.log().messages().collect();
    let cascade: Vec<&&str> = msgs.iter().filter(|m| m.starts_with("regime ->")).collect();
    assert!(cascade.len() >= 4, "each phase transition logged: {msgs:?}");
    assert!(
        cascade[0].contains("supersonic"),
        "opens supersonic: {}",
        cascade[0]
    );
    assert!(
        cascade[1].contains("burn"),
        "ignition is a transition: {}",
        cascade[1]
    );
    assert!(
        cascade[2].contains("transonic"),
        "Mach crossing under thrust: {}",
        cascade[2]
    );
    assert!(cascade[3].contains("subsonic"), "and again: {}", cascade[3]);

    // Integrity: the propulsion state stayed physical across the leg.
    assert!(field.scalar("mass").unwrap()[0] > 0.0);
    assert!(field.scalar("propellant").unwrap()[0] > 0.0);
    assert!(field.aero_force().unwrap().iter().all(|c| c.is_finite()));
}

#[test]
fn the_in_flight_decrement_stays_inside_the_m1_band() {
    // Gate 4b in flight: the applied fraction is the cited correlation, not a re-derived number.
    let stack = burn_stack();
    let mut field = burn_field(2.5, 50_000.0);
    field.set_throttle_action(0.5);
    stack
        .apply(&StepContext::<2, f64>::qtt(0.1, 0), &mut field)
        .expect("burn step");

    let applied = field.scalar("preserved_drag_fraction").expect("published")[0];
    let c_t = deep_causality_physics::srp_thrust_coefficient_kernel(
        deep_causality_physics::Force::new(0.5 * BURN_THRUST).unwrap(),
        deep_causality_physics::Pressure::new(BURN_Q_INF).unwrap(),
        deep_causality_physics::Area::new(BURN_S_REF).unwrap(),
    )
    .unwrap();
    let expected = deep_causality_physics::srp_preserved_drag_fraction_kernel(c_t).unwrap();
    assert!(
        (applied - expected).abs() < 1e-12,
        "the A0 correlation is the authority"
    );
}
