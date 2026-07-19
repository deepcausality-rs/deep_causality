/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The terminal-guidance stage and its ignition-corridor commit (`throttle-guidance-stage`,
//! `ignition-corridor-commit`): the stopping-distance law and its saturation, zero-from-step-0 so
//! the envelope stays live, the four-condition conjunction, the rising edge and one-way latch, and
//! the published-scalar navigation read.

use deep_causality_cfd::{
    Ambient, CoupledField, IGNITION_LATCH_FIELD, IgnitionCorridor, PhysicsStage, StepContext,
    ThrottleGuidance,
};
use deep_causality_physics::{PhysicsErrorEnum, SolenoidalField, VelocityOneForm};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const THRUST: f64 = 60_000.0; // full-throttle thrust, N
const GRAVITY: f64 = 9.80665;
const MASS: f64 = 1_000.0;
const DT: f64 = 0.1;

// A corridor the nominal field satisfies.
fn corridor() -> IgnitionCorridor<f64> {
    IgnitionCorridor::new(1.5, 3.0, 1_000.0, 6_000.0, 50.0)
}

fn empty_context() -> (Manifold<LatticeComplex<2, f64>, f64>, SolenoidalField<f64>) {
    let n = 4;
    let lattice = LatticeComplex::<2, f64>::new([n, n], [true, true]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric = CubicalReggeGeometry::<2, f64>::uniform(1.0);
    let manifold = Manifold::from_cubical_with_metric(lattice, data, metric, 0);
    let n1 = manifold.complex().num_cells(1);
    let zero = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let velocity = VelocityOneForm::from_raw(zero);
    let (state, _) = SolenoidalField::from_leray_projection(&velocity, &manifold).unwrap();
    (manifold, state)
}

/// A field inside the corridor on every axis.
fn committed_field() -> CoupledField<f64> {
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("flight_mach", vec![2.0]);
    field.set_scalar("q_inf", vec![3_000.0]);
    field.set_scalar("nav_mode", vec![1.0]);
    field.set_scalar("nav_position_variance", vec![400.0]); // sigma = 20 m, inside the 50 m margin
    field.set_scalar("flight_speed", vec![600.0]);
    field.set_scalar("flight_altitude", vec![30_000.0]);
    field.set_scalar("mass", vec![MASS]);
    field
}

fn guidance() -> ThrottleGuidance<f64> {
    ThrottleGuidance::new(THRUST, GRAVITY).with_corridor(corridor())
}

fn log_has(field: &CoupledField<f64>, needle: &str) -> bool {
    field.log().messages().any(|m| m.contains(needle))
}

#[test]
fn the_commanded_throttle_follows_the_stopping_distance_law() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = committed_field();

    PhysicsStage::<2, f64>::apply(&guidance(), &ctx, &mut field).unwrap();

    // a_cmd = v^2 / 2h + g; theta = m * a_cmd / T_full
    let a_cmd = 600.0_f64 * 600.0 / (2.0 * 30_000.0) + GRAVITY;
    let expected = MASS * a_cmd / THRUST;
    let theta = field.throttle_action().unwrap();
    assert!(
        (theta - expected).abs() < 1e-12,
        "theta {theta} vs expected {expected}"
    );
}

#[test]
fn the_throttle_saturates_at_one() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = committed_field();
    // A very low altitude demands far more deceleration than the engine can supply.
    field.set_scalar("flight_altitude", vec![50.0]);

    PhysicsStage::<2, f64>::apply(&guidance(), &ctx, &mut field).unwrap();

    assert_eq!(field.throttle_action(), Some(1.0));
}

#[test]
fn guidance_commands_zero_before_commit_but_still_writes_the_channel() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = committed_field();
    // Outside the Mach band: the corridor does not hold.
    field.set_scalar("flight_mach", vec![8.0]);

    PhysicsStage::<2, f64>::apply(&guidance(), &ctx, &mut field).unwrap();

    // The channel MUST be present so the gate's burn axes are evaluated pre-ignition.
    assert_eq!(
        field.throttle_action(),
        Some(0.0),
        "the channel must be written with zero, not left absent"
    );
    assert!(field.scalar(IGNITION_LATCH_FIELD).is_none());
}

#[test]
fn a_stage_without_a_corridor_never_ignites_but_keeps_the_channel_live() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = committed_field();

    let bare = ThrottleGuidance::new(THRUST, GRAVITY);
    PhysicsStage::<2, f64>::apply(&bare, &ctx, &mut field).unwrap();

    assert_eq!(field.throttle_action(), Some(0.0));
    assert!(field.scalar(IGNITION_LATCH_FIELD).is_none());
}

#[test]
fn all_four_conditions_holding_commits_and_logs() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 7);
    let mut field = committed_field();

    PhysicsStage::<2, f64>::apply(&guidance(), &ctx, &mut field).unwrap();

    assert!(field.scalar(IGNITION_LATCH_FIELD).is_some());
    assert!(field.throttle_action().unwrap() > 0.0);
    assert!(log_has(&field, "ignition corridor committed at step 7"));
}

#[test]
fn each_condition_short_leaves_the_corridor_uncommitted() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);

    // Mach outside the band.
    let mut mach = committed_field();
    mach.set_scalar("flight_mach", vec![0.5]);
    // Dynamic pressure outside the window.
    let mut q = committed_field();
    q.set_scalar("q_inf", vec![100.0]);
    // Dead reckoning.
    let mut nav = committed_field();
    nav.set_scalar("nav_mode", vec![0.0]);
    // Uncertainty outside the margin: sigma = 100 m against a 50 m margin.
    let mut margin = committed_field();
    margin.set_scalar("nav_position_variance", vec![10_000.0]);

    for (name, field) in [
        ("mach", &mut mach),
        ("q", &mut q),
        ("nav", &mut nav),
        ("margin", &mut margin),
    ] {
        PhysicsStage::<2, f64>::apply(&guidance(), &ctx, field).unwrap();
        assert_eq!(
            field.throttle_action(),
            Some(0.0),
            "{name} short must leave the throttle at zero"
        );
        assert!(
            field.scalar(IGNITION_LATCH_FIELD).is_none(),
            "{name} short must not latch"
        );
        assert!(!log_has(field, "ignition corridor committed"));
    }
}

#[test]
fn an_absent_sensor_is_a_condition_not_met() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = committed_field();
    field.take_scalar("nav_position_variance");

    PhysicsStage::<2, f64>::apply(&guidance(), &ctx, &mut field).unwrap();

    assert_eq!(field.throttle_action(), Some(0.0));
    assert!(field.scalar(IGNITION_LATCH_FIELD).is_none());
}

#[test]
fn the_commit_fires_exactly_once_across_many_satisfying_steps() {
    let (manifold, state) = empty_context();
    let mut field = committed_field();
    let g = guidance();

    for step in 1..=5 {
        let ctx = StepContext::new(&manifold, &state, DT, step);
        PhysicsStage::<2, f64>::apply(&g, &ctx, &mut field).unwrap();
    }

    let commits = field
        .log()
        .messages()
        .filter(|m| m.contains("ignition corridor committed"))
        .count();
    assert_eq!(commits, 1, "the commit is a rising edge, not a condition");
}

#[test]
fn the_latch_survives_a_condition_lapse() {
    let (manifold, state) = empty_context();
    let mut field = committed_field();
    let g = guidance();

    let ctx = StepContext::new(&manifold, &state, DT, 1);
    PhysicsStage::<2, f64>::apply(&g, &ctx, &mut field).unwrap();
    assert!(field.throttle_action().unwrap() > 0.0);

    // Navigation degrades to dead reckoning: the burn must not be extinguished.
    field.set_scalar("nav_mode", vec![0.0]);
    let ctx = StepContext::new(&manifold, &state, DT, 2);
    PhysicsStage::<2, f64>::apply(&g, &ctx, &mut field).unwrap();

    assert!(
        field.throttle_action().unwrap() > 0.0,
        "a transient nav dropout must not extinguish a committed burn"
    );
}

#[test]
fn the_latch_rides_the_field_so_it_crosses_a_leg_boundary() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = committed_field();
    PhysicsStage::<2, f64>::apply(&guidance(), &ctx, &mut field).unwrap();

    // A leg boundary carries the CoupledField and rebuilds the carrier; cloning the field is what
    // `MarchState` does. Carrier-internal state would be lost here — the latch must not be.
    let mut next_leg = field.clone();
    next_leg.set_scalar("flight_mach", vec![0.4]); // far outside the corridor now

    let ctx = StepContext::new(&manifold, &state, DT, 1);
    PhysicsStage::<2, f64>::apply(&guidance(), &ctx, &mut next_leg).unwrap();

    assert!(
        next_leg.throttle_action().unwrap() > 0.0,
        "the latch must survive the leg boundary the carrier state does not"
    );
}

#[test]
fn the_margin_is_compared_against_the_square_root_of_the_variance_trace() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);

    // Variance 2500 m^2 -> sigma 50 m, exactly at a 50 m margin: admitted.
    let mut at_margin = committed_field();
    at_margin.set_scalar("nav_position_variance", vec![2_500.0]);
    PhysicsStage::<2, f64>::apply(&guidance(), &ctx, &mut at_margin).unwrap();
    assert!(at_margin.scalar(IGNITION_LATCH_FIELD).is_some());

    // Variance 2601 m^2 -> sigma 51 m: refused. Compared against the raw trace it would pass,
    // which is exactly the units error this pins.
    let mut past_margin = committed_field();
    past_margin.set_scalar("nav_position_variance", vec![2_601.0]);
    PhysicsStage::<2, f64>::apply(&guidance(), &ctx, &mut past_margin).unwrap();
    assert!(past_margin.scalar(IGNITION_LATCH_FIELD).is_none());
}

#[test]
fn ground_contact_is_an_error_not_a_throttle() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = committed_field();
    field.set_scalar("flight_altitude", vec![0.0]);

    let err = PhysicsStage::<2, f64>::apply(&guidance(), &ctx, &mut field).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::Singularity(_)),
        "expected the kernel's singularity, got {err:?}"
    );
}

#[test]
fn a_committed_burn_without_mass_errors() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = committed_field();
    field.take_scalar("mass");

    let err = PhysicsStage::<2, f64>::apply(&guidance(), &ctx, &mut field).unwrap_err();
    assert!(matches!(
        err.0,
        PhysicsErrorEnum::PhysicalInvariantBroken(_)
    ));
}

#[test]
fn the_navigation_engine_is_never_touched() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = committed_field();

    // No engine is seeded; the commit must still evaluate from the published scalars alone.
    assert!(field.nav().is_none());
    PhysicsStage::<2, f64>::apply(&guidance(), &ctx, &mut field).unwrap();

    assert!(field.scalar(IGNITION_LATCH_FIELD).is_some());
    assert!(field.nav().is_none());
}

// ── The stopping burn: coast-then-burn, the target plane, and the contact speed ──────────────

/// A committed field low and slow enough that a stopping burn is a live question.
fn terminal_field(altitude: f64, speed: f64) -> CoupledField<f64> {
    let mut field = committed_field();
    field.set_scalar(IGNITION_LATCH_FIELD, vec![1.0]);
    field.set_scalar("flight_altitude", vec![altitude]);
    field.set_scalar("flight_speed", vec![speed]);
    field
}

#[test]
fn a_stopping_burn_coasts_above_the_ignition_altitude() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    // h_ign = v^2 / 2(a_T - g) = 50^2 / (2 * 50.2) ~ 24.9 m. At 400 m the burn has not started.
    let mut field = terminal_field(400.0, 50.0);
    let stage = guidance().with_stopping_burn(0.0);

    PhysicsStage::<2, f64>::apply(&stage, &ctx, &mut field).unwrap();

    assert_eq!(
        field.throttle_action(),
        Some(0.0),
        "a committed guidance above its ignition altitude must coast, not burn: the closed form \
         degenerates to a_cmd ~ g at large h, which hovers the vehicle instead of landing it"
    );
}

#[test]
fn a_stopping_burn_lights_at_the_ignition_altitude_and_latches() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = terminal_field(20.0, 50.0); // below h_ign ~ 24.9 m
    let stage = guidance().with_stopping_burn(0.0);

    PhysicsStage::<2, f64>::apply(&stage, &ctx, &mut field).unwrap();
    assert!(field.throttle_action().unwrap() > 0.0);
    assert!(log_has(&field, "stopping burn started"));

    // The latch is what keeps it lit: a_T rises as propellant burns off, so h_ign FALLS, and a
    // live predicate would find the vehicle above the new h_ign on the next step and shut down.
    field.set_scalar("flight_altitude", vec![400.0]);
    PhysicsStage::<2, f64>::apply(&stage, &ctx, &mut field).unwrap();
    assert!(
        field.throttle_action().unwrap() > 0.0,
        "a started stopping burn must stay lit; the decision is made once, like the corridor commit"
    );
}

#[test]
fn the_law_targets_the_commanded_plane_not_the_surface() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = terminal_field(100.0, 50.0);
    let stage = guidance().with_target_altitude(15.0);

    PhysicsStage::<2, f64>::apply(&stage, &ctx, &mut field).unwrap();

    // Height above the TARGET, not the geocenter: 100 - 15 = 85 m.
    let a_cmd = 50.0_f64 * 50.0 / (2.0 * 85.0) + GRAVITY;
    let expected = MASS * a_cmd / THRUST;
    let theta = field.throttle_action().unwrap();
    assert!(
        (theta - expected).abs() < 1e-12,
        "theta {theta} vs expected {expected}"
    );
}

#[test]
fn a_contact_speed_is_removed_from_the_energy_the_burn_must_shed() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    let mut field = terminal_field(100.0, 50.0);
    let stage = guidance().with_contact_speed(2.0);

    PhysicsStage::<2, f64>::apply(&stage, &ctx, &mut field).unwrap();

    // The burn sheds down to the contact speed, not to rest: v_eff^2 = v^2 - v_c^2.
    let a_cmd = (50.0_f64 * 50.0 - 4.0) / (2.0 * 100.0) + GRAVITY;
    let expected = MASS * a_cmd / THRUST;
    let theta = field.throttle_action().unwrap();
    assert!(
        (theta - expected).abs() < 1e-12,
        "theta {theta} vs expected {expected}"
    );
}

#[test]
fn arriving_at_the_target_plane_settles_but_ground_contact_still_errors() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);

    // At the commanded plane, still above the surface: balance weight and settle.
    let mut field = terminal_field(15.0, 2.0);
    let stage = guidance().with_target_altitude(15.0);
    PhysicsStage::<2, f64>::apply(&stage, &ctx, &mut field).unwrap();
    let expected = MASS * GRAVITY / THRUST;
    assert!((field.throttle_action().unwrap() - expected).abs() < 1e-12);

    // At the surface: a different situation, and still the kernel's singularity.
    let mut field = terminal_field(0.0, 2.0);
    let err = PhysicsStage::<2, f64>::apply(&stage, &ctx, &mut field).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::Singularity(_)),
        "expected the kernel's singularity, got {err:?}"
    );
}

#[test]
fn a_stopping_burn_that_cannot_stop_burns_rather_than_coasting() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, DT, 1);
    // Thrust-to-weight below one: no ignition altitude exists, so there is no coast to hold.
    let mut field = terminal_field(400.0, 50.0);
    field.set_scalar("mass", vec![THRUST / GRAVITY * 2.0]);
    let stage = guidance().with_stopping_burn(0.0);

    PhysicsStage::<2, f64>::apply(&stage, &ctx, &mut field).unwrap();

    assert!(field.throttle_action().unwrap() > 0.0);
    assert!(log_has(&field, "stopping burn started"));
}
