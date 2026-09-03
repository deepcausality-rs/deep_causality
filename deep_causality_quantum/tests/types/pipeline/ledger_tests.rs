/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qcm")]

//! The ledger: counts on ℕ, reals on the scalar, checked draw-down, and a copyable monad state.

use deep_causality_core::PropagatingProcess;
use deep_causality_quantum::{Ledger, QuantumErrorEnum};

/// Counts are ℕ; the width is named once, here.
type Count = u64;

#[test]
fn test_the_default_is_the_two_zeros_and_it_is_copy() {
    let l: Ledger<f64, Count> = Ledger::default();
    assert_eq!(l.shots(), 0);
    assert_eq!(l.experiments(), 0);
    assert_eq!(l.predictions(), 0);
    assert_eq!(l.device_time(), 0.0);
    assert_eq!(l.cost(), 0.0);
    assert_eq!(l.bits(), 0.0);
    let copied = l;
    assert_eq!(copied, l, "Copy: the original is still usable");
}

#[test]
fn test_the_draw_down_is_checked_arithmetic_rather_than_a_guard() {
    assert_eq!(Ledger::<f64, Count>::draw_down(100, 60).unwrap(), 40);
    assert_eq!(Ledger::<f64, Count>::draw_down(100, 100).unwrap(), 0);
    match Ledger::<f64, Count>::draw_down(100, 150).unwrap_err().0 {
        QuantumErrorEnum::CalculationError(msg) => assert!(msg.contains("shortfall 50"), "{msg}"),
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_the_ledger_is_the_causal_monad_state() {
    // `pure` requires `State: Default`, and the hand-written Default is what satisfies it.
    let process: PropagatingProcess<f64, Ledger<f64, Count>, ()> = PropagatingProcess::pure(0.5);
    assert_eq!(*process.state(), Ledger::default());
}

#[test]
fn test_the_width_is_a_parameter_and_moves_no_threshold() {
    // u32 and u64 both instantiate; the count width is the only thing that changes.
    let narrow: Ledger<f64, u32> = Ledger::new();
    let wide: Ledger<f64, u64> = Ledger::new();
    assert_eq!(narrow.shots(), 0u32);
    assert_eq!(wide.shots(), 0u64);
    assert!(Ledger::<f64, u32>::draw_down(5, 7).is_err());
}
