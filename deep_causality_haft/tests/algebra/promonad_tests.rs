/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{HKT3Unbound, Promonad};

// Simplified Triple for testing
#[derive(Debug, PartialEq, Clone)]
struct Triple<A, B, C>(A, B, C);

struct TripleWitness;
impl HKT3Unbound for TripleWitness {
    type Type<A, B, C> = Triple<A, B, C>;
}

impl Promonad<TripleWitness> for TripleWitness {
    fn merge<A, B, C, F>(pa: Triple<A, A, A>, pb: Triple<B, B, B>, mut f: F) -> Triple<C, C, C>
    where
        F: FnMut(A, B) -> C,
    {
        // Simplified - just use first result
        let c1 = f(pa.0, pb.0);
        let c2 = f(pa.1, pb.1);
        let c3 = f(pa.2, pb.2);
        Triple(c1, c2, c3)
    }

    fn fuse<A, B, C>(_input_a: A, _input_b: B) -> Triple<A, B, C> {
        // Simplified - panic for C (not used in test)
        panic!("Not implemented for test")
    }
}

#[test]
fn test_promonad() {
    // Test merge
    let triple_a = Triple(10, 10, 10);
    let triple_b = Triple(5, 5, 5);

    let merged = TripleWitness::merge(triple_a, triple_b, |a, b| a + b);
    assert_eq!(merged, Triple(15, 15, 15));
}
