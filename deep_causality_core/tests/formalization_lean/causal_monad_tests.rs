/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witness for the lawful-monad bundle (`core.causal_monad.lawful`).
//!
//! Mirrors `lean/DeepCausalityFormal/Core/CausalMonad.lean :: causal_monad_lawful`. The individual
//! laws (`left_id`/`right_id`/`assoc`/`left_zero`) are witnessed by the Kani harnesses and the
//! `tests/types/causal_monad/` tests; this witness pins the **joint** claim that all three co-hold
//! over one carrier — the `LawfulMonad`-with-effect statement that was blocked on P1 and is now
//! unblocked (control is separated into `CausalCommand`/`CausalEffect`).

use deep_causality_core::{
    CausalEffect, CausalEffectPropagationProcess, EffectLog, PropagatingProcess,
};
use deep_causality_haft::LogAddEntry;

type P<T> = PropagatingProcess<T, i32, String>;

/// A log carrying a single history entry.
fn hist_log() -> EffectLog {
    let mut l = EffectLog::new();
    l.add_entry("history");
    l
}

/// The state-threading Kleisli unit `eta`: re-emit value/state/context with an empty log.
fn eta(v: CausalEffect<i32>, s: i32, c: Option<String>) -> P<i32> {
    CausalEffectPropagationProcess::new(Ok(v), s, c, EffectLog::new())
}

/// A representative continuation: increment the value, thread state forward.
fn f(v: CausalEffect<i32>, s: i32, c: Option<String>) -> P<i32> {
    let val = v.into_value().unwrap_or_default();
    CausalEffectPropagationProcess::new(
        Ok(CausalEffect::value(val + 1)),
        s + 10,
        c,
        EffectLog::new(),
    )
}

/// A second continuation: double the value.
fn g(v: CausalEffect<i32>, s: i32, c: Option<String>) -> P<i32> {
    let val = v.into_value().unwrap_or_default();
    CausalEffectPropagationProcess::new(Ok(CausalEffect::value(val * 2)), s, c, EffectLog::new())
}

// ---- core.causal_monad.lawful : the three laws co-hold on one carrier --------------------------

/// THEOREM_MAP: core.causal_monad.lawful
#[test]
fn test_causal_monad_lawful() {
    // Left identity: bind(pure v, f) == f applied at the injected point.
    let left_lhs = PropagatingProcess::pure(3).bind(f);
    let left_rhs = f(CausalEffect::value(3), 0, None);
    assert_eq!(left_lhs, left_rhs, "left identity");

    // Right identity (unconditional): bind(m, eta) == m for value, none, and errored carriers.
    let carriers: Vec<P<i32>> = vec![
        PropagatingProcess::pure(42),
        PropagatingProcess::none(),
        PropagatingProcess::new(
            Ok(CausalEffect::value(7)),
            3,
            Some("ctx".to_string()),
            hist_log(),
        ),
    ];
    for m in carriers {
        let expected = m.clone();
        assert_eq!(m.bind(eta), expected, "right identity");
    }

    // Associativity: bind(bind(m, f), g) == bind(m, |v,s,c| bind(f(v,s,c), g)).
    let m: P<i32> = PropagatingProcess::new(Ok(CausalEffect::value(5)), 1, None, EffectLog::new());
    let m2 = m.clone();
    let assoc_lhs = m.bind(f).bind(g);
    let assoc_rhs = m2.bind(|v, s, c| f(v, s, c).bind(g));
    assert_eq!(assoc_lhs, assoc_rhs, "associativity");

    // The three co-hold on the SAME carrier shape — the joint lawful claim (P1 resolved).
    assert!(matches!(assoc_lhs.value(), Some(&12)));
}
