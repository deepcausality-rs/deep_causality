/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{
    CausalEffect, CausalEffectPropagationProcess, CausalFlow, CausalityError, CausalityErrorEnum,
    EffectLog,
};

fn errored() -> CausalFlow<i64> {
    CausalFlow::fail(CausalityError::new(CausalityErrorEnum::Custom(
        "boom".into(),
    )))
}

fn none_valued() -> CausalFlow<i64> {
    CausalFlow::from(CausalEffectPropagationProcess::new(
        Ok(CausalEffect::none()),
        (),
        None,
        EffectLog::new(),
    ))
}

// ---- iterate_n -------------------------------------------------------------

#[test]
fn iterate_n_applies_exactly_n_times() {
    let out = CausalFlow::value(0_i64)
        .iterate_n(5, |f| f.map(|x| x + 1))
        .finish();
    assert_eq!(out, Ok(5));
}

#[test]
fn iterate_n_zero_is_identity() {
    let out = CausalFlow::value(7_i64)
        .iterate_n(0, |f| f.map(|x| x + 1))
        .finish();
    assert_eq!(out, Ok(7));
}

#[test]
fn iterate_n_short_circuits_on_error_midway() {
    let out = CausalFlow::value(0_i64)
        .iterate_n(10, |f| {
            f.try_step(|x| {
                if x == 2 {
                    Err(CausalityError::new(CausalityErrorEnum::Custom(
                        "stop".into(),
                    )))
                } else {
                    Ok(x + 1)
                }
            })
        })
        .finish();
    assert!(out.is_err());
}

#[test]
fn iterate_n_on_errored_flow_is_noop() {
    let out = errored().iterate_n(3, |f| f.map(|x| x + 1)).finish();
    assert!(out.is_err());
}

// ---- iterate_until ---------------------------------------------------------

#[test]
fn iterate_until_stops_on_predicate() {
    let out = CausalFlow::value(0_i64)
        .iterate_until(|x| *x >= 3, 100, |f| f.map(|x| x + 1))
        .finish();
    assert_eq!(out, Ok(3));
}

#[test]
fn iterate_until_predicate_true_initially_runs_no_step() {
    let out = CausalFlow::value(5_i64)
        .iterate_until(|x| *x >= 3, 100, |f| f.map(|x| x + 1))
        .finish();
    assert_eq!(out, Ok(5));
}

#[test]
fn iterate_until_fails_at_bound_with_max_steps_exceeded() {
    let flow = CausalFlow::value(0_i64).iterate_until(|x| *x >= 100, 3, |f| f.map(|x| x + 1));
    assert!(flow.is_err());
    let err = flow.finish().unwrap_err();
    assert_eq!(
        format!("{err:?}"),
        format!(
            "{:?}",
            CausalityError::new(CausalityErrorEnum::MaxStepsExceeded)
        )
    );
}

#[test]
fn iterate_until_on_errored_flow_is_noop() {
    let out = errored().iterate_until(|x| *x >= 0, 5, |f| f.map(|x| x + 1));
    assert!(out.is_err());
}

#[test]
fn iterate_until_step_error_short_circuits() {
    let out = CausalFlow::value(0_i64).iterate_until(
        |x| *x >= 100,
        10,
        |f| {
            f.try_step(|x| {
                if x == 1 {
                    Err(CausalityError::new(CausalityErrorEnum::Custom(
                        "stop".into(),
                    )))
                } else {
                    Ok(x + 1)
                }
            })
        },
    );
    assert!(out.is_err());
}

#[test]
fn iterate_until_none_value_never_satisfies_and_fails_at_bound() {
    // A value-less carrier: the predicate never sees a value, so the loop runs to the bound.
    let out = none_valued().iterate_until(|x| *x > 0, 2, |f| f);
    assert!(out.is_err());
}

// ---- iterate_to_fixpoint ---------------------------------------------------

#[test]
fn iterate_to_fixpoint_stops_when_value_stable() {
    // Saturates at 3, then 3 -> 3 is the fixpoint.
    let out = CausalFlow::value(0_i64)
        .iterate_to_fixpoint(100, |f| f.map(|x| (x + 1).min(3)))
        .finish();
    assert_eq!(out, Ok(3));
}

#[test]
fn iterate_to_fixpoint_fails_at_bound() {
    let flow = CausalFlow::value(0_i64).iterate_to_fixpoint(3, |f| f.map(|x| x + 1));
    assert!(flow.is_err());
}

#[test]
fn iterate_to_fixpoint_on_errored_flow_is_noop() {
    let out = errored().iterate_to_fixpoint(3, |f| f.map(|x| x + 1));
    assert!(out.is_err());
}

#[test]
fn iterate_to_fixpoint_step_error_short_circuits() {
    let out = CausalFlow::value(0_i64).iterate_to_fixpoint(10, |f| {
        f.try_step(|x| {
            if x == 1 {
                Err(CausalityError::new(CausalityErrorEnum::Custom(
                    "stop".into(),
                )))
            } else {
                Ok(x + 1)
            }
        })
    });
    assert!(out.is_err());
}

// ---- state threading -------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Default)]
struct Counter {
    ticks: u32,
}

#[test]
fn loops_thread_state_through_each_step() {
    let proc = CausalFlow::process(Counter::default())
        .map(|_unit| 0_i64)
        .iterate_n(3, |f| {
            f.step_mut(|v, st, _ctx| {
                st.ticks += 1;
                Ok(v + 1)
            })
        })
        .into_process();
    assert_eq!(proc.state().ticks, 3);
    assert_eq!(proc.value(), Some(&3));
}
