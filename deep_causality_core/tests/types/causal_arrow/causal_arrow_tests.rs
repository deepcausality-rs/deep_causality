/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{
    CausalArrow, CausalEffect, CausalFlow, CausalityError, CausalityErrorEnum, EffectLog,
    causal_arrow,
};
use deep_causality_haft::Arrow;
use std::cell::Cell;

fn err(msg: &str) -> CausalityError {
    CausalityError::new(CausalityErrorEnum::Custom(msg.into()))
}

#[test]
fn arrow_is_reusable_across_inputs() {
    // Stateless stages ignore the threaded `(state, context)`; `run_value` seeds unit state.
    let inc = causal_arrow(|x: i64, _s, _c| CausalFlow::value(x + 1));
    // `run` takes `&self`, so the same arrow applies to many inputs.
    assert_eq!(inc.run_value(3).finish(), Ok(4));
    assert_eq!(inc.run_value(10).finish(), Ok(11));
}

#[test]
fn arrow_application_equals_the_monad_pipeline() {
    let arrow = causal_arrow(|x: i64, _s, _c| CausalFlow::value(x + 1));
    let via_arrow = arrow.run_value(7).finish();
    let via_monad = CausalFlow::value(7_i64).map(|x| x + 1).finish();
    assert_eq!(via_arrow, via_monad);
}

#[test]
fn next_composes_by_binding() {
    let arrow = causal_arrow(|x: i64, _s, _c| CausalFlow::value(x + 1))
        .next(|x, _s, _c| CausalFlow::value(x * 2))
        .build();
    // (3 + 1) * 2
    assert_eq!(arrow.run((3, (), None)).finish(), Ok(8));
}

#[test]
fn next_short_circuits_second_stage_on_first_error() {
    let ran = Cell::new(false);
    let arrow = causal_arrow(|_x: i64, _s, _c| CausalFlow::<i64>::fail(err("first stage failed")))
        .next(|x, _s, _c| {
            ran.set(true);
            CausalFlow::value(x + 1)
        })
        .build();
    let out = arrow.run((0, (), None));
    assert!(out.is_err());
    assert!(
        !ran.get(),
        "second stage must not run after a first-stage error"
    );
}

#[test]
fn three_pipelines_compose_into_one() {
    // ((3 + 1) * 2) - 3 == 5
    let composite = causal_arrow(|x: i64, _s, _c| CausalFlow::value(x + 1))
        .next(|x, _s, _c| CausalFlow::value(x * 2))
        .next(|x, _s, _c| CausalFlow::value(x - 3))
        .build();
    assert_eq!(composite.run((3, (), None)).finish(), Ok(5));
    // Reusable: a second input runs the same composite.
    assert_eq!(composite.run((10, (), None)).finish(), Ok(19));
}

#[test]
fn builder_run_terminal_applies_the_chain() {
    let out = causal_arrow(|x: i64, _s, _c| CausalFlow::value(x + 1))
        .next(|x, _s, _c| CausalFlow::value(x * 2))
        .run_value(3);
    assert_eq!(out.finish(), Ok(8));
}

#[test]
fn and_then_applies_a_reified_arrow() {
    // The bridge from a live flow to a held-as-data composite: apply it with `next`/`run_value`.
    let arrow = causal_arrow(|x: i64, _s, _c| CausalFlow::value(x * 10));
    let out = CausalFlow::value(2_i64)
        .next(|v| arrow.run_value(v))
        .finish();
    assert_eq!(out, Ok(20));
}

#[test]
fn causal_arrow_marker_trait_is_a_usable_bound() {
    fn run_arrow<A: CausalArrow<i64, i64>>(a: &A, x: i64) -> Result<i64, CausalityError> {
        a.run((x, (), None)).finish()
    }
    let a = causal_arrow(|x: i64, _s, _c| CausalFlow::value(x + 1)).build();
    assert_eq!(run_arrow(&a, 5), Ok(6));
}

/// Witness for `THEOREM_MAP: core.causal_arrow.category_laws` (Lean:
/// `DeepCausalityFormal/Core/CausalArrow.lean :: kcomp_left_id/kcomp_right_id/kcomp_assoc`).
/// The state-threading composition IS the Kleisli category law over arbitrary state; identity and
/// associativity of `>>>` are additionally exercised by `next_composes_by_binding` and
/// `three_pipelines_compose_into_one`.
#[test]
fn arrow_threads_accumulated_state() {
    // State is a running sum; each stage adds the incoming value to the state and passes the value
    // on. Composition must thread the state `s0 -> s1 -> s2` — the D2 fix.
    let step = |x: i64, s: i64, _c: Option<()>| {
        CausalFlow::from_parts(Ok(CausalEffect::value(x)), s + x, None, EffectLog::new())
    };
    let pipeline = causal_arrow(step).next(step).build();
    // value 5, initial state 0: state 0 -> +5 -> 5 (stage 1) -> +5 -> 10 (stage 2).
    let out = pipeline.run((5, 0, None)).into_process();
    assert_eq!(*out.state(), 10, "state must thread across both stages");
    assert_eq!(out.into_value(), Some(5));
}

/// Additional case of `THEOREM_MAP: core.causal_arrow.category_laws` (Lean:
/// `DeepCausalityFormal/Core/CausalArrow.lean :: kcomp_right_id`, the `none` branch). Right identity
/// `f >>> arr id = f` holds UNCONDITIONALLY — including when `f` emits a `None` effect: there is no
/// `None → Err` collapse, so composing a value-less stage with the identity arrow leaves it a
/// value-less (but non-errored) carrier with its state intact.
#[test]
fn arrow_right_identity_on_none_emitting_stage() {
    // f emits `None` (and bumps the state); `arr id` re-emits value/state/context untouched.
    let none_stage = |_x: i64, s: i64, _c: Option<()>| {
        CausalFlow::from_parts(Ok(CausalEffect::none()), s + 1, None, EffectLog::new())
    };
    let id_stage = |x: i64, s: i64, _c: Option<()>| {
        CausalFlow::from_parts(Ok(CausalEffect::value(x)), s, None, EffectLog::new())
    };

    // f alone.
    let out_f = causal_arrow(none_stage)
        .build()
        .run((5, 0, None))
        .into_process();
    // f >>> arr id.
    let out_c = causal_arrow(none_stage)
        .next(id_stage)
        .build()
        .run((5, 0, None))
        .into_process();

    assert!(
        !out_f.is_err() && !out_c.is_err(),
        "a None effect is not an error"
    );
    let (sf, sc) = (*out_f.state(), *out_c.state());
    assert_eq!(
        sf, sc,
        "state threads identically on both sides of f >>> arr id = f"
    );
    assert_eq!(sc, 1);
    assert!(out_f.into_value().is_none(), "f yields no value");
    assert!(
        out_c.into_value().is_none(),
        "f >>> arr id yields the same — no None→Err collapse"
    );
}

/// Witness for `THEOREM_MAP: core.causal_arrow.left_zero` (Lean:
/// `DeepCausalityFormal/Core/CausalArrow.lean :: kcomp_left_zero`). An errored stage short-circuits
/// composition; the downstream stage never runs and the accumulated state survives.
#[test]
fn arrow_error_short_circuit_preserves_state() {
    // A first-stage error must short-circuit AND preserve the state accumulated so far.
    let acc = |x: i64, s: i64, _c: Option<()>| {
        CausalFlow::from_parts(Ok(CausalEffect::value(x)), s + x, None, EffectLog::new())
    };
    let boom = |_x: i64, s: i64, _c: Option<()>| {
        CausalFlow::<i64, i64, ()>::from_parts(Err(err("boom")), s, None, EffectLog::new())
    };
    let ran = Cell::new(false);
    let pipeline = causal_arrow(acc)
        .next(boom)
        .next(|x, s: i64, _c| {
            ran.set(true);
            CausalFlow::from_parts(Ok(CausalEffect::value(x)), s, None, EffectLog::new())
        })
        .build();
    let out = pipeline.run((7, 0, None)).into_process();
    assert!(out.is_err());
    assert!(!ran.get(), "stage after the error must not run");
    assert_eq!(
        *out.state(),
        7,
        "state accumulated before the error is preserved"
    );
}
