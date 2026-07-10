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

/// THEOREM_MAP: haft.interpreter.choice_preserved
///
/// `interpret_kleisli` preserves the choice generators: the effect runs only on the taken branch
/// (`Left`/`Right`/`Choice` re-inject, `Fanin` eliminates), the untaken branch's effect never
/// fires, and a failing branch short-circuits exactly when taken.
#[test]
fn test_interpreter_choice_preserved() {
    use deep_causality_haft::Monad;

    // choice(halve, inc): the partial branch fails only when routed to.
    let choice_term: ArrowCore<Op> = ArrowCore::Choice(
        Box::new(ArrowCore::Gen(Op::Halve)),
        Box::new(ArrowCore::Gen(Op::Inc)),
    );
    // InL(even) takes the halve branch and succeeds.
    assert_eq!(
        choice_term
            .interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::inl(ArrowVal::Leaf(6))),
        Some(ArrowVal::inl(ArrowVal::Leaf(3)))
    );
    // InL(odd) takes the halve branch and short-circuits.
    assert_eq!(
        choice_term
            .interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::inl(ArrowVal::Leaf(7))),
        None
    );
    // InR(odd) never touches the failing branch: the untaken effect does not fire.
    assert_eq!(
        choice_term
            .interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::inr(ArrowVal::Leaf(7))),
        Some(ArrowVal::inr(ArrowVal::Leaf(8)))
    );

    // left/right agree with choice(f, id)/choice(id, f) — the preservation equations.
    let left_term: ArrowCore<Op> = ArrowCore::Left(Box::new(ArrowCore::Gen(Op::Halve)));
    let left_as_choice: ArrowCore<Op> =
        ArrowCore::Choice(Box::new(ArrowCore::Gen(Op::Halve)), Box::new(ArrowCore::Id));
    for val in [
        ArrowVal::inl(ArrowVal::Leaf(6)),
        ArrowVal::inl(ArrowVal::Leaf(7)),
        ArrowVal::inr(ArrowVal::Leaf(7)),
    ] {
        assert_eq!(
            left_term.interpret_kleisli::<OptionWitness, i64, _>(&phi, val.clone()),
            left_as_choice.interpret_kleisli::<OptionWitness, i64, _>(&phi, val)
        );
    }

    // Fanin eliminates: the branch result IS the output (no re-injection), and composing after a
    // fanin equals binding the branch result — Kleisli composition through the elimination.
    let fanin_then_inc: ArrowCore<Op> = ArrowCore::Compose(
        Box::new(ArrowCore::Fanin(
            Box::new(ArrowCore::Gen(Op::Halve)),
            Box::new(ArrowCore::Gen(Op::Double)),
        )),
        Box::new(ArrowCore::Gen(Op::Inc)),
    );
    assert_eq!(
        fanin_then_inc
            .interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::inl(ArrowVal::Leaf(6))),
        Some(ArrowVal::Leaf(4)) // 6 -> halve -> 3 -> inc -> 4
    );
    let via_bind = <OptionWitness as Monad<OptionWitness>>::bind(
        ArrowCore::<Op>::Fanin(
            Box::new(ArrowCore::Gen(Op::Halve)),
            Box::new(ArrowCore::Gen(Op::Double)),
        )
        .interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::inr(ArrowVal::Leaf(5))),
        |mid| ArrowCore::<Op>::Gen(Op::Inc).interpret_kleisli::<OptionWitness, i64, _>(&phi, mid),
    );
    assert_eq!(
        fanin_then_inc
            .interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::inr(ArrowVal::Leaf(5))),
        via_bind // 5 -> double -> 10 -> inc -> 11
    );
}

/// THEOREM_MAP: haft.interpreter.preserves_compose
///
/// The strength combinators thread the effect over the two halves of a [`ArrowVal::Pair`]:
/// `First`/`Second`/`Split` act componentwise, `Fanout` copies the input first. A failing half
/// short-circuits the whole pair (`bind`), and a mis-shaped (non-pair) input passes through purely.
#[test]
fn test_interpreter_strength_preserved() {
    // First(Double) acts on the left half of a pair, passing the right through.
    let first_double: ArrowCore<Op> = ArrowCore::First(Box::new(ArrowCore::Gen(Op::Double)));
    assert_eq!(
        first_double.interpret_kleisli::<OptionWitness, i64, _>(
            &phi,
            ArrowVal::pair(ArrowVal::Leaf(5), ArrowVal::Leaf(9)),
        ),
        Some(ArrowVal::pair(ArrowVal::Leaf(10), ArrowVal::Leaf(9)))
    );
    // First on a non-pair (Leaf) passes through unchanged — the effect never fires.
    assert_eq!(
        first_double.interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::Leaf(5)),
        Some(ArrowVal::Leaf(5))
    );

    // Second(Inc) acts on the right half of a pair, passing the left through.
    let second_inc: ArrowCore<Op> = ArrowCore::Second(Box::new(ArrowCore::Gen(Op::Inc)));
    assert_eq!(
        second_inc.interpret_kleisli::<OptionWitness, i64, _>(
            &phi,
            ArrowVal::pair(ArrowVal::Leaf(5), ArrowVal::Leaf(9)),
        ),
        Some(ArrowVal::pair(ArrowVal::Leaf(5), ArrowVal::Leaf(10)))
    );
    // Second on a non-pair (Leaf) passes through unchanged.
    assert_eq!(
        second_inc.interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::Leaf(9)),
        Some(ArrowVal::Leaf(9))
    );

    // Split(Inc, Double) acts on both halves independently.
    let split_term: ArrowCore<Op> = ArrowCore::Split(
        Box::new(ArrowCore::Gen(Op::Inc)),
        Box::new(ArrowCore::Gen(Op::Double)),
    );
    assert_eq!(
        split_term.interpret_kleisli::<OptionWitness, i64, _>(
            &phi,
            ArrowVal::pair(ArrowVal::Leaf(5), ArrowVal::Leaf(9)),
        ),
        Some(ArrowVal::pair(ArrowVal::Leaf(6), ArrowVal::Leaf(18)))
    );
    // Split on a non-pair (Leaf) passes through unchanged.
    assert_eq!(
        split_term.interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::Leaf(5)),
        Some(ArrowVal::Leaf(5))
    );

    // Fanout(Inc, Double) copies the input, feeds both arms, and pairs the results.
    let fanout_term: ArrowCore<Op> = ArrowCore::Fanout(
        Box::new(ArrowCore::Gen(Op::Inc)),
        Box::new(ArrowCore::Gen(Op::Double)),
    );
    assert_eq!(
        fanout_term.interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::Leaf(7)),
        Some(ArrowVal::pair(ArrowVal::Leaf(8), ArrowVal::Leaf(14)))
    );

    // A failing half short-circuits the whole pair: First(Halve) on an odd left half -> None.
    let first_halve: ArrowCore<Op> = ArrowCore::First(Box::new(ArrowCore::Gen(Op::Halve)));
    assert_eq!(
        first_halve.interpret_kleisli::<OptionWitness, i64, _>(
            &phi,
            ArrowVal::pair(ArrowVal::Leaf(7), ArrowVal::Leaf(2)),
        ),
        None
    );
    // ... but succeeds on an even left half.
    assert_eq!(
        first_halve.interpret_kleisli::<OptionWitness, i64, _>(
            &phi,
            ArrowVal::pair(ArrowVal::Leaf(8), ArrowVal::Leaf(2)),
        ),
        Some(ArrowVal::pair(ArrowVal::Leaf(4), ArrowVal::Leaf(2)))
    );
}

/// THEOREM_MAP: haft.interpreter.preserves_id
///
/// A combinator fed a value of the wrong shape passes it through purely (`pure`), never firing the
/// effect: `Gen` on a non-leaf, `Right` on a non-`InR`, and `Choice`/`Fanin` on a non-sum.
#[test]
fn test_interpreter_shape_mismatch_passthrough() {
    // Gen on a Pair (non-leaf) passes through unchanged — the generator never fires.
    let gen_inc: ArrowCore<Op> = ArrowCore::Gen(Op::Inc);
    let pair = ArrowVal::pair(ArrowVal::Leaf(3), ArrowVal::Leaf(4));
    assert_eq!(
        gen_inc.interpret_kleisli::<OptionWitness, i64, _>(&phi, pair.clone()),
        Some(pair)
    );
    // A Gen(Halve) that WOULD fail on a leaf still passes an InL through untouched (no effect).
    let gen_halve: ArrowCore<Op> = ArrowCore::Gen(Op::Halve);
    assert_eq!(
        gen_halve
            .interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::inl(ArrowVal::Leaf(7))),
        Some(ArrowVal::inl(ArrowVal::Leaf(7)))
    );

    // Right(Inc) acts on InR only; an InL injection passes through unchanged.
    let right_inc: ArrowCore<Op> = ArrowCore::Right(Box::new(ArrowCore::Gen(Op::Inc)));
    assert_eq!(
        right_inc
            .interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::inr(ArrowVal::Leaf(9))),
        Some(ArrowVal::inr(ArrowVal::Leaf(10)))
    );
    assert_eq!(
        right_inc
            .interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::inl(ArrowVal::Leaf(9))),
        Some(ArrowVal::inl(ArrowVal::Leaf(9)))
    );

    // Choice on a non-sum (Leaf) passes through — neither branch is taken.
    let choice_term: ArrowCore<Op> = ArrowCore::Choice(
        Box::new(ArrowCore::Gen(Op::Halve)),
        Box::new(ArrowCore::Gen(Op::Inc)),
    );
    assert_eq!(
        choice_term.interpret_kleisli::<OptionWitness, i64, _>(&phi, ArrowVal::Leaf(11)),
        Some(ArrowVal::Leaf(11))
    );

    // Fanin on a non-sum (Pair) passes through — no injection to eliminate.
    let fanin_term: ArrowCore<Op> = ArrowCore::Fanin(
        Box::new(ArrowCore::Gen(Op::Halve)),
        Box::new(ArrowCore::Gen(Op::Double)),
    );
    let pair2 = ArrowVal::pair(ArrowVal::Leaf(1), ArrowVal::Leaf(2));
    assert_eq!(
        fanin_term.interpret_kleisli::<OptionWitness, i64, _>(&phi, pair2.clone()),
        Some(pair2)
    );
}
