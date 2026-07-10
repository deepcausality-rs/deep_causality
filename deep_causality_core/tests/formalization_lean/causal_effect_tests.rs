/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the success-channel laws of `CausalEffect`.
//!
//! Mirrors `lean/DeepCausalityFormal/Core/CausalEffect.lean`. The value content is `Option<V>`,
//! whose functor laws are `haft.functor.laws` (proved in `Haft/Functor.lean`); this witness checks
//! (1) the honest `Maybe` projection `into_value` (`Pure(Some v) ↦ Some v`, `Pure(None)`/command
//! `↦ None`) — the `core.causal_effect.into_value` id — and (2) that the total `CausalEffect::map`
//! lifts the `Option` functor through the value leaves. The 18 behavioural unit tests in
//! `tests/types/causal_effect/` cover the rest of the API.
//!
//! Stage-1 carrier-stack witnesses (`causaloid-formalization-roadmap.md`):
//! * `core.causal_effect.transformer_stack` — `try_and_then` is the lawful composite bind of
//!   `Except E (Free CausalCommand (Maybe V))`: left/right identity, associativity, the `Err`
//!   global left zero, the `None` local zero, and relay threading with error hoisting.
//! * `core.causal_effect.fold_universal` — `fold` satisfies the two handler equations and any
//!   handler-shaped function agrees with it (initiality, spot-checked).
//! * `core.causal_effect.relay_termination` — the fuel-bounded relay loop is total: fuel-monotone
//!   on answers, and a self-relay cycle exhausts instead of hanging (the engine realization is
//!   `MAX_RELAY_ROUNDS` in `deep_causality::…::graph_reasoning`, tested behaviourally there).

use deep_causality_core::CausalEffect;

// ---- core.causal_effect.into_value : the honest Maybe projection -------------------------------

/// THEOREM_MAP: core.causal_effect.into_value
#[test]
fn test_causal_effect_into_value() {
    // Pure(Some v) ↦ Some v
    assert_eq!(CausalEffect::value(7i64).into_value(), Some(7));
    // Pure(None) ↦ None
    assert_eq!(CausalEffect::<i64>::none().into_value(), None);
    // command (RelayTo) ↦ None — honest: a command carries no value (no payload-drop caveat).
    let command = CausalEffect::relay_to(2, CausalEffect::value(5i64));
    assert!(command.is_command());
    assert_eq!(command.into_value(), None);
}

// ---- value functor = the Option functor (haft.functor.laws) ------------------------------------

/// The value functor is `Option` (`haft.functor.laws`): `map` applies `f` to the `Some` leaf and
/// passes `None` through, and `map id = id` — no bespoke value-type functor, no panic.
#[test]
fn test_causal_effect_value_functor_is_option() {
    assert_eq!(
        CausalEffect::value(3i64).map(|x| x + 1).into_value(),
        Some(4)
    );
    assert_eq!(
        CausalEffect::<i64>::none().map(|x| x + 1).into_value(),
        None
    );
    // Identity law lifted through the leaf.
    assert_eq!(CausalEffect::value(9i64).map(|x| x).into_value(), Some(9));
    // Total & uniform on a command: maps the sub-program's leaves, stays a command (no panic).
    let mapped = CausalEffect::relay_to(1, CausalEffect::value(4i64)).map(|x| x + 10);
    assert!(mapped.is_command());
    assert_eq!(mapped.command_target(), Some(1));
}

// ---- core.causal_effect.transformer_stack : the composite outcome monad ------------------------

/// The composite pure: `Ok(Pure(Some v))`.
fn opure(v: i64) -> Result<CausalEffect<i64>, String> {
    Ok(CausalEffect::value(v))
}

/// The composite bind (`obind`): `Err` is the outermost zero; the success layer is
/// `CausalEffect::try_and_then`.
fn obind<F>(m: Result<CausalEffect<i64>, String>, k: F) -> Result<CausalEffect<i64>, String>
where
    F: FnOnce(i64) -> Result<CausalEffect<i64>, String>,
{
    m.and_then(|eff| eff.try_and_then(k))
}

/// Representative outcomes across every shape of the stack: error, `None`, value, single relay,
/// nested relay chain.
fn stack_samples() -> Vec<Result<CausalEffect<i64>, String>> {
    vec![
        Err("boom".to_string()),
        Ok(CausalEffect::none()),
        Ok(CausalEffect::value(4)),
        Ok(CausalEffect::relay_to(2, CausalEffect::value(5))),
        Ok(CausalEffect::relay_to(
            1,
            CausalEffect::relay_to(3, CausalEffect::value(6)),
        )),
    ]
}

/// THEOREM_MAP: core.causal_effect.transformer_stack
#[test]
fn test_transformer_stack_monad_laws() {
    let f = |x: i64| opure(x + 1);
    let g = |x: i64| {
        if x % 2 == 0 {
            opure(x * 2)
        } else {
            Err(format!("odd: {x}"))
        }
    };

    // Left identity: obind(opure v, f) = f v.
    for v in [0i64, 3, -7] {
        assert_eq!(obind(opure(v), f), f(v));
        assert_eq!(obind(opure(v), g), g(v));
    }

    // Right identity: obind(m, opure) = m — across every outcome shape.
    for m in stack_samples() {
        assert_eq!(obind(m.clone(), opure), m);
    }

    // Associativity: obind(obind(m, f), g) = obind(m, |v| obind(f(v), g)) — including the case
    // where the inner continuation errors under a relay node (error hoisting).
    for m in stack_samples() {
        assert_eq!(obind(obind(m.clone(), f), g), obind(m, |v| obind(f(v), g)));
    }
}

/// THEOREM_MAP: core.causal_effect.transformer_stack
#[test]
fn test_transformer_stack_zeros_and_relay_threading() {
    let k = |x: i64| opure(x + 1);

    // `Err` is the global left zero.
    assert_eq!(obind(Err("e".to_string()), k), Err("e".to_string()));

    // `None` is a local zero: preserved, the continuation never runs.
    let bound = CausalEffect::<i64>::none()
        .try_and_then::<i64, String, _>(|_| panic!("continuation must not run on None"));
    assert_eq!(bound, Ok(CausalEffect::none()));

    // Relay threading: the bind goes under the command, the target is preserved.
    let relayed = CausalEffect::relay_to(2, CausalEffect::value(5)).try_and_then(k);
    assert_eq!(
        relayed,
        Ok(CausalEffect::relay_to(2, CausalEffect::value(6)))
    );

    // Error hoisting: a failure *inside* the relayed sub-program aborts the whole outcome.
    let hoisted = CausalEffect::relay_to(2, CausalEffect::value(5))
        .try_and_then(|_| Err::<CausalEffect<i64>, _>("inner".to_string()));
    assert_eq!(hoisted, Err("inner".to_string()));

    // The pure `Free ∘ Maybe` layer (`and_then`) agrees with `try_and_then` on the success side.
    let pure_layer =
        CausalEffect::relay_to(2, CausalEffect::value(5)).and_then(|x| CausalEffect::value(x + 1));
    assert_eq!(
        Ok(pure_layer),
        CausalEffect::relay_to(2, CausalEffect::value(5))
            .try_and_then::<i64, String, _>(|x| Ok(CausalEffect::value(x + 1)))
    );
}

// ---- core.causal_effect.fold_universal : the handler is the unique interpreter -----------------

/// THEOREM_MAP: core.causal_effect.fold_universal
#[test]
fn test_fold_universal() {
    // The two handler equations, at a representative pure case and algebra: count the relay depth,
    // starting from whether a value is present.
    let pure_case = |o: Option<i64>| if o.is_some() { 1usize } else { 0 };
    let algebra = |_t: usize, x: usize| x + 1;

    // fold (pure o) = pure_case o
    assert_eq!(CausalEffect::value(9).fold(&pure_case, &algebra), 1);
    assert_eq!(CausalEffect::<i64>::none().fold(&pure_case, &algebra), 0);

    // fold (relay t sub) = algebra t (fold sub)
    let chain = CausalEffect::relay_to(1, CausalEffect::relay_to(3, CausalEffect::value(6)));
    assert_eq!(chain.fold(&pure_case, &algebra), 3); // 1 (value) + 2 (relay nodes)

    // Uniqueness (initiality), spot-checked: an independently written handler-shaped recursion
    // agrees with fold on every sample — the interpreter is determined by (pure_case, algebra).
    fn by_hand(e: CausalEffect<i64>) -> usize {
        if e.is_command() {
            let (_t, sub) = e.into_command().expect("is_command");
            by_hand(sub) + 1
        } else {
            usize::from(e.into_value().is_some())
        }
    }
    let samples = [
        CausalEffect::value(1),
        CausalEffect::<i64>::none(),
        CausalEffect::relay_to(0, CausalEffect::none()),
        CausalEffect::relay_to(1, CausalEffect::relay_to(2, CausalEffect::value(5))),
    ];
    for s in samples {
        assert_eq!(s.clone().fold(&pure_case, &algebra), by_hand(s));
    }
}

// ---- core.causal_effect.relay_termination : the fuel-bounded relay loop is total ---------------

/// The Lean `run` transcribed over the real `CausalEffect`: a value answers, a relay consumes one
/// unit of fuel and re-enters with the step function's next program, exhaustion reports `None`.
/// (The engine realization is the `MAX_RELAY_ROUNDS` bound in the graph-reasoning loops.)
fn run<G>(fuel: usize, m: CausalEffect<i64>, g: &G) -> Option<Option<i64>>
where
    G: Fn(usize, CausalEffect<i64>) -> CausalEffect<i64>,
{
    if fuel == 0 {
        return None;
    }
    if m.is_command() {
        let (t, sub) = m.into_command().expect("is_command");
        run(fuel - 1, g(t, sub), g)
    } else {
        Some(m.into_value())
    }
}

/// THEOREM_MAP: core.causal_effect.relay_termination
#[test]
fn test_relay_termination_fuel_bound() {
    // A step function that resolves after two hops: target 0 answers, other targets relay onward.
    let resolving = |t: usize, sub: CausalEffect<i64>| {
        if t == 0 {
            sub
        } else {
            CausalEffect::relay_to(t - 1, sub)
        }
    };

    // A value answers immediately at any nonzero fuel.
    assert_eq!(run(1, CausalEffect::value(7), &resolving), Some(Some(7)));

    // A finite relay chain answers within its length...
    let chain = CausalEffect::relay_to(2, CausalEffect::value(9));
    assert_eq!(run(4, chain.clone(), &resolving), Some(Some(9)));
    // ...and fuel monotonicity: more fuel never changes an answer.
    assert_eq!(run(64, chain, &resolving), Some(Some(9)));

    // The self-relay cycle exhausts for EVERY fuel — the unbounded loop is cut, never hung.
    let self_relay = |t: usize, sub: CausalEffect<i64>| CausalEffect::relay_to(t, sub);
    for fuel in [1usize, 2, 8, 64] {
        assert_eq!(
            run(
                fuel,
                CausalEffect::relay_to(1, CausalEffect::value(0)),
                &self_relay
            ),
            None
        );
    }
}
