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

/// The raw `f64` MV-algebra carrier: `bottom = 0.0`, `top = 1.0`, `meet = min`, `join = max`,
/// `complement = 1 − p`. `bottom`/`top`/`meet`/`join` are exact for these operands; complement is
/// exact for values representable without rounding (`0.3` is not, so it is compared with tolerance).
#[test]
fn test_f64_mv_algebra() {
    // lattice bounds
    assert_eq!(<f64 as Verdict>::bottom(), 0.0);
    assert_eq!(<f64 as Verdict>::top(), 1.0);
    // meet = min, join = max (exact for these operands)
    assert_eq!(0.3_f64.meet(0.8), 0.3);
    assert_eq!(0.3_f64.join(0.8), 0.8);
    // bottom is the join identity, top is the meet identity
    assert_eq!(0.4_f64.join(<f64 as Verdict>::bottom()), 0.4);
    assert_eq!(0.4_f64.meet(<f64 as Verdict>::top()), 0.4);
    // complement = 1 − p
    assert!((0.3_f64.complement() - 0.7).abs() < 1e-12);
    // complement involution up to rounding
    assert!((Verdict::complement(Verdict::complement(0.3_f64)) - 0.3).abs() < 1e-12);
}
