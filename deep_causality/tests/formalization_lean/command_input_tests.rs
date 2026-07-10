/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for `lean/DeepCausalityFormal/Core/CommandInput.lean` — the F-3 command-input
//! theorem: a command (`RelayTo`) on a singleton's INPUT channel yields a specific, named error —
//! never a silent `None`, never a dropped signal. Lean proves the dispatch is total, the command
//! path is an error (never `ok`), and the command error is distinct from the absence error; these
//! tests pin the real `Causaloid::evaluate` and `evaluate_stateful` singleton paths to those
//! statements.

use deep_causality::utils_test::test_utils;
use deep_causality::{
    CausalEffect, EffectLog, MonadicCausable, PropagatingEffect, PropagatingProcess,
    StatefulMonadicCausable,
};

/// THEOREM_MAP: core.causaloid.command_input
///
/// Lean: `command_yields_cmd_err`, `command_never_ok`, `command_err_distinct_from_absent`
/// (`Core/CommandInput.lean`). The stateless `Causaloid::evaluate` singleton path: a command on the
/// input channel produces the command-specific error (not the generic "input value is None"), never
/// a value, never a silent `None` — and the error is distinct from the absence-of-evidence error.
#[test]
fn test_command_input_yields_command_error() {
    let causaloid = test_utils::get_test_causaloid_deterministic_true();

    // A command (`RelayTo`) on the INPUT channel — carries no `I` value to evaluate.
    let command_input: PropagatingEffect<bool> =
        PropagatingEffect::from_effect(CausalEffect::relay_to(2, CausalEffect::value(true)));
    assert!(
        command_input.value().is_none(),
        "a command carries no input value"
    );

    let out = causaloid.evaluate(&command_input);

    // F-3: the command-specific error — total, never a manufactured `None`, never a dropped signal.
    let msg = out
        .error()
        .expect("command input must error, not manufacture None")
        .to_string();
    assert!(
        msg.contains("command") && msg.contains("RelayTo"),
        "not the command-specific error: {msg}"
    );
    assert!(
        !msg.contains("input value is None"),
        "command conflated with absence-of-evidence: {msg}"
    );
    assert!(
        out.value().is_none(),
        "a command input must not yield a value"
    );

    // Distinct from the absence path: a `None` input gives the GENERIC error, not the command one.
    let none_input: PropagatingEffect<bool> =
        PropagatingEffect::from_effect(CausalEffect::<bool>::none());
    let none_msg = causaloid
        .evaluate(&none_input)
        .error()
        .expect("none input errors too")
        .to_string();
    assert_ne!(
        msg, none_msg,
        "command error must be distinct from the absence error (F-3)"
    );

    // The guard is command-specific: a value input is accepted, not rejected.
    let value_input = PropagatingEffect::from_value(true);
    assert!(
        causaloid.evaluate(&value_input).error().is_none(),
        "a value input must be accepted"
    );
}

/// THEOREM_MAP: core.causaloid.command_input
///
/// Lean: `command_yields_cmd_err`, `command_never_ok` (`Core/CommandInput.lean`). The stateful
/// `Causaloid::evaluate_stateful` singleton path mirrors the stateless one exactly: a command on the
/// input channel yields the command-specific error, never a value.
#[test]
fn test_command_input_yields_command_error_stateful() {
    let causaloid = test_utils::get_test_causaloid_deterministic_true();

    // The stateful command input; the context type is inferred from the `evaluate_stateful` call.
    let command_input = PropagatingProcess::new(
        Ok(CausalEffect::relay_to(2, CausalEffect::value(true))),
        (),
        None,
        EffectLog::new(),
    );

    let out = causaloid.evaluate_stateful(&command_input);

    let msg = out
        .error()
        .expect("stateful command input must error, not manufacture None")
        .to_string();
    assert!(
        msg.contains("command") && msg.contains("RelayTo"),
        "not the command-specific error: {msg}"
    );
    assert!(!msg.contains("input value is None"));
    assert!(
        out.value().is_none(),
        "a command input must not yield a value"
    );
}
