/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witness for the functor-agreement law (`core.witness.agree`).
//!
//! Mirrors `lean/DeepCausalityFormal/Core/Consistency.lean`. The HKT process witness
//! `CausalEffectPropagationProcessWitness::fmap` and the inherent `CausalFlow::map` must produce the
//! same result on every carrier of the value fragment — value (`Some`), absence (`None`), and error
//! (`Err`). Deviation D15 is retired: the former arity-5 `.expect` panic and the four-way `fmap`
//! divergence are gone, so this is checked with NO `Ok(Value _)`-only restriction and no panic path.

use deep_causality_core::{
    CausalEffect, CausalEffectPropagationProcess, CausalEffectPropagationProcessWitness,
    CausalFlow, CausalityError, CausalityErrorEnum, EffectLog,
};
use deep_causality_haft::Functor;

type Witness = CausalEffectPropagationProcessWitness<(), (), CausalityError, EffectLog>;

/// A stateless carrier with the given outcome.
fn carrier(
    outcome: Result<CausalEffect<i64>, CausalityError>,
) -> CausalEffectPropagationProcess<i64, (), (), CausalityError, EffectLog> {
    CausalEffectPropagationProcess::new(outcome, (), None, EffectLog::new())
}

// ---- core.witness.agree : witness fmap = inherent fmap on every carrier ------------------------

/// THEOREM_MAP: core.witness.agree
#[test]
fn test_witness_agrees_with_inherent_fmap() {
    let f = |x: i64| x + 1;

    // Value carrier: both map the leaf to `Some(3)`.
    let witnessed = CausalFlow::from(Witness::fmap(carrier(Ok(CausalEffect::value(2))), f))
        .finish()
        .ok();
    let inherent = CausalFlow::from(carrier(Ok(CausalEffect::value(2))))
        .map(f)
        .finish()
        .ok();
    assert_eq!(witnessed, inherent);
    assert_eq!(inherent, Some(3));
}

#[test]
fn test_witness_agrees_on_none() {
    let f = |x: i64| x + 1;

    // None carrier: both stay `Ok(None)` — not an error, but no value to finish with.
    let witnessed = CausalFlow::from(Witness::fmap(carrier(Ok(CausalEffect::none())), f));
    let inherent = CausalFlow::from(carrier(Ok(CausalEffect::none()))).map(f);

    assert_eq!(witnessed.is_err(), inherent.is_err());
    assert!(!inherent.is_err()); // `Ok(None)` is not the error channel …
    assert!(witnessed.finish().is_err()); // … but has no value (both agree).
}

#[test]
fn test_witness_agrees_on_error() {
    let f = |x: i64| x + 1;
    let err = || CausalityError::new(CausalityErrorEnum::ValueNotAvailable);

    // Error carrier: `f` is not invoked (left zero); both stay in the error channel.
    let witnessed = CausalFlow::from(Witness::fmap(carrier(Err(err())), f));
    let inherent = CausalFlow::from(carrier(Err(err()))).map(f);

    assert_eq!(witnessed.is_err(), inherent.is_err());
    assert!(witnessed.is_err());
}
