/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Promonad.lean` (lax-monoidal merge; see
//! deviation D3 — the trait is not the categorical promonad).

use deep_causality_haft::{HKT3Unbound, NoConstraint, Promonad, Satisfies};

// Diagonal triple carrier, mirroring the crate's canonical promonad test.
#[derive(Debug, PartialEq, Clone)]
struct Triple<A, B, C>(A, B, C);
struct TripleWitness;
impl HKT3Unbound for TripleWitness {
    type Constraint = NoConstraint;
    type Type<A, B, C> = Triple<A, B, C>;
}

impl Promonad<TripleWitness> for TripleWitness {
    fn merge<A, B, C, F>(pa: Triple<A, A, A>, pb: Triple<B, B, B>, mut f: F) -> Triple<C, C, C>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        F: FnMut(A, B) -> C,
    {
        Triple(f(pa.0, pb.0), f(pa.1, pb.1), f(pa.2, pb.2))
    }

    fn fuse<A, B, C>(_a: A, _b: B) -> Triple<A, B, C>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
    {
        // `fuse`'s free `C` is structurally undetermined (deviation D3) — no lawful
        // implementation exists for this carrier.
        unimplemented!("fuse cannot be implemented for a value-carrying C")
    }
}

/// THEOREM_MAP: haft.promonad.merge_naturality
#[test]
fn test_promonad_merge_naturality() {
    // Binaturality: merge (map p a) (map q b) h = merge a b (|x, y| h (p x) (q y))
    let map3 = |t: Triple<i32, i32, i32>, p: fn(i32) -> i32| Triple(p(t.0), p(t.1), p(t.2));
    let a = Triple(1, 2, 3);
    let b = Triple(10, 20, 30);
    let p = |x: i32| x * 2;
    let q = |y: i32| y + 1;
    let h = |x: i32, y: i32| x + y;

    assert_eq!(
        TripleWitness::merge(map3(a.clone(), p), map3(b.clone(), q), h),
        TripleWitness::merge(a, b, |x, y| h(p(x), q(y)))
    );
}
