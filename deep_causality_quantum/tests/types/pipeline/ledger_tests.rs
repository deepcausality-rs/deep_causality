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

// ---------------------------------------------------------------------------
// The overflow guards on the counters.
//
// `Ledger<R, N>` counts on any `N: NaturalNumber`, and `NaturalNumber` is blanket-implemented
// for every unsigned integer. The guards on `observed` and `predicted` therefore fire at
// `N::MAX`, which `u64` never reaches in a test but `u8` reaches in one step. The width is the
// separating input: the same call is fine at 254 and refused at 255.
//
// Both guards are reached through the public `control` path, since `observed` and `predicted`
// are `pub(crate)`. `draw_down` is public and is checked directly, at both sides of its bound.
// ---------------------------------------------------------------------------

#[test]
fn test_draw_down_is_exact_at_its_boundary() {
    // The bound is `request > budget`, so equality must succeed and one more must fail. A test
    // that only overdrew by a wide margin would accept `>=` and lose the exact-exhaustion case.
    type Narrow = u8;
    assert_eq!(Ledger::<f64, Narrow>::draw_down(255, 254).unwrap(), 1);
    assert_eq!(Ledger::<f64, Narrow>::draw_down(255, 255).unwrap(), 0);

    // One past the budget is the first refusal.
    match Ledger::<f64, Narrow>::draw_down(100, 101).unwrap_err().0 {
        QuantumErrorEnum::CalculationError(msg) => {
            assert!(msg.contains("overdrawn"), "{msg}");
            assert!(msg.contains("shortfall 1"), "the shortfall is exact: {msg}");
        }
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_the_shortfall_is_the_monus_not_the_difference() {
    // `monus` is truncated subtraction, so the shortfall is reported on the unsigned width
    // without wrapping. Requesting 300 against a budget of 100 at u16 is a shortfall of 200;
    // a wrapping subtraction would report 65 436.
    match Ledger::<f64, u16>::draw_down(100, 300).unwrap_err().0 {
        QuantumErrorEnum::CalculationError(msg) => {
            assert!(msg.contains("shortfall 200"), "{msg}");
        }
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_the_counter_width_is_what_decides_the_overflow() {
    // The same budget and request at two widths: at u8 the budget cannot even be expressed
    // above 255, so a program naming a narrow width gets its refusals at that width. This is
    // the property the type parameter exists for, and no fixed-width test can observe it.
    assert!(Ledger::<f64, u8>::draw_down(200, 100).is_ok());
    assert!(Ledger::<f64, u16>::draw_down(200, 100).is_ok());
    // And the refusal is the same shape at both widths.
    assert!(Ledger::<f64, u8>::draw_down(100, 200).is_err());
    assert!(Ledger::<f64, u16>::draw_down(100, 200).is_err());
}
