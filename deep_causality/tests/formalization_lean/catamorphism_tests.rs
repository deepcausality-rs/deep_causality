/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for `lean/DeepCausalityFormal/Core/Catamorphism.lean` — the keystone
//! (Stage 5): `evaluate` is the unique F-algebra catamorphism per fixed carrier
//! (`catamorphism_unique`), encapsulation is flat (`encapsulation_flat`), and the
//! `Atom`/`compose` fragment is the reified term language with `evaluate = interpret` on it
//! (`arrow_fragment`, including the ⊕-enlarged generators). Lean proves ∀; these tests pin the
//! real engine to the same statements at representative inputs.

use deep_causality::utils_test::test_utils;
use deep_causality::{
    AggregateLogic, CausableGraph, Causaloid, CausaloidGraph, MonadicCausable,
    MonadicCausableCollection, MonadicCausableGraphReasoning, PropagatingEffect, Verdict,
};
use deep_causality_haft::{ArrowCore, ArrowTerm, ArrowVal};

/// THEOREM_MAP: core.causaloid.catamorphism_unique
///
/// Lean: `catamorphism_unique` (`Core/Catamorphism.lean`). Uniqueness spot-check: a BY-HAND
/// interpreter satisfying the algebra's case equations — atoms through the element semantics,
/// bags folded with the carrier's operations — agrees with the real evaluation everywhere it is
/// compared. The carrier is FIXED (`bool`, `All`/`Any` folds); uniqueness across carriers is
/// neither claimed nor true (per-carrier scoping, assumption #6).
#[test]
fn test_catamorphism_unique() {
    // Atom case equation: h(atom a) = elemSem a — the by-hand interpreter is the raw causal fn.
    let atom = test_utils::get_test_causaloid_deterministic_true();
    let input = PropagatingEffect::from_value(true);
    let by_hand_atom = atom.causal_fn().expect("element")(true);
    assert_eq!(atom.evaluate(&input).value(), by_hand_atom.value());

    // Bag case equations: h(coll cs) = fold of the members through the carrier ops. The by-hand
    // interpreter folds the members' raw outputs with meet (All) / join (Any); the real path is
    // `evaluate_collection`. Equal on every pattern — the interpreter satisfying the equations
    // IS the evaluation (initiality, spot-checked).
    let patterns: [&[bool]; 3] = [&[true, false, true], &[true, true], &[false]];
    for pattern in patterns {
        let bag: Vec<_> = pattern
            .iter()
            .map(|&b| {
                if b {
                    test_utils::get_test_causaloid_deterministic_true()
                } else {
                    test_utils::get_test_causaloid_deterministic_false()
                }
            })
            .collect();

        let by_hand_all = bag
            .iter()
            .map(|c| {
                c.causal_fn().expect("element")(true)
                    .value_cloned()
                    .expect("value")
            })
            .fold(bool::top(), bool::meet);
        let by_hand_any = bag
            .iter()
            .map(|c| {
                c.causal_fn().expect("element")(true)
                    .value_cloned()
                    .expect("value")
            })
            .fold(bool::bottom(), bool::join);

        let real_all = bag
            .evaluate_collection(&input, &AggregateLogic::All, Some(0.5))
            .value_cloned()
            .expect("All value");
        let real_any = bag
            .evaluate_collection(&input, &AggregateLogic::Any, Some(0.5))
            .value_cloned()
            .expect("Any value");

        assert_eq!(real_all, by_hand_all, "All over {pattern:?}");
        assert_eq!(real_any, by_hand_any, "Any over {pattern:?}");
    }
}

fn chain_graph(ids: &[u64]) -> CausaloidGraph<Causaloid<f64, f64, (), ()>> {
    fn add_one(x: f64) -> PropagatingEffect<f64> {
        PropagatingEffect::from_value(x + 1.0)
    }
    fn double(x: f64) -> PropagatingEffect<f64> {
        PropagatingEffect::from_value(x * 2.0)
    }
    let mut g = CausaloidGraph::new(0);
    let mut prev: Option<usize> = None;
    for &id in ids {
        // Alternate element maps so composition order is observable.
        let f = if id % 2 == 0 { add_one } else { double };
        let idx = g
            .add_causaloid(Causaloid::new(id, f, "chain node"))
            .expect("add node");
        if let Some(p) = prev {
            g.add_edge(p, idx).expect("edge");
        }
        prev = Some(idx);
    }
    g.freeze();
    g
}

/// THEOREM_MAP: core.causaloid.encapsulation_flat
///
/// Lean: `encapsulation_flat`, `evalL_append` (`Core/Catamorphism.lean`). Nested fold = flat
/// fold: evaluating the whole chain in one pass equals evaluating a prefix subgraph and feeding
/// its result into the suffix subgraph — the wrapped/two-stage evaluation and the flattened
/// one-pass evaluation agree on the value channel (catamorphism fusion on the real engine).
#[test]
fn test_encapsulation_flat() {
    // Flat: the 4-node chain 0(+1) -> 1(*2) -> 2(+1) -> 3(*2) in one pass.
    let flat = chain_graph(&[0, 1, 2, 3]);
    let seed = PropagatingEffect::from_value(3.0f64);
    let one_pass = flat.evaluate_subgraph_from_cause(0, &seed);
    assert!(one_pass.is_ok());
    // 3 -> +1 = 4 -> *2 = 8 -> +1 = 9 -> *2 = 18.
    assert_eq!(one_pass.value(), Some(&18.0));

    // Nested: the same chain split into two encapsulated stages, the first stage's result
    // seeding the second — the "causaloid-wrapped subgraph" evaluation.
    let prefix = chain_graph(&[0, 1]);
    let suffix = chain_graph(&[2, 3]);
    let mid = prefix.evaluate_subgraph_from_cause(0, &seed);
    assert!(mid.is_ok());
    let two_stage = suffix.evaluate_subgraph_from_cause(
        0,
        &PropagatingEffect::from_value(*mid.value().expect("mid value")),
    );
    assert!(two_stage.is_ok());
    assert_eq!(two_stage.value(), one_pass.value());
}

/// THEOREM_MAP: core.causaloid.arrow_fragment
///
/// Lean: `arrow_fragment`, `interp_respects_category_laws` (`Core/Catamorphism.lean`). The
/// atom/compose fragment ≅ `ArrowTerm`: the real chain-graph evaluation equals interpreting the
/// reified chain term over the same element semantics — including a term from the ⊕-enlarged
/// set (`choice`/`fanin`) — and the interpretation factors through `T/≈` (terms related by
/// associativity interpret equally).
#[test]
fn test_arrow_fragment() {
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Op {
        AddOne,
        Double,
    }
    fn interp(op: &Op, x: i64) -> i64 {
        match op {
            Op::AddOne => x + 1,
            Op::Double => x * 2,
        }
    }

    // Causaloid side: the chain 0(+1) -> 1(*2) -> 2(+1) -> 3(*2) on the real engine.
    let g = chain_graph(&[0, 1, 2, 3]);
    let evaluated = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(3.0f64));
    assert_eq!(evaluated.value(), Some(&18.0));

    // Term side: the same chain reified and interpreted — atom for atom.
    let term = ArrowTerm::<i64, i64, Op>::generator(Op::AddOne)
        .compose(ArrowTerm::<i64, i64, Op>::generator(Op::Double))
        .compose(ArrowTerm::<i64, i64, Op>::generator(Op::AddOne))
        .compose(ArrowTerm::<i64, i64, Op>::generator(Op::Double));
    let interpreted = term.core().interpret(&interp, ArrowVal::Leaf(3));
    assert_eq!(interpreted, ArrowVal::Leaf(18));

    // The quotient T/≈: two associations of the same chain interpret equally (the interpreter
    // factors through the term language modulo the category laws — assumption #8).
    let left_assoc: ArrowCore<Op> = ArrowCore::Compose(
        Box::new(ArrowCore::Compose(
            Box::new(ArrowCore::Gen(Op::AddOne)),
            Box::new(ArrowCore::Gen(Op::Double)),
        )),
        Box::new(ArrowCore::Gen(Op::AddOne)),
    );
    let right_assoc: ArrowCore<Op> = ArrowCore::Compose(
        Box::new(ArrowCore::Gen(Op::AddOne)),
        Box::new(ArrowCore::Compose(
            Box::new(ArrowCore::Gen(Op::Double)),
            Box::new(ArrowCore::Gen(Op::AddOne)),
        )),
    );
    for x in [0i64, 3, -7] {
        assert_eq!(
            left_assoc.interpret(&interp, ArrowVal::Leaf(x)),
            right_assoc.interpret(&interp, ArrowVal::Leaf(x))
        );
    }

    // The ⊕-enlarged fragment: a choice/fanin term agrees with composing the taken branch —
    // the correspondence covers the Stage-2b generators (haft.arrow_term.choice_interpret_sound).
    let branchy: ArrowCore<Op> = ArrowCore::Compose(
        Box::new(ArrowCore::Choice(
            Box::new(ArrowCore::Gen(Op::AddOne)),
            Box::new(ArrowCore::Gen(Op::Double)),
        )),
        Box::new(ArrowCore::Fanin(
            Box::new(ArrowCore::Gen(Op::Double)),
            Box::new(ArrowCore::Gen(Op::AddOne)),
        )),
    );
    // InL(3): AddOne then Double = (3+1)*2 = 8; InR(3): Double then AddOne = 3*2+1 = 7.
    assert_eq!(
        branchy.interpret(&interp, ArrowVal::inl(ArrowVal::Leaf(3))),
        ArrowVal::Leaf(8)
    );
    assert_eq!(
        branchy.interpret(&interp, ArrowVal::inr(ArrowVal::Leaf(3))),
        ArrowVal::Leaf(7)
    );
}
