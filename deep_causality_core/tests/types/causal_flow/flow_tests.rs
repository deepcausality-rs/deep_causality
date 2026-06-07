/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{
    CausalEffectPropagationProcess, CausalFlow, CausalityError, CausalityErrorEnum, EffectLog,
    EffectValue, PropagatingEffect, PropagatingProcess,
};
use std::cell::Cell;

fn err(msg: &str) -> CausalityError {
    CausalityError::new(CausalityErrorEnum::Custom(msg.into()))
}

// -----------------------------------------------------------------------------
// 5.1 Constructors
// -----------------------------------------------------------------------------

#[test]
fn effect_seeds_unit_value() {
    let e = CausalFlow::effect().into_effect();
    assert_eq!(e.value, EffectValue::Value(()));
    assert!(e.error.is_none());
    assert!(e.context.is_none());
}

#[test]
fn value_seeds_value() {
    assert_eq!(CausalFlow::value(7i64).finish(), Ok(7));
}

#[test]
fn fail_seeds_error() {
    let f = CausalFlow::<i64>::fail(err("boom"));
    assert!(f.is_err());
    match f.finish() {
        Err(e) => assert!(format!("{e:?}").contains("boom")),
        Ok(_) => panic!("expected error"),
    }
}

#[test]
fn process_context_seeds_state_and_context() {
    let p = CausalFlow::process(5i64).context("cfg").into_process();
    assert_eq!(p.state, 5);
    assert_eq!(p.context, Some("cfg"));
    assert_eq!(p.value, EffectValue::Value(()));
}

// -----------------------------------------------------------------------------
// 5.2 Steps
// -----------------------------------------------------------------------------

#[test]
fn try_step_chains_on_success() {
    let out = CausalFlow::value(2i64)
        .try_step(|x| Ok(x + 3))
        .try_step(|x| Ok(x * 10))
        .finish();
    assert_eq!(out, Ok(50));
}

#[test]
fn try_step_err_short_circuits() {
    let out: Result<i64, _> = CausalFlow::value(2i64)
        .try_step(|_| Err(err("nope")))
        .finish();
    assert!(out.is_err());
}

#[test]
fn and_then_chains_flows() {
    let out = CausalFlow::value(2i64)
        .and_then(|x| CausalFlow::value(x * 5))
        .finish();
    assert_eq!(out, Ok(10));
}

#[test]
fn map_transforms_and_short_circuits() {
    assert_eq!(CausalFlow::value(4i64).map(|x| x + 1).finish(), Ok(5));
    assert!(
        CausalFlow::<i64>::fail(err("x"))
            .map(|x| x + 1)
            .finish()
            .is_err()
    );
}

#[test]
fn guard_validates() {
    let ok = CausalFlow::value(5i64)
        .guard(|v| if *v > 0 { Ok(()) } else { Err(err("neg")) })
        .finish();
    assert_eq!(ok, Ok(5));
    let bad = CausalFlow::value(-1i64)
        .guard(|v| if *v > 0 { Ok(()) } else { Err(err("neg")) })
        .finish();
    assert!(bad.is_err());
}

#[test]
fn recover_clears_error_and_is_noop_on_success() {
    assert_eq!(
        CausalFlow::<i64>::fail(err("x")).recover(|_| 7).finish(),
        Ok(7)
    );
    assert_eq!(CausalFlow::value(5i64).recover(|_| 7).finish(), Ok(5));
}

#[test]
fn try_step_with_reads_state_and_context() {
    let out = CausalFlow::process(3i64)
        .context(4i64)
        .try_step_with(|_v: (), st: &i64, ctx: Option<&i64>| Ok(*st + *ctx.unwrap()))
        .finish();
    assert_eq!(out, Ok(7));
}

#[test]
fn step_mut_mutates_state_while_transforming_value() {
    let p = CausalFlow::process(0i64)
        .step_mut(|_v: (), st: &mut i64, _ctx| {
            *st += 10;
            Ok("done")
        })
        .into_process();
    assert_eq!(p.state, 10);
    assert_eq!(p.value, EffectValue::Value("done"));
}

#[test]
fn update_value_updates_value_in_place() {
    let p = CausalFlow::value(5i64)
        .update_value(|v| v * 2)
        .into_effect();
    assert_eq!(p.value, EffectValue::Value(10));
}

#[test]
fn update_state_evolves_state() {
    let p = CausalFlow::process(1i64)
        .update_state(|s, _v| s + 100)
        .into_process();
    assert_eq!(p.state, 101);
}

#[test]
fn update_context_evolves_context_from_value() {
    let p = CausalFlow::process(0i64) // state = 0
        .context(10i64) // context = 10
        .map(|_unit| 5i64) // value = 5
        .update_context(|c, v| Some(c.unwrap_or(0) + v))
        .into_process();
    assert_eq!(p.context, Some(15)); // 10 + 5
    assert_eq!(p.value, EffectValue::Value(5)); // unchanged
    assert_eq!(p.state, 0); // unchanged
}

#[test]
fn update_value_state_context_rewrites_all_three() {
    let p = CausalFlow::process(1i64) // state = 1
        .context(2i64) // context = 2
        .map(|_unit| 3i64) // value = 3
        .update_value_state_context(|v, s, c| {
            let ctx = c.unwrap_or(0);
            (v + 10, s + ctx, Some(ctx + 100))
        })
        .into_process();
    assert_eq!(p.value, EffectValue::Value(13)); // 3 + 10
    assert_eq!(p.state, 3); // 1 + 2
    assert_eq!(p.context, Some(102)); // 2 + 100
}

#[test]
fn intervene_substitutes_value_and_logs_override() {
    let p = CausalFlow::value(1i64).intervene(99).into_effect();
    assert_eq!(p.value, EffectValue::Value(99));
    assert!(format!("{:?}", p.logs).contains("ValueAlternation"));
}

#[test]
fn intervene_if_fires_only_on_condition() {
    let fired = CausalFlow::value(10i64)
        .intervene_if(|v| *v > 5, |_| 0)
        .finish();
    assert_eq!(fired, Ok(0));
    let passthrough = CausalFlow::value(3i64)
        .intervene_if(|v| *v > 5, |_| 0)
        .finish();
    assert_eq!(passthrough, Ok(3));
}

#[test]
fn intervene_if_skips_errored_flow() {
    // A carrier holding BOTH a value and an error (reachable via `from`): the error must take
    // precedence so neither closure runs (they panic if invoked).
    let errored: PropagatingEffect<i64> = CausalEffectPropagationProcess {
        value: EffectValue::Value(10),
        state: (),
        context: None,
        error: Some(err("boom")),
        logs: EffectLog::new(),
    };
    let out = CausalFlow::from(errored)
        .intervene_if(
            |_| panic!("cond ran on an errored flow"),
            |_| panic!("f ran on an errored flow"),
        )
        .finish();
    assert!(out.is_err());
}

#[test]
fn map_preserves_contextual_link_carrier() {
    // `ContextualLink` is not a plain value; `map` must pass it through, not drop it to `None`.
    let linked: PropagatingEffect<i64> = CausalEffectPropagationProcess {
        value: EffectValue::ContextualLink(7, 9),
        state: (),
        context: None,
        error: None,
        logs: EffectLog::new(),
    };
    let out = CausalFlow::from(linked).map(|x: i64| x + 1).into_effect();
    assert!(matches!(out.value, EffectValue::ContextualLink(7, 9)));
    assert!(out.error.is_none());
}

#[test]
fn map_surfaces_error_on_dispatch_variant() {
    // `RelayTo` embeds a `PropagatingEffect` a value-level map cannot retype; `map` must surface
    // `ValueNotAvailable` rather than silently dropping the dispatch command.
    let dispatch: PropagatingEffect<i64> = CausalEffectPropagationProcess {
        value: EffectValue::RelayTo(3, Box::new(PropagatingEffect::from_value(42))),
        state: (),
        context: None,
        error: None,
        logs: EffectLog::new(),
    };
    let out = CausalFlow::from(dispatch).map(|x: i64| x + 1).finish();
    assert!(out.is_err());
    assert!(format!("{:?}", out.unwrap_err()).contains("ValueNotAvailable"));
}

#[test]
fn bind_or_error_passthrough_runs_existing_stage() {
    fn stage(x: i64, s: (), c: Option<()>) -> PropagatingProcess<i64, (), ()> {
        CausalEffectPropagationProcess {
            value: EffectValue::Value(x * 2),
            state: s,
            context: c,
            error: None,
            logs: EffectLog::new(),
        }
    }
    let out = CausalFlow::value(21i64)
        .bind_or_error(stage, "fail")
        .finish();
    assert_eq!(out, Ok(42));
}

#[test]
fn bind_passthrough_runs_existing_stage() {
    let out = CausalFlow::value(5i64)
        .bind(|ev, s, c| {
            let v = ev.into_value().unwrap_or_default();
            CausalEffectPropagationProcess {
                value: EffectValue::Value(v + 1),
                state: s,
                context: c,
                error: None,
                logs: EffectLog::new(),
            }
        })
        .finish();
    assert_eq!(out, Ok(6));
}

// -----------------------------------------------------------------------------
// 5.3 Terminals + interop
// -----------------------------------------------------------------------------

#[test]
fn run_dispatches_to_handler_by_outcome() {
    let ok_seen = Cell::new(0i64);
    CausalFlow::value(9i64).run(|v| ok_seen.set(v), |_| ok_seen.set(-1));
    assert_eq!(ok_seen.get(), 9);

    let err_seen = Cell::new(false);
    CausalFlow::<i64>::fail(err("x")).run(|_| {}, |_| err_seen.set(true));
    assert!(err_seen.get());
}

#[test]
fn from_and_into_round_trip_losslessly() {
    let eff = PropagatingEffect::from_value(42i64);
    let flow: CausalFlow<i64> = CausalFlow::from(eff);
    assert_eq!(flow.into_effect().value, EffectValue::Value(42));
}

#[test]
fn finish_on_absent_value_without_error_is_value_not_available() {
    // A flow carrying neither a value nor an error short-circuits `finish` to ValueNotAvailable.
    let flow: CausalFlow<i64> = CausalFlow::from(PropagatingEffect::none());
    let out = flow.finish();
    assert!(out.is_err());
    assert!(format!("{:?}", out.unwrap_err()).contains("ValueNotAvailable"));
}

// -----------------------------------------------------------------------------
// 5.4 Behavior-preserving parity with the raw monad
// -----------------------------------------------------------------------------

#[test]
fn flow_chain_matches_raw_bind_chain() {
    let via_flow: PropagatingEffect<i64> = CausalFlow::value(2i64)
        .try_step(|x| Ok(x + 3))
        .into_effect();
    let via_raw: PropagatingEffect<i64> = PropagatingEffect::from_value(2i64).bind_or_error(
        |x, s, c| CausalEffectPropagationProcess {
            value: EffectValue::Value(x + 3),
            state: s,
            context: c,
            error: None,
            logs: EffectLog::new(),
        },
        "msg",
    );
    assert_eq!(via_flow.value, via_raw.value);
    assert_eq!(via_flow.error.is_some(), via_raw.error.is_some());
}

// -----------------------------------------------------------------------------
// 5.5 Error-path coverage
// -----------------------------------------------------------------------------

#[test]
fn step_does_not_invoke_closure_on_a_failed_flow() {
    let called = Cell::new(false);
    let out = CausalFlow::<i64>::fail(err("x"))
        .try_step(|v| {
            called.set(true);
            Ok(v + 1)
        })
        .finish();
    assert!(out.is_err());
    assert!(!called.get(), "closure must not run on a failed flow");
}

#[test]
fn finish_on_none_value_yields_error() {
    // A flow whose value is None (not an error) still finishes as an error.
    let none_flow: CausalFlow<i64> =
        CausalFlow::from(PropagatingEffect::from_effect_value(EffectValue::None));
    assert!(none_flow.finish().is_err());
}
