/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The 3-DOF [`BankSteeredLift`] aero producer and the [`BranchAccumulator`] branch reducer.

use super::{ctx, field};
use deep_causality_cfd::{
    BankSteeredLift, BranchAccumulator, CoupledField, CyberneticCorrect, PhysicsStage,
    SafetyEnvelope,
};

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
// BankSteeredLift (the 3-DOF ④ producer)
// ---------------------------------------------------------------------------

// A truth vehicle on the +x radial flying tangentially along +y: the lift plane's
// in-plane "up" is +x and the side direction v̂ × n̂ is −z.
fn steered_field(bank: Option<f64>) -> CoupledField<f64> {
    let mut f = field();
    f.set_scalar("speed", vec![100.0]);
    f.set_scalar("truth_state", vec![7.0e6, 0.0, 0.0, 0.0, 7.5e3, 0.0]);
    if let Some(b) = bank {
        f.set_control_action(b);
    }
    f
}

// rho_ref = 1, C_d·A/m = 1, L/D = 1 over U_max = 100: a_drag = a_lift = q = 5000.
fn steered_stage() -> BankSteeredLift<f64> {
    BankSteeredLift::new(1.0, 1.0, 1.0)
}

#[test]
fn bank_steered_lift_writes_zero_force_without_speed() {
    let mut f = field();
    f.set_scalar("truth_state", vec![7.0e6, 0.0, 0.0, 0.0, 7.5e3, 0.0]);
    steered_stage().apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(
        f.aero_force(),
        Some([0.0, 0.0, 0.0]),
        "no dynamic pressure, zero force"
    );
}

#[test]
fn missing_speed_zeroes_a_previously_written_force() {
    // Step 1 writes a real aero force; step 2's speed field is gone. The stale force must not
    // stay latched and keep kicking the trajectory: the stage zeroes the ④ channel.
    let mut f = steered_field(None);
    steered_stage().apply(&ctx(1), &mut f).expect("applies");
    assert_ne!(f.aero_force(), Some([0.0, 0.0, 0.0]), "a real force landed");

    assert!(f.take_scalar("speed").is_some(), "the publisher goes quiet");
    steered_stage().apply(&ctx(2), &mut f).expect("applies");
    assert_eq!(
        f.aero_force(),
        Some([0.0, 0.0, 0.0]),
        "no force this step, not the latched one"
    );
}

#[test]
fn bank_steered_lift_falls_back_to_axis_drag_without_a_truth_state() {
    let mut f = field();
    f.set_scalar("speed", vec![100.0]);
    steered_stage().apply(&ctx(1), &mut f).expect("applies");
    let a = f.aero_force().expect("force written");
    assert_eq!(a, [-5000.0, 0.0, 0.0], "the AeroForceCoupling behavior");
}

#[test]
fn zero_bank_keeps_the_lift_in_plane() {
    let mut f = steered_field(None);
    steered_stage().apply(&ctx(1), &mut f).expect("applies");
    let a = f.aero_force().expect("force written");
    // Drag opposes +y; the zero-bank lift points up the local radial (+x); nothing leaves
    // the orbital plane.
    assert!((a[0] - 5000.0).abs() < 1e-9, "lift up the radial: {}", a[0]);
    assert!((a[1] + 5000.0).abs() < 1e-9, "drag against v: {}", a[1]);
    assert!(a[2].abs() < 1e-9, "in-plane at zero bank: {}", a[2]);
}

#[test]
fn opposite_banks_curve_the_trajectory_oppositely() {
    let bank = 0.5_f64;
    let mut left = steered_field(Some(bank));
    steered_stage().apply(&ctx(1), &mut left).expect("applies");
    let mut right = steered_field(Some(-bank));
    steered_stage().apply(&ctx(1), &mut right).expect("applies");

    let al = left.aero_force().expect("force");
    let ar = right.aero_force().expect("force");
    assert!(al[2] != 0.0, "banking leaves the plane");
    assert!(
        (al[2] + ar[2]).abs() < 1e-9,
        "mirror banks push out-of-plane oppositely: {} vs {}",
        al[2],
        ar[2]
    );
    // The in-plane lift shrinks by cos φ identically on both.
    assert!((al[0] - ar[0]).abs() < 1e-9);
    assert!((al[0] - 5000.0 * bank.cos()).abs() < 1e-6);
}

#[test]
fn the_clamped_command_actuates_not_the_raw_one() {
    // A raw guidance command far beyond the envelope's bank cap: the gate clamps the channel,
    // and the next step's lift flies the clamped value (the one-step actuation lag).
    let cap = 0.2_f64;
    let gate = CyberneticCorrect::new(SafetyEnvelope::new(1.0e9, 100.0, cap));

    let mut f = steered_field(Some(10.0));
    gate.apply(&ctx(1), &mut f).expect("gate clamps");
    steered_stage().apply(&ctx(2), &mut f).expect("applies");
    let clamped = f.aero_force().expect("force");

    let mut reference = steered_field(Some(cap));
    steered_stage()
        .apply(&ctx(2), &mut reference)
        .expect("applies");
    let expected = reference.aero_force().expect("force");

    assert_eq!(
        clamped, expected,
        "the actuated bank is the gate's clamp, not the raw command"
    );
}

#[test]
fn finish_at_derives_the_miss_from_the_terminal_state() {
    let aim = [6.4e6_f64, 0.0, 0.0];

    // On the aim point: zero miss.
    let on_target = BranchAccumulator::new(0.0).finish_at(aim, aim);
    assert_eq!(on_target.miss_distance, 0.0);

    // Distinct terminal states (distinct banks steer distinct trajectories) yield
    // distinct, dynamics-derived misses.
    let short = BranchAccumulator::new(0.3).finish_at([6.4e6, 3.0e3, -4.0e3], aim);
    let wide = BranchAccumulator::new(-0.3).finish_at([6.4e6, 9.0e3, -1.2e4], aim);
    assert_eq!(short.miss_distance, 5.0e3);
    assert_eq!(wide.miss_distance, 1.5e4);
    assert!(short.miss_distance != wide.miss_distance);
}
