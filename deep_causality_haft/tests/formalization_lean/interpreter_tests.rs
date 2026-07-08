/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the one-way interpreter `ArrowTerm → Kleisli<M>`.
//!
//! Mirrors `lean/DeepCausalityFormal/Haft/Interpreter.lean`. Three facts:
//!
//! * `haft.interpreter.preserves_id` — `interpret_kleisli(Id) = Kleisli::id = pure`.
//! * `haft.interpreter.preserves_compose` — `interpret_kleisli(Compose f g)` equals the Kleisli
//!   composition `Kleisli::compose(interpret_kleisli f, interpret_kleisli g)`.
//! * `haft.interpreter.naturality` — the natural-transformation square for `OptionToVec`:
//!   `transform ∘ fmap f = fmap f ∘ transform`.
//!
//! The interpreter is exercised over `OptionWitness` with a *partial* generator (`Halve` fails on
//! odd inputs), so `bind`'s short-circuit is on the path both laws must respect.

use deep_causality_haft::{
    ArrowCore, ArrowVal, Category, Functor, Kleisli, NaturalTransformation, OptionToVec,
    OptionWitness, Pure, VecWitness,
};

// ---- generator set and its Kleisli interpretation (into Option) --------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
enum Op {
    Inc,
    Double,
    Halve,
}

/// A partial generator interpretation: `Halve` fails (`None`) on odd inputs.
fn phi(op: &Op, x: i64) -> Option<i64> {
    match op {
        Op::Inc => Some(x + 1),
        Op::Double => Some(x * 2),
        Op::Halve => {
            if x % 2 == 0 {
                Some(x / 2)
            } else {
                None
            }
        }
    }
}

/// THEOREM_MAP: haft.interpreter.preserves_id
///
/// The interpreter sends the identity term to the Kleisli identity (`pure`), on every input shape.
#[test]
fn test_interpreter_preserves_id() {
    let id_term: ArrowCore<Op> = ArrowCore::Id;
    let inputs = [
        ArrowVal::Leaf(7),
        ArrowVal::pair(ArrowVal::Leaf(1), ArrowVal::Leaf(2)),
    ];
    for input in inputs {
        // interpret_kleisli(Id) = pure
        assert_eq!(
            id_term.interpret_kleisli::<OptionWitness, i64, _>(&phi, input.clone()),
            <OptionWitness as Pure<OptionWitness>>::pure(input.clone())
        );
        // ... which is exactly Kleisli::id.
        let kid = <Kleisli<OptionWitness> as Category>::id::<ArrowVal<i64>>();
        assert_eq!(
            id_term.interpret_kleisli::<OptionWitness, i64, _>(&phi, input.clone()),
            kid(input)
        );
    }
}

/// THEOREM_MAP: haft.interpreter.preserves_compose
///
/// The interpreter is functorial: interpreting a composite equals composing the interpretations in
/// the Kleisli category (`bind`), including when the effect short-circuits.
#[test]
fn test_interpreter_preserves_compose() {
    let f: ArrowCore<Op> = ArrowCore::Gen(Op::Halve);
    let g: ArrowCore<Op> = ArrowCore::Gen(Op::Inc);
    let composite = ArrowCore::Compose(Box::new(f.clone()), Box::new(g.clone()));

    for x in [4, 5, 8, 3] {
        // interpret(Compose f g)
        let lhs = composite.interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::Leaf(x));

        // Kleisli::compose(interpret f, interpret g)
        let kf = |v: ArrowVal<i64>| f.interpret_kleisli::<OptionWitness, i64, _>(&phi, v);
        let kg = |v: ArrowVal<i64>| g.interpret_kleisli::<OptionWitness, i64, _>(&phi, v);
        let composed = <Kleisli<OptionWitness> as Category>::compose::<
            ArrowVal<i64>,
            ArrowVal<i64>,
            ArrowVal<i64>,
        >(kf, kg);
        let rhs = composed(ArrowVal::Leaf(x));

        assert_eq!(lhs, rhs);
    }

    // Spot-check the actual values: Halve then Inc.
    assert_eq!(
        composite.interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::Leaf(4)),
        Some(ArrowVal::Leaf(3)) // 4 -> 2 -> 3
    );
    assert_eq!(
        composite.interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::Leaf(5)),
        None // 5 is odd -> Halve fails -> short-circuit
    );

    // A totally-defined composite (Inc then Double) never short-circuits.
    let inc_double = ArrowCore::Compose(
        Box::new(ArrowCore::Gen(Op::Inc)),
        Box::new(ArrowCore::Gen(Op::Double)),
    );
    assert_eq!(
        inc_double.interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::Leaf(5)),
        Some(ArrowVal::Leaf(12)) // 5 -> 6 -> 12
    );
}

/// THEOREM_MAP: haft.interpreter.naturality
///
/// The `Option ⇒ Vec` component commutes with `fmap`: transforming then mapping equals mapping then
/// transforming (the naturality square).
#[test]
fn test_interpreter_naturality() {
    let f = |x: i64| x + 100;
    for opt in [Some(3), None] {
        // transform ∘ fmap f
        let lhs = OptionToVec::transform(<OptionWitness as Functor<OptionWitness>>::fmap(opt, f));
        // fmap f ∘ transform
        let rhs = <VecWitness as Functor<VecWitness>>::fmap(OptionToVec::transform(opt), f);
        assert_eq!(lhs, rhs);
    }
}
