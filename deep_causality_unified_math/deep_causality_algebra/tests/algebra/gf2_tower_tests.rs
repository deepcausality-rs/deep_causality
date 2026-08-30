/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Where 𝔽₂ sits in the tower.
//!
//! Each admission witness compiles only if `Gf2` reaches the bound, so calling it is the assertion.
//! The exclusions cannot be written as failing calls — this MSRV has no negative impls and the
//! repository has no `trybuild` harness — so they are stated as the compile-fail doctest on
//! `DivisibleByIntegers` and as the law checks below, which are exhaustive because the field has two
//! elements.

use deep_causality_algebra::{
    AbelianGroup, AddMonoid, Additive, Annihilating, Associative, Commutative, CommutativeRing,
    CommutativeSemiring, Distributive, Field, IntegralDomain, InvMonoid, MulMonoid, Multiplicative,
    Ring, Semiring,
};
use deep_causality_num::{Gf2, One, Zero};

fn admits_abelian_group<T: AbelianGroup>() {}
fn admits_add_monoid<T: AddMonoid>() {}
fn admits_mul_monoid<T: MulMonoid>() {}
fn admits_semiring<T: Semiring>() {}
fn admits_commutative_semiring<T: CommutativeSemiring>() {}
fn admits_ring<T: Ring>() {}
fn admits_commutative_ring<T: CommutativeRing>() {}
fn admits_integral_domain<T: IntegralDomain>() {}
fn admits_field<T: Field>() {}
fn admits_inv_monoid<T: InvMonoid>() {}
fn admits_associative_add<T: Associative<Additive>>() {}
fn admits_associative_mul<T: Associative<Multiplicative>>() {}
fn admits_commutative_add<T: Commutative<Additive>>() {}
fn admits_commutative_mul<T: Commutative<Multiplicative>>() {}
fn admits_distributive<T: Distributive>() {}
fn admits_annihilating<T: Annihilating>() {}

const O: Gf2 = Gf2::ZERO;
const I: Gf2 = Gf2::ONE;
const BOTH: [Gf2; 2] = [O, I];

#[test]
fn test_gf2_carries_every_law_marker() {
    admits_associative_add::<Gf2>();
    admits_associative_mul::<Gf2>();
    admits_commutative_add::<Gf2>();
    admits_commutative_mul::<Gf2>();
    admits_distributive::<Gf2>();
    admits_annihilating::<Gf2>();
}

#[test]
fn test_gf2_climbs_the_additive_and_multiplicative_chains() {
    admits_add_monoid::<Gf2>();
    admits_mul_monoid::<Gf2>();
    admits_abelian_group::<Gf2>();
}

#[test]
fn test_gf2_reaches_ring_and_commutative_ring() {
    admits_semiring::<Gf2>();
    admits_commutative_semiring::<Gf2>();
    admits_ring::<Gf2>();
    admits_commutative_ring::<Gf2>();
}

#[test]
fn test_gf2_reaches_integral_domain() {
    admits_integral_domain::<Gf2>();
}

#[test]
fn test_gf2_reaches_field_through_the_blanket() {
    // `Field` is blanket-implemented over `CommutativeRing + InvMonoid + Div + DivAssign`, so this
    // is not written by hand anywhere. That is exactly why `DivisibleByIntegers` is needed: nothing
    // opted 𝔽₂ in, and nothing would have opted it out.
    admits_inv_monoid::<Gf2>();
    admits_field::<Gf2>();
}

// The laws the markers promise, checked exhaustively. Over a two-element field "exhaustive" is
// eight triples, so these are proofs rather than samples.

#[test]
fn test_additive_associativity_holds_exhaustively() {
    for a in BOTH {
        for b in BOTH {
            for c in BOTH {
                assert_eq!((a + b) + c, a + (b + c));
            }
        }
    }
}

#[test]
fn test_multiplicative_associativity_holds_exhaustively() {
    for a in BOTH {
        for b in BOTH {
            for c in BOTH {
                assert_eq!((a * b) * c, a * (b * c));
            }
        }
    }
}

#[test]
fn test_commutativity_holds_exhaustively() {
    for a in BOTH {
        for b in BOTH {
            assert_eq!(a + b, b + a);
            assert_eq!(a * b, b * a);
        }
    }
}

#[test]
fn test_distributivity_holds_exhaustively() {
    for a in BOTH {
        for b in BOTH {
            for c in BOTH {
                assert_eq!(a * (b + c), a * b + a * c);
                assert_eq!((b + c) * a, b * a + c * a);
            }
        }
    }
}

#[test]
fn test_annihilation_holds_exhaustively() {
    for a in BOTH {
        assert_eq!(Gf2::zero() * a, Gf2::zero());
        assert_eq!(a * Gf2::zero(), Gf2::zero());
    }
}

#[test]
fn test_no_zero_divisors_holds_exhaustively() {
    // The integral-domain axiom: a*b = 0 implies a = 0 or b = 0.
    for a in BOTH {
        for b in BOTH {
            if (a * b).is_zero() {
                assert!(a.is_zero() || b.is_zero());
            }
        }
    }
}

#[test]
fn test_non_triviality() {
    // The other integral-domain axiom: 1 != 0.
    assert_ne!(Gf2::one(), Gf2::zero());
}

#[test]
fn test_field_division_really_inverts() {
    // What `Invertible` promises: a * (1 / a) == 1 for every non-zero a. F2 has one such a.
    assert_eq!(I * (Gf2::one() / I), Gf2::one());
}

#[test]
fn test_inv_monoid_inverse_agrees_with_division() {
    assert_eq!(InvMonoid::inverse(&I), Gf2::one() / I);
}
