/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Algebra/Verdict.lean` (bounded lattice + complement).

use deep_causality_algebra::Verdict;

/// THEOREM_MAP: algebra.verdict.lattice_laws
#[test]
fn test_verdict_lattice_laws() {
    // Bounded-lattice identities on the boolean verdict carrier.
    // meet with top and join with bottom act as identities.
    assert!(true.meet(<bool as Verdict>::top()));
    assert!(!false.join(<bool as Verdict>::bottom()));
    // meet / join commutativity
    assert_eq!(true.meet(false), false.meet(true));
    assert_eq!(true.join(false), false.join(true));
    // absorption: x ⊓ (x ⊔ y) = x and x ⊔ (x ⊓ y) = x
    assert!(true.meet(true.join(false)));
    assert!(true.join(true.meet(false)));
}

/// THEOREM_MAP: algebra.verdict.complement
#[test]
fn test_verdict_complement() {
    // Involution: complement (complement x) = x
    assert!(Verdict::complement(Verdict::complement(true)));
    assert!(!Verdict::complement(Verdict::complement(false)));
    // De Morgan: complement (x ⊓ y) = complement x ⊔ complement y
    assert_eq!(
        true.meet(false).complement(),
        true.complement().join(false.complement())
    );
}
