/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the reified free Arrow (`ArrowTerm`).
//!
//! Mirrors `lean/DeepCausalityFormal/Haft/ArrowTerm.lean` (Hughes 2000; Awodey §5, initial
//! algebras / free objects). Two laws:
//!
//! * `haft.arrow_term.interpret_sound` — interpreting a term over [`ArrowVal`] equals running the
//!   *same* pipeline built from the eager [`Arrow`] combinators (`Lift`/`Compose`/`Split`/…). The
//!   erased core is a faithful reification: reifying then interpreting is the identity on behaviour.
//! * `haft.arrow_term.free` — the interpretation is *determined by the generators*: two
//!   interpreters that agree on every generator produce equal results on every term (universal
//!   property of the free arrow), and changing a generator's action changes the result.
//!
//! Also checks that the typed builder lowers to the expected erased [`ArrowCore`] value
//! (structural round-trip) and that mistyped wiring is rejected at compile time (the `compile_fail`
//! doctest lives on `ArrowTerm` in `src/arrow/arrow_term.rs`).

use deep_causality_haft::{Arrow, ArrowCore, ArrowTerm, ArrowVal, Lift};

// ---- generator set and its interpretation ------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
enum Op {
    Inc,
    Double,
    Neg,
}

fn inc(x: i64) -> i64 {
    x + 1
}
fn double(x: i64) -> i64 {
    x * 2
}
fn neg(x: i64) -> i64 {
    -x
}

/// The canonical interpretation of the generators as leaf endomorphisms.
fn interp(op: &Op, x: i64) -> i64 {
    match op {
        Op::Inc => inc(x),
        Op::Double => double(x),
        Op::Neg => neg(x),
    }
}

// ---- ArrowVal helpers --------------------------------------------------------------------------

fn as_pair(v: ArrowVal<i64>) -> (i64, i64) {
    match v {
        ArrowVal::Pair(a, b) => match (*a, *b) {
            (ArrowVal::Leaf(x), ArrowVal::Leaf(y)) => (x, y),
            _ => panic!("expected a pair of leaves"),
        },
        ArrowVal::Leaf(_) | ArrowVal::InL(_) | ArrowVal::InR(_) => panic!("expected a pair"),
    }
}

/// THEOREM_MAP: haft.arrow_term.interpret_sound
///
/// For a representative term touching every combinator, interpreting the erased core reproduces the
/// output of the eager `Arrow` pipeline of the same shape.
#[test]
fn test_arrow_term_interpret_sound() {
    // Term: first(inc) >>> split(double, neg)  :  (i64, i64) -> (i64, i64)
    let term = ArrowTerm::<i64, i64, Op>::generator(Op::Inc)
        .first::<i64>()
        .compose(
            ArrowTerm::<i64, i64, Op>::generator(Op::Double)
                .split(ArrowTerm::<i64, i64, Op>::generator(Op::Neg)),
        );

    // The eager pipeline of identical shape over real `Arrow` combinators.
    let eager = Lift::new(inc)
        .first::<i64>()
        .compose(Lift::new(double).split(Lift::new(neg)));

    for (a, b) in [(10, 7), (0, 0), (-3, 4)] {
        let interpreted = term.core().interpret(
            &interp,
            ArrowVal::pair(ArrowVal::Leaf(a), ArrowVal::Leaf(b)),
        );
        assert_eq!(as_pair(interpreted), eager.run((a, b)));
    }

    // fanout(inc, double) >>> second(neg) : i64 -> (i64, i64), copy then act on the right.
    let term2 = ArrowTerm::<i64, i64, Op>::generator(Op::Inc)
        .fanout(ArrowTerm::generator(Op::Double))
        .compose(ArrowTerm::<i64, i64, Op>::generator(Op::Neg).second::<i64>());
    let eager2 = Lift::new(inc)
        .fanout(Lift::new(double))
        .compose(Lift::new(neg).second::<i64>());
    for x in [5, 0, -8] {
        let interpreted = term2.core().interpret(&interp, ArrowVal::Leaf(x));
        assert_eq!(as_pair(interpreted), eager2.run(x));
    }

    // The identity term is the identity function on any value.
    let idt = ArrowTerm::<i64, i64, Op>::id();
    assert_eq!(
        idt.core().interpret(&interp, ArrowVal::Leaf(42)),
        ArrowVal::Leaf(42)
    );
}

/// THEOREM_MAP: haft.arrow_term.free
///
/// The interpretation is determined entirely by the generators: interpreters agreeing on every
/// generator agree on every term, and changing a generator's action changes the result. This is
/// the universal property of the free arrow.
#[test]
fn test_arrow_term_free() {
    let term = ArrowTerm::<i64, i64, Op>::generator(Op::Inc)
        .first::<i64>()
        .compose(
            ArrowTerm::<i64, i64, Op>::generator(Op::Double)
                .split(ArrowTerm::<i64, i64, Op>::generator(Op::Neg)),
        );
    let input = || ArrowVal::pair(ArrowVal::Leaf(9), ArrowVal::Leaf(4));

    // A syntactically different interpreter that is extensionally equal to `interp` on the
    // generators — same result on the whole term (interpretation factors through the generators).
    fn interp_alt(op: &Op, x: i64) -> i64 {
        match op {
            Op::Inc => x - (-1), // = x + 1
            Op::Double => x + x, // = x * 2
            Op::Neg => 0 - x,    // = -x
        }
    }
    assert_eq!(
        term.core().interpret(&interp, input()),
        term.core().interpret(&interp_alt, input())
    );

    // An interpreter that disagrees on a *used* generator (`Double`) changes the result — the term
    // genuinely depends on the generators' action, so the free extension is faithful.
    fn interp_diff(op: &Op, x: i64) -> i64 {
        match op {
            Op::Double => x * 3, // differs
            other => interp(other, x),
        }
    }
    assert_ne!(
        term.core().interpret(&interp, input()),
        term.core().interpret(&interp_diff, input())
    );
}

/// THEOREM_MAP: haft.arrow_term.interpret_sound
///
/// Structural round-trip: the typed builder lowers to exactly the erased `ArrowCore` value it
/// denotes — the reification is inspectable data, not an opaque closure.
#[test]
fn test_arrow_term_core_round_trip() {
    let term = ArrowTerm::<i64, i64, Op>::generator(Op::Inc)
        .first::<i64>()
        .compose(
            ArrowTerm::<i64, i64, Op>::generator(Op::Double)
                .split(ArrowTerm::<i64, i64, Op>::generator(Op::Neg)),
        );

    let expected = ArrowCore::Compose(
        Box::new(ArrowCore::First(Box::new(ArrowCore::Gen(Op::Inc)))),
        Box::new(ArrowCore::Split(
            Box::new(ArrowCore::Gen(Op::Double)),
            Box::new(ArrowCore::Gen(Op::Neg)),
        )),
    );
    assert_eq!(term.into_core(), expected);
}

/// THEOREM_MAP: haft.arrow_term.choice_interpret_sound
///
/// Interpreting the choice generators agrees with the eager ArrowChoice combinators: `left`/
/// `right`/`choice` route the sum node component-wise and `fanin` eliminates it — for a
/// representative term, the erased core reproduces the eager `Either` pipeline of the same shape.
#[test]
fn test_arrow_term_choice_interpret_sound() {
    use deep_causality_haft::{Arrow, Either, Lift};

    // Term: choice(inc, double) >>> fanin(neg, id) : Either<i64, i64> -> i64.
    let term = ArrowTerm::<i64, i64, Op>::generator(Op::Inc)
        .choice(ArrowTerm::<i64, i64, Op>::generator(Op::Double))
        .compose(
            ArrowTerm::<i64, i64, Op>::generator(Op::Neg).fanin(ArrowTerm::<i64, i64, Op>::id()),
        );
    let eager = Lift::new(inc)
        .choice(Lift::new(double))
        .compose(Lift::new(neg).fanin(Lift::new(|x: i64| x)));

    for x in [0, 5, -7] {
        for (val, eff) in [
            (
                ArrowVal::inl(ArrowVal::Leaf(x)),
                Either::<i64, i64>::Left(x),
            ),
            (ArrowVal::inr(ArrowVal::Leaf(x)), Either::Right(x)),
        ] {
            let interpreted = term.core().interpret(&interp, val);
            assert_eq!(interpreted, ArrowVal::Leaf(eager.run(eff)));
        }
    }

    // left(inc) acts on InL only; the right injection passes through untouched.
    let left_term = ArrowTerm::<i64, i64, Op>::generator(Op::Inc).left::<i64>();
    assert_eq!(
        left_term
            .core()
            .interpret(&interp, ArrowVal::inl(ArrowVal::Leaf(4))),
        ArrowVal::inl(ArrowVal::Leaf(5))
    );
    assert_eq!(
        left_term
            .core()
            .interpret(&interp, ArrowVal::inr(ArrowVal::Leaf(4))),
        ArrowVal::inr(ArrowVal::Leaf(4))
    );

    // right(inc) is the mirror.
    let right_term = ArrowTerm::<i64, i64, Op>::generator(Op::Inc).right::<i64>();
    assert_eq!(
        right_term
            .core()
            .interpret(&interp, ArrowVal::inr(ArrowVal::Leaf(4))),
        ArrowVal::inr(ArrowVal::Leaf(5))
    );
    assert_eq!(
        right_term
            .core()
            .interpret(&interp, ArrowVal::inl(ArrowVal::Leaf(4))),
        ArrowVal::inl(ArrowVal::Leaf(4))
    );
}

/// THEOREM_MAP: haft.arrow_term.choice_free
///
/// The universal property extends to the enlarged generator set: interpretations agreeing on the
/// generators agree on every choice term, and changing a generator's action changes the routed
/// result.
#[test]
fn test_arrow_term_choice_free() {
    let term = ArrowTerm::<i64, i64, Op>::generator(Op::Inc)
        .choice(ArrowTerm::<i64, i64, Op>::generator(Op::Double))
        .compose(
            ArrowTerm::<i64, i64, Op>::generator(Op::Neg).fanin(ArrowTerm::<i64, i64, Op>::id()),
        );

    // An extensionally equal generator interpretation gives equal results on every injection.
    let interp_same = |op: &Op, x: i64| match op {
        Op::Inc => x + 1,
        Op::Double => x + x,
        Op::Neg => -x,
    };
    // A different action on `Double` changes the result reached through the right injection.
    let interp_diff = |op: &Op, x: i64| match op {
        Op::Inc => x + 1,
        Op::Double => x * 3,
        Op::Neg => -x,
    };

    for x in [0, 5, -7] {
        for val in [
            ArrowVal::inl(ArrowVal::Leaf(x)),
            ArrowVal::inr(ArrowVal::Leaf(x)),
        ] {
            assert_eq!(
                term.core().interpret(&interp, val.clone()),
                term.core().interpret(&interp_same, val)
            );
        }
    }
    assert_ne!(
        term.core()
            .interpret(&interp, ArrowVal::inr(ArrowVal::Leaf(5))),
        term.core()
            .interpret(&interp_diff, ArrowVal::inr(ArrowVal::Leaf(5)))
    );
}

/// THEOREM_MAP: haft.arrow_term.interpret_sound
///
/// A combinator applied to a value of the wrong shape passes it through unchanged — the mismatch the
/// typed [`ArrowTerm`] façade makes unreachable for well-wired terms: `Gen` on a non-leaf,
/// `First`/`Second`/`Split` on a non-pair, `Choice`/`Fanin` on a non-sum.
#[test]
fn test_arrow_term_shape_mismatch_passthrough() {
    // Gen on a Pair (non-leaf) passes through unchanged — the generator never fires.
    let gen_term: ArrowCore<Op> = ArrowCore::Gen(Op::Inc);
    let pair = ArrowVal::pair(ArrowVal::Leaf(3), ArrowVal::Leaf(4));
    assert_eq!(gen_term.interpret(&interp, pair.clone()), pair);

    // First / Second / Split on a Leaf (non-pair) pass through unchanged.
    let first: ArrowCore<Op> = ArrowCore::First(Box::new(ArrowCore::Gen(Op::Inc)));
    assert_eq!(
        first.interpret(&interp, ArrowVal::Leaf(5)),
        ArrowVal::Leaf(5)
    );

    let second: ArrowCore<Op> = ArrowCore::Second(Box::new(ArrowCore::Gen(Op::Inc)));
    assert_eq!(
        second.interpret(&interp, ArrowVal::Leaf(5)),
        ArrowVal::Leaf(5)
    );

    let split: ArrowCore<Op> = ArrowCore::Split(
        Box::new(ArrowCore::Gen(Op::Inc)),
        Box::new(ArrowCore::Gen(Op::Double)),
    );
    assert_eq!(
        split.interpret(&interp, ArrowVal::Leaf(5)),
        ArrowVal::Leaf(5)
    );

    // Choice / Fanin on a Leaf (non-sum) pass through unchanged.
    let choice: ArrowCore<Op> = ArrowCore::Choice(
        Box::new(ArrowCore::Gen(Op::Inc)),
        Box::new(ArrowCore::Gen(Op::Double)),
    );
    assert_eq!(
        choice.interpret(&interp, ArrowVal::Leaf(5)),
        ArrowVal::Leaf(5)
    );

    let fanin: ArrowCore<Op> = ArrowCore::Fanin(
        Box::new(ArrowCore::Gen(Op::Neg)),
        Box::new(ArrowCore::Gen(Op::Double)),
    );
    assert_eq!(
        fanin.interpret(&interp, ArrowVal::Leaf(5)),
        ArrowVal::Leaf(5)
    );
}
