/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{HKT3Unbound, MonoidalMerge, NoConstraint, Satisfies};

// Simplified Triple for testing
#[derive(Debug, PartialEq, Clone)]
struct Triple<A, B, C>(A, B, C);

struct TripleWitness;
impl HKT3Unbound for TripleWitness {
    type Constraint = NoConstraint;
    type Type<A, B, C> = Triple<A, B, C>;
}

impl MonoidalMerge<TripleWitness> for TripleWitness {
    fn merge<A, B, C, F>(pa: Triple<A, A, A>, pb: Triple<B, B, B>, mut f: F) -> Triple<C, C, C>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        F: FnMut(A, B) -> C,
    {
        let c1 = f(pa.0, pb.0);
        let c2 = f(pa.1, pb.1);
        let c3 = f(pa.2, pb.2);
        Triple(c1, c2, c3)
    }
}

#[test]
fn test_monoidal_merge() {
    // Test merge
    let triple_a = Triple(10, 10, 10);
    let triple_b = Triple(5, 5, 5);

    let merged = TripleWitness::merge(triple_a, triple_b, |a, b| a + b);
    assert_eq!(merged, Triple(15, 15, 15));
}
