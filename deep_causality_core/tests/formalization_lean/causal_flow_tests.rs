/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the flow-facade laws and documented extensions (`core.causal_flow.*`).
//!
//! Mirrors `lean/DeepCausalityFormal/Core/CausalFlow.lean`. `CausalFlow` is a newtype over the
//! causal monad; the facade lowers faithfully (`flow_iso`), its `map` is a total value functor
//! (`map_id`/`map_comp`, and `map = and_then(pure∘f)` on the `None` effect as well as a value — D14),
//! and `recover`/`iterate_*`/`finish` are documented extensions with their own contracts.

use deep_causality_core::{
    CausalEffect, CausalEffectPropagationProcess, CausalFlow, EffectLog, PropagatingProcess,
};

/// A stateless `None`-effect flow (no value, not an error).
fn none_flow() -> CausalFlow<i64> {
    CausalFlow::from(CausalEffectPropagationProcess::new(
        Ok(CausalEffect::none()),
        (),
        None,
        EffectLog::new(),
    ))
}

// ---- core.causal_flow.flow_iso -----------------------------------------------------------------

/// THEOREM_MAP: core.causal_flow.flow_iso
#[test]
fn test_causal_flow_iso() {
    let p: PropagatingProcess<i64, (), ()> =
        CausalEffectPropagationProcess::new(Ok(CausalEffect::value(7)), (), None, EffectLog::new());
    // wrap-then-unwrap is the identity.
    assert_eq!(CausalFlow::from(p.clone()).into_process(), p);
}

// ---- core.causal_flow.map_id -------------------------------------------------------------------

/// THEOREM_MAP: core.causal_flow.map_id
#[test]
fn test_causal_flow_map_id() {
    assert_eq!(CausalFlow::value(5_i64).map(|x| x).finish(), Ok(5));
    // Total on the None effect too: map id leaves it value-less, not errored.
    assert!(!none_flow().map(|x| x).is_err());
    assert!(none_flow().map(|x| x).finish().is_err()); // no value to finish with
}

// ---- core.causal_flow.map_comp -----------------------------------------------------------------

/// THEOREM_MAP: core.causal_flow.map_comp
#[test]
fn test_causal_flow_map_comp() {
    let f = |x: i64| x + 1;
    let g = |x: i64| x * 2;
    let composed = CausalFlow::value(3_i64).map(move |x| g(f(x))).finish();
    let sequential = CausalFlow::value(3_i64).map(f).map(g).finish();
    assert_eq!(composed, sequential);
    assert_eq!(composed, Ok(8));
}

// ---- core.causal_flow.map_eq_andThen -----------------------------------------------------------

/// THEOREM_MAP: core.causal_flow.map_eq_andThen
#[test]
fn test_causal_flow_map_eq_and_then() {
    let f = |x: i64| x + 10;

    // On a value: map f == and_then(pure ∘ f).
    let via_map = CausalFlow::value(5_i64).map(f).finish();
    let via_and_then = CausalFlow::value(5_i64)
        .and_then(|v, _s, _c| CausalFlow::value(f(v)))
        .finish();
    assert_eq!(via_map, via_and_then);
    assert_eq!(via_map, Ok(15));

    // D14: they also agree on the None effect — both pass None through, neither errors.
    let map_none = none_flow().map(f);
    let and_then_none = none_flow().and_then(|v, _s, _c| CausalFlow::value(f(v)));
    assert_eq!(map_none.is_err(), and_then_none.is_err());
    assert!(!map_none.is_err());
    assert!(map_none.finish().is_err() && and_then_none.finish().is_err());
}

// ---- core.causal_flow.recover ------------------------------------------------------------------

/// THEOREM_MAP: core.causal_flow.recover
#[test]
fn test_causal_flow_recover() {
    // No-op on success.
    assert_eq!(CausalFlow::value(7_i64).recover(|_| 0).finish(), Ok(7));
    // Raise ↦ handler value.
    let err = deep_causality_core::CausalityError::new(
        deep_causality_core::CausalityErrorEnum::ValueNotAvailable,
    );
    assert_eq!(
        CausalFlow::<i64>::fail(err).recover(|_| 42).finish(),
        Ok(42)
    );
}

// ---- core.causal_flow.iterate ------------------------------------------------------------------

/// THEOREM_MAP: core.causal_flow.iterate
#[test]
fn test_causal_flow_iterate() {
    // Converges within the budget: stops when the predicate holds.
    let converged = CausalFlow::value(0_i64)
        .iterate_until(|v| *v >= 3, 10, |f| f.map(|x| x + 1))
        .finish();
    assert_eq!(converged, Ok(3));

    // Budget exhausted without the predicate ⇒ MaxStepsExceeded (error channel).
    let exhausted = CausalFlow::value(0_i64).iterate_until(|v| *v > 100, 3, |f| f.map(|x| x + 1));
    assert!(exhausted.is_err());
}

// ---- core.causal_flow.finish -------------------------------------------------------------------

/// THEOREM_MAP: core.causal_flow.finish
#[test]
fn test_causal_flow_finish() {
    // `finish` depends only on the outcome — two carriers with the same value but different
    // state/context/log finish identically (state/context/log are dropped).
    let mut log_a = EffectLog::new();
    use deep_causality_haft::LogAddEntry;
    log_a.add_entry("a");
    let a: PropagatingProcess<i64, i64, String> = CausalEffectPropagationProcess::new(
        Ok(CausalEffect::value(7)),
        1,
        Some("x".to_string()),
        log_a,
    );
    let b: PropagatingProcess<i64, i64, String> = CausalEffectPropagationProcess::new(
        Ok(CausalEffect::value(7)),
        999,
        Some("different".to_string()),
        EffectLog::new(),
    );
    assert_eq!(CausalFlow::from(a).finish(), CausalFlow::from(b).finish());
    assert_eq!(CausalFlow::from(with_value(7)).finish(), Ok(7));
}

fn with_value(v: i64) -> PropagatingProcess<i64, i64, String> {
    CausalEffectPropagationProcess::new(Ok(CausalEffect::value(v)), 0, None, EffectLog::new())
}
