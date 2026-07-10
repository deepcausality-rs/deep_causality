/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for `lean/DeepCausalityFormal/Core/VerdictClosure.lean` — the Verdict closure
//! of collection aggregation (Stage 3): `All`/`Any`/`None`/`Some(k)` are closed operations in the
//! Verdict algebra, so `Coll : Causaloid → Causaloid` (assumption #5). Lean proves ∀; these tests
//! pin the real `evaluate_collection` path — now bounded by `Aggregatable: Verdict` — to the same
//! statements at representative inputs.

use deep_causality::utils_test::test_utils;
use deep_causality::{
    AggregateLogic, BaseCausaloid, Causaloid, CausaloidType, MonadicCausableCollection,
    PropagatingEffect, Verdict,
};
use std::sync::Arc;

fn bool_bag(pattern: &[bool]) -> Vec<BaseCausaloid<bool, bool>> {
    pattern
        .iter()
        .map(|&b| {
            if b {
                test_utils::get_test_causaloid_deterministic_true()
            } else {
                test_utils::get_test_causaloid_deterministic_false()
            }
        })
        .collect()
}

/// THEOREM_MAP: core.verdict.closure
///
/// Lean: `closure_fold_step`, `none_is_any_complement`, `someK_decides`, `coll_closure`
/// (`Core/VerdictClosure.lean`). Every aggregation mode, run through the real
/// `evaluate_collection`, lands back in the verdict carrier and matches the lattice folds; `None`
/// is `Any` post-composed with the carrier's `complement`; `Some(k)` decides by count — and the
/// aggregated bag is again a causaloid (the `Coll` summand of the fixpoint).
#[test]
fn test_verdict_closure_aggregation_modes() {
    let effect = PropagatingEffect::from_value(true);
    let patterns: [&[bool]; 4] = [
        &[true, true, true],
        &[true, false, true],
        &[false, false],
        &[true],
    ];

    for pattern in patterns {
        let bag = bool_bag(pattern);

        // All = meet-fold, Any = join-fold — closed Boolean operations.
        let all = bag
            .evaluate_collection(&effect, &AggregateLogic::All, Some(0.5))
            .value_cloned()
            .expect("All lands in the carrier");
        let any = bag
            .evaluate_collection(&effect, &AggregateLogic::Any, Some(0.5))
            .value_cloned()
            .expect("Any lands in the carrier");
        let expected_all = pattern.iter().copied().fold(bool::top(), bool::meet);
        let expected_any = pattern.iter().copied().fold(bool::bottom(), bool::join);
        assert_eq!(all, expected_all);
        assert_eq!(any, expected_any);

        // None = Any ∘ complement (the spec's None characterization, on the real path).
        let none = bag
            .evaluate_collection(&effect, &AggregateLogic::None, Some(0.5))
            .value_cloned()
            .expect("None lands in the carrier");
        assert_eq!(none, any.complement());

        // Some(k) decides by the firing count (the Count monoid + boundary comparison).
        let fired = pattern.iter().filter(|&&b| b).count();
        for k in 0..=pattern.len() {
            let some_k = bag
                .evaluate_collection(&effect, &AggregateLogic::Some(k), Some(0.5))
                .value_cloned()
                .expect("Some(k) lands in the carrier");
            assert_eq!(some_k, fired >= k, "Some({k}) over {pattern:?}");
        }

        // Coll : Causaloid → Causaloid — the aggregated bag rolls into a Collection causaloid.
        let coll: BaseCausaloid<bool, bool> = Causaloid::from_causal_collection(
            42,
            Arc::new(bag),
            "the closure witness",
            AggregateLogic::All,
            0.5,
        );
        assert_eq!(*coll.causal_type(), CausaloidType::Collection);
    }
}

/// THEOREM_MAP: core.verdict.carriers
///
/// Lean: `bool_carrier_characterization`, `bool_distributive` (`Core/VerdictClosure.lean`);
/// MV pinned here per the deviation note. The named carriers behind the one trait: `bool` is the
/// Boolean algebra (distributive, excluded middle holds), `f64`/`Prob` is the MV algebra on
/// `[0, 1]` (`meet = min`, `join = max`, `complement = 1 − p`; excluded middle fails), and the
/// uncertain carriers lift both pointwise. No general tensor/operator type implements `Verdict`
/// (the effect-algebra scope guard: meet/join are partial off the commuting fragment).
#[test]
fn test_verdict_carriers() {
    // bool: Boolean — distributivity and excluded middle.
    for x in [false, true] {
        for y in [false, true] {
            for z in [false, true] {
                assert_eq!(x.meet(y.join(z)), x.meet(y).join(x.meet(z)));
            }
        }
        assert_eq!(x.join(x.complement()), bool::top()); // excluded middle holds
        assert_eq!(x.complement().complement(), x); // involution
    }

    // f64: MV on [0,1] — min/max/1−p; involution holds, excluded middle FAILS (the class split).
    let ps = [0.0, 0.3, 0.5, 0.9, 1.0];
    for &p in &ps {
        for &q in &ps {
            assert_eq!(p.meet(q), p.min(q));
            assert_eq!(p.join(q), p.max(q));
            // Closure: every operation stays in [0,1].
            for v in [p.meet(q), p.join(q), p.complement()] {
                assert!((0.0..=1.0).contains(&v));
            }
        }
        assert!((p.complement().complement() - p).abs() < 1e-12); // involution
    }
    assert_ne!(0.3f64.join(0.3f64.complement()), f64::top()); // excluded middle fails (MV, not Boolean)

    // The uncertain carriers lift the same algebras pointwise (point masses sample
    // deterministically).
    use deep_causality_uncertain::{UncertainBool, UncertainF64};
    let t = UncertainBool::point(true);
    let f = UncertainBool::point(false);
    assert!(!t.meet(f).sample().expect("sample"));
    let t = UncertainBool::point(true);
    let f = UncertainBool::point(false);
    assert!(t.join(f).sample().expect("sample"));
    let f = UncertainBool::point(false);
    assert!(f.complement().sample().expect("sample"));
    assert!(!UncertainBool::bottom().sample().expect("sample"));
    assert!(UncertainBool::top().sample().expect("sample"));

    let a = UncertainF64::point(0.3);
    let b = UncertainF64::point(0.8);
    assert_eq!(a.meet(b).sample().expect("sample"), 0.3);
    let a = UncertainF64::point(0.3);
    let b = UncertainF64::point(0.8);
    assert_eq!(a.join(b).sample().expect("sample"), 0.8);
    let a = UncertainF64::point(0.3);
    assert!((a.complement().sample().expect("sample") - 0.7).abs() < 1e-12);
    assert_eq!(UncertainF64::bottom().sample().expect("sample"), 0.0);
    assert_eq!(UncertainF64::top().sample().expect("sample"), 1.0);
}

/// THEOREM_MAP: core.verdict.perm_invariance
///
/// Lean: `aggregate_perm`, `coll_perm` (`Core/VerdictClosure.lean`). The #1 scoped order-invariance
/// theorem on the collection path: for every `AggregateLogic` mode, permuting the member bag leaves
/// the aggregate **value** unchanged — `All`/`Any` are the commutative-associative meet-/join-folds,
/// `None` inherits from `Any`, `Some(k)` from the permutation-invariant firing count. Scope (#1
/// ruling): the value channel, on the stateless all-success path. Here the real `evaluate_collection`
/// is run on a base bag and on permutations of the SAME multiset, asserting equal values per mode.
#[test]
fn test_verdict_perm_invariance() {
    let effect = PropagatingEffect::from_value(true);

    // Each row: a base ordering and permutations of the SAME multiset (same members, reordered).
    let cases: [(&[bool], &[&[bool]]); 3] = [
        (
            &[true, false, true],
            &[&[true, true, false], &[false, true, true]],
        ),
        (
            &[true, false, false, true],
            &[&[false, true, true, false], &[true, true, false, false]],
        ),
        (
            &[false, false, true],
            &[&[true, false, false], &[false, true, false]],
        ),
    ];

    let eval = |bag: &[bool], logic: &AggregateLogic| -> bool {
        bool_bag(bag)
            .evaluate_collection(&effect, logic, Some(0.5))
            .value_cloned()
            .expect("aggregate lands in the carrier")
    };

    for (base, perms) in cases {
        let n = base.len();
        for perm in perms {
            // Sanity: a genuine permutation — same multiset (fired count equal), possibly reordered.
            assert_eq!(
                base.iter().filter(|&&b| b).count(),
                perm.iter().filter(|&&b| b).count(),
                "test bug: {perm:?} is not a permutation of {base:?}"
            );

            // All / Any / None: value invariant under permutation.
            for logic in [
                AggregateLogic::All,
                AggregateLogic::Any,
                AggregateLogic::None,
            ] {
                assert_eq!(
                    eval(base, &logic),
                    eval(perm, &logic),
                    "{logic:?} not permutation-invariant: {base:?} vs {perm:?}"
                );
            }

            // Some(k) for every threshold: value invariant (the firing count is a bag invariant).
            for k in 0..=n {
                assert_eq!(
                    eval(base, &AggregateLogic::Some(k)),
                    eval(perm, &AggregateLogic::Some(k)),
                    "Some({k}) not permutation-invariant: {base:?} vs {perm:?}"
                );
            }
        }
    }
}
