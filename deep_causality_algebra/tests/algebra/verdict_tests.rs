/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Law-tests for the `Verdict` bounded-lattice/complement carrier. Mirrors
//! `DeepCausalityFormal/Algebra/Verdict.lean`.

use deep_causality_algebra::{Prob, Verdict};

#[test]
fn test_bool_boolean_algebra() {
    // lattice identities
    assert!(true.meet(<bool as Verdict>::top()));
    assert!(!false.join(<bool as Verdict>::bottom()));
    // commutativity + associativity
    assert_eq!(true.meet(false), false.meet(true));
    assert_eq!(true.join(false), false.join(true));
    // absorption
    assert!(true.meet(true.join(false)));
    assert!(true.join(true.meet(false)));
    // complement involution + De Morgan
    assert!(Verdict::complement(Verdict::complement(true)));
    assert_eq!(
        true.meet(false).complement(),
        true.complement().join(false.complement())
    );
    // None = complement of Any (join-fold)
    let any = [false, false]
        .iter()
        .copied()
        .fold(<bool as Verdict>::bottom(), Verdict::join);
    assert!(any.complement()); // no child fires -> None holds
}

/// Involution holds up to floating-point rounding, so `Prob` values are compared with a tolerance.
#[test]
fn test_prob_mv_algebra() {
    fn approx(a: Prob, b: Prob) {
        assert!((a.0 - b.0).abs() < 1e-12, "{a:?} !~ {b:?}");
    }
    approx(Prob(0.3).complement(), Prob(0.7));
    approx(
        Verdict::complement(Verdict::complement(Prob(0.3))),
        Prob(0.3),
    ); // involution up to rounding
    assert_eq!(Prob(0.3).meet(Prob(0.8)), Prob(0.3)); // min (exact)
    assert_eq!(Prob(0.3).join(Prob(0.8)), Prob(0.8)); // max (exact)
    assert_eq!(<Prob as Verdict>::bottom(), Prob(0.0));
    assert_eq!(<Prob as Verdict>::top(), Prob(1.0));
}
