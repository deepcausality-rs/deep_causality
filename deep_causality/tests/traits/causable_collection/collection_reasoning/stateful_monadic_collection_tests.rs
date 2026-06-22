/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for [`StatefulMonadicCausableCollection`] over `Vec<Causaloid<...>>`.

use deep_causality::*;
use deep_causality_core::CausalityErrorEnum;
use deep_causality_haft::LogAddEntry;

#[derive(Debug, Default, Clone, PartialEq)]
struct CounterState {
    count: u64,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct ConfigCtx {
    threshold: u64,
}

fn item_true_increment(
    obs: EffectValue<u64>,
    mut state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<bool, CounterState, ConfigCtx> {
    let val = obs.into_value().unwrap_or(0);
    state.count += 1;
    let mut p = PropagatingProcess {
        value: EffectValue::Value(val > 0),
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    p.logs.add_entry("item_true_increment");
    p
}

fn item_false_increment(
    _obs: EffectValue<u64>,
    mut state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<bool, CounterState, ConfigCtx> {
    state.count += 1;
    let mut p = PropagatingProcess {
        value: EffectValue::Value(false),
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    p.logs.add_entry("item_false_increment");
    p
}

fn item_failing(
    _obs: EffectValue<u64>,
    state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<bool, CounterState, ConfigCtx> {
    let mut p = PropagatingProcess {
        value: EffectValue::None,
        state,
        context: ctx,
        error: Some(CausalityError::new(CausalityErrorEnum::Custom(
            "item_failing: deliberate".into(),
        ))),
        logs: EffectLog::new(),
    };
    p.logs.add_entry("item_failing: invoked");
    p
}

fn build_incoming() -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    PropagatingProcess {
        value: EffectValue::Value(7),
        state: CounterState::default(),
        context: Some(ConfigCtx { threshold: 1 }),
        error: None,
        logs: EffectLog::new(),
    }
}

fn item_uncertain_float(
    _obs: EffectValue<u64>,
    state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<deep_causality_uncertain::UncertainF64, CounterState, ConfigCtx> {
    PropagatingProcess {
        value: EffectValue::Value(deep_causality_uncertain::Uncertain::<f64>::point(1.0)),
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    }
}

#[test]
fn evaluate_collection_stateful_short_circuits_on_incoming_error() {
    // An incoming process that already carries an error returns immediately
    // with that error and the incoming state preserved; no item runs.
    let items: Vec<Causaloid<u64, bool, CounterState, ConfigCtx>> =
        vec![Causaloid::new_with_context(
            1,
            item_true_increment,
            ConfigCtx { threshold: 1 },
            "a",
        )];

    let incoming = PropagatingProcess {
        value: EffectValue::Value(7u64),
        state: CounterState { count: 9 },
        context: Some(ConfigCtx { threshold: 1 }),
        error: Some(CausalityError::new(CausalityErrorEnum::Custom(
            "pre-existing".into(),
        ))),
        logs: EffectLog::new(),
    };

    let out =
        items
            .as_slice()
            .evaluate_collection_stateful(&incoming, &AggregateLogic::All, Some(0.5));

    assert!(out.error.is_some());
    assert_eq!(out.state.count, 9, "incoming state preserved, no item ran");
}

#[test]
fn evaluate_collection_stateful_empty_collection_errors() {
    let items: Vec<Causaloid<u64, bool, CounterState, ConfigCtx>> = vec![];

    let out = items.as_slice().evaluate_collection_stateful(
        &build_incoming(),
        &AggregateLogic::All,
        Some(0.5),
    );

    assert!(out.error.is_some());
    assert!(format!("{:?}", out.error).contains("Cannot evaluate an empty collection"));
}

#[test]
fn evaluate_collection_stateful_aggregation_error() {
    // Items evaluate successfully but produce UncertainF64 values, which the
    // aggregation helper cannot combine -> the `Err(e)` aggregation arm runs.
    let items: Vec<
        Causaloid<u64, deep_causality_uncertain::UncertainF64, CounterState, ConfigCtx>,
    > = vec![
        Causaloid::new_with_context(1, item_uncertain_float, ConfigCtx { threshold: 1 }, "a"),
        Causaloid::new_with_context(2, item_uncertain_float, ConfigCtx { threshold: 1 }, "b"),
    ];

    let incoming = PropagatingProcess {
        value: EffectValue::Value(7u64),
        state: CounterState::default(),
        context: Some(ConfigCtx { threshold: 1 }),
        error: None,
        logs: EffectLog::new(),
    };

    let out =
        items
            .as_slice()
            .evaluate_collection_stateful(&incoming, &AggregateLogic::All, Some(0.5));

    assert!(out.error.is_some());
    assert!(format!("{:?}", out.error).contains("not supported"));
}

#[test]
fn evaluate_collection_stateful_aggregates_and_threads_state() {
    // Three items each increment the counter; aggregator is "Some(2)" out of 3 trues.
    let items: Vec<Causaloid<u64, bool, CounterState, ConfigCtx>> = vec![
        Causaloid::new_with_context(1, item_true_increment, ConfigCtx { threshold: 1 }, "a"),
        Causaloid::new_with_context(2, item_true_increment, ConfigCtx { threshold: 1 }, "b"),
        Causaloid::new_with_context(3, item_true_increment, ConfigCtx { threshold: 1 }, "c"),
    ];

    let incoming = build_incoming();
    let out = items.as_slice().evaluate_collection_stateful(
        &incoming,
        &AggregateLogic::Some(2),
        Some(0.5),
    );

    assert!(out.error.is_none(), "expected success, got {:?}", out.error);
    assert_eq!(
        out.state.count, 3,
        "state must reflect three counter increments threaded across items"
    );
    assert_eq!(out.value, EffectValue::Value(true));
}

#[test]
fn evaluate_collection_stateful_short_circuits_with_state_at_failure_point() {
    // Item 2 fails; item 3 must not execute. State should reflect only item 1's increment.
    let items: Vec<Causaloid<u64, bool, CounterState, ConfigCtx>> = vec![
        Causaloid::new_with_context(1, item_true_increment, ConfigCtx { threshold: 1 }, "a"),
        Causaloid::new_with_context(2, item_failing, ConfigCtx { threshold: 1 }, "fail"),
        Causaloid::new_with_context(3, item_false_increment, ConfigCtx { threshold: 1 }, "c"),
    ];

    let incoming = build_incoming();
    let out = items.as_slice().evaluate_collection_stateful(
        &incoming,
        &AggregateLogic::Some(2),
        Some(0.5),
    );

    assert!(out.error.is_some(), "expected error from item 2");
    assert_eq!(
        out.state.count, 1,
        "state must reflect item 1 only (the state item 2 received as input)"
    );
    // Logs must include the "item_failing: invoked" entry but no item_3 log.
    let log_text = format!("{:?}", out.logs);
    assert!(
        log_text.contains("item_failing"),
        "expected log entry from failing item: {}",
        log_text
    );
    assert!(
        !log_text.contains("item_false_increment"),
        "downstream item 3 must not have logged: {}",
        log_text
    );
}
