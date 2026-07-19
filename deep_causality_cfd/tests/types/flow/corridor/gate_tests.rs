/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The [`CyberneticCorrect`] bounded-correction gate on the corridor path, where the envelope
//! carries no burn axes.

use super::{ctx, field, gate};
use deep_causality_cfd::{BankCorrection, PhysicsStage};
use deep_causality_haft::LogSize;

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
