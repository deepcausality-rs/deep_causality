/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{
    CausalEffectPropagationProcess, CausalFlow, CausalityError, CausalityErrorEnum, EffectLog,
    EffectValue, Either,
};

fn err(msg: &str) -> CausalityError {
    CausalityError::new(CausalityErrorEnum::Custom(msg.into()))
}

// ---- branch ----------------------------------------------------------------

#[test]
fn branch_takes_true_arm() {
    let out = CausalFlow::value(10_i64)
        .branch(|v| *v > 5, |f| f.map(|x| x * 2), |f| f)
        .finish();
    assert_eq!(out, Ok(20));
}

#[test]
fn branch_takes_false_arm() {
    let out = CausalFlow::value(3_i64)
        .branch(|v| *v > 5, |f| f.map(|x| x * 2), |f| f.map(|x| x + 100))
        .finish();
    assert_eq!(out, Ok(103));
}

#[test]
fn branch_is_noop_on_errored_flow() {
    let out = CausalFlow::<i64>::fail(err("boom"))
        .branch(
            |_| panic!("predicate ran on errored flow"),
            |_| panic!("on_true ran on errored flow"),
            |_| panic!("on_false ran on errored flow"),
        )
        .finish();
    assert!(out.is_err());
}

#[test]
fn branch_passes_through_value_less_flow() {
    let none: CausalFlow<i64> = CausalFlow::from(CausalEffectPropagationProcess {
        value: EffectValue::None,
        state: (),
        context: None,
        error: None,
        logs: EffectLog::new(),
    });
    let out = none.branch(
        |_| panic!("predicate ran on a value-less flow"),
        |_| panic!("on_true ran"),
        |_| panic!("on_false ran"),
    );
    // No value and no error: finishing yields a ValueNotAvailable error, and no arm ran.
    assert!(out.finish().is_err());
}

// ---- branch_with -----------------------------------------------------------

#[test]
fn branch_with_reads_context_in_predicate() {
    let out = CausalFlow::process(())
        .context(5_i64)
        .map(|_| 10_i64)
        .branch_with(
            |v, _st, ctx| *v > *ctx.expect("threshold present"),
            |hot| hot.map(|v| v * 2),
            |cold| cold,
        )
        .finish();
    assert_eq!(out, Ok(20));
}

#[derive(Clone, Debug, PartialEq, Default)]
struct St {
    corrections: u32,
}

#[test]
fn branch_with_threads_state_through_the_arm() {
    let proc = CausalFlow::process(St::default())
        .map(|_| 10_i64)
        .branch_with(
            |v, _st, _ctx| *v > 5,
            |hot| {
                hot.step_mut(|v, st, _| {
                    st.corrections += 1;
                    Ok(v)
                })
            },
            |cold| cold,
        )
        .into_process();
    assert_eq!(proc.state.corrections, 1);
    assert_eq!(proc.value, EffectValue::Value(10));
}

#[test]
fn branch_with_is_noop_on_errored_flow() {
    let out = CausalFlow::<i64>::fail(err("boom")).branch_with(
        |_, _, _| panic!("predicate ran on errored flow"),
        |_| panic!("on_true ran"),
        |_| panic!("on_false ran"),
    );
    assert!(out.is_err());
}

#[test]
fn branch_with_passes_through_value_less_flow() {
    let none: CausalFlow<i64> = CausalFlow::from(CausalEffectPropagationProcess {
        value: EffectValue::None,
        state: (),
        context: None,
        error: None,
        logs: EffectLog::new(),
    });
    let out = none.branch_with(
        |_, _, _| panic!("predicate ran"),
        |_| panic!("on_true ran"),
        |_| panic!("on_false ran"),
    );
    assert!(out.finish().is_err());
}

// ---- either ----------------------------------------------------------------

#[test]
fn either_routes_left() {
    let flow: CausalFlow<Either<i64, String>> = CausalFlow::value(Either::Left(5));
    let out = flow
        .either(|l| l.map(|x| x * 2), |r| r.map(|_s| -1_i64))
        .finish();
    assert_eq!(out, Ok(10));
}

#[test]
fn either_routes_right() {
    let flow: CausalFlow<Either<i64, String>> = CausalFlow::value(Either::Right("hi".to_string()));
    let out = flow
        .either(|l| l.map(|x| x * 2), |r| r.map(|s| s.len() as i64))
        .finish();
    assert_eq!(out, Ok(2));
}

#[test]
fn either_is_noop_on_errored_flow() {
    let flow: CausalFlow<Either<i64, String>> = CausalFlow::fail(err("boom"));
    let out: CausalFlow<i64> = flow.either(
        |_l| panic!("left arm ran on errored flow"),
        |_r| panic!("right arm ran on errored flow"),
    );
    assert!(out.is_err());
}

#[test]
fn either_passes_through_value_less_flow() {
    let none: CausalFlow<Either<i64, String>> = CausalFlow::from(CausalEffectPropagationProcess {
        value: EffectValue::None,
        state: (),
        context: None,
        error: None,
        logs: EffectLog::new(),
    });
    let out: CausalFlow<i64> = none.either(
        |_l| panic!("left arm ran on a value-less flow"),
        |_r| panic!("right arm ran on a value-less flow"),
    );
    assert!(out.finish().is_err());
}
