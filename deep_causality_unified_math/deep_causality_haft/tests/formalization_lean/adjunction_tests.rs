/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Adjunction.lean` (Mac Lane, CWM §IV.1).

use deep_causality_haft::{Adjunction, HKT, NoConstraint, Satisfies};

// The crate's canonical adjunction instance (Identity ⊣ Identity), mirroring
// `tests/algebra/adjunction_tests.rs`. The Lean file proves the non-trivial currying
// adjunction (- × S) ⊣ (S → -), which no `fn`-pointer carrier can express (unit must
// capture `a`).
#[derive(Debug, PartialEq, Clone)]
struct Idn<T>(T);
struct IdnWitness;
impl HKT for IdnWitness {
    type Constraint = NoConstraint;
    type Type<T> = Idn<T>;
}

struct IdnAdjunction;
impl Adjunction<IdnWitness, IdnWitness, ()> for IdnAdjunction {
    // Extraction from an identity functor cannot fail: the carrier holds exactly one value, so
    // there is no empty case. `Infallible` states that in the type rather than in a comment.
    type Error = core::convert::Infallible;

    fn unit<A>(_ctx: &(), a: A) -> Idn<Idn<A>>
    where
        A: Satisfies<NoConstraint> + Clone,
        Idn<A>: Satisfies<NoConstraint>,
    {
        Idn(Idn(a))
    }

    fn counit<B>(_ctx: &(), lrb: Idn<Idn<B>>) -> Result<B, Self::Error>
    where
        B: Satisfies<NoConstraint> + Clone,
        Idn<B>: Satisfies<NoConstraint>,
    {
        Ok(lrb.0.0)
    }

    fn left_adjunct<A, B, Func>(_ctx: &(), a: A, f: Func) -> Idn<B>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        Idn<A>: Satisfies<NoConstraint>,
        Func: Fn(Idn<A>) -> B,
    {
        Idn(f(Idn(a)))
    }

    fn right_adjunct<A, B, Func>(_ctx: &(), la: Idn<A>, mut f: Func) -> Result<B, Self::Error>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint> + Clone,
        Idn<B>: Satisfies<NoConstraint>,
        Func: FnMut(A) -> Idn<B>,
    {
        Ok(f(la.0).0)
    }
}

/// THEOREM_MAP: haft.adjunction.triangles
#[test]
fn test_adjunction_triangles() {
    // Triangle on R: R(ε) ∘ η_R = id — R acts by post-composition, so the composite is
    // Idn(counit(unit(rb).0)) for the Identity adjunction.
    // `counit` is partial in general, so the composite carries a `Result`. For this adjunction the
    // error type is `Infallible`, which is the type-level statement that the `Ok` arm is the only
    // one: `unwrap` here cannot fire.
    let rb = Idn(5);
    assert_eq!(
        Idn(IdnAdjunction::counit(&(), IdnAdjunction::unit(&(), rb.clone()).0).unwrap()),
        rb
    );
    // Triangle on L: ε_L ∘ L(η) = id — for Identity the same composite, read on L.
    let la = Idn(7);
    assert_eq!(
        Idn(IdnAdjunction::counit(&(), IdnAdjunction::unit(&(), la.0)).unwrap()),
        la
    );
}

/// THEOREM_MAP: haft.adjunction.adjunct_inverse
#[test]
fn test_adjunction_adjunct_inverse() {
    // The Hom-bijection: right_adjunct (left_adjunct f) = f and conversely.
    let f = |la: Idn<i32>| la.0 * 3;
    for a in [2, -8] {
        let round =
            IdnAdjunction::right_adjunct(&(), Idn(a), |x| IdnAdjunction::left_adjunct(&(), x, f));
        // `right_adjunct` is partial in general; here it is total, so the law reads `= Ok(f la)`.
        assert_eq!(round, Ok(f(Idn(a))));
    }
    let g = |a: i32| Idn(a + 9);
    for a in [2, -8] {
        let round = IdnAdjunction::left_adjunct(&(), a, |la: Idn<i32>| {
            IdnAdjunction::right_adjunct(&(), la, g).unwrap()
        });
        assert_eq!(round, g(a));
    }
}
