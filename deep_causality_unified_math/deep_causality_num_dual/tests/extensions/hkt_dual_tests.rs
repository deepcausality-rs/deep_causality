/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Functor, Foldable and lax monoidal laws for `DualWitness`.
//!
//! `Dual<A>` is `A^S` over the two-element index set, so componentwise pairing is the only lawful
//! `φ`. The mutation tests at the foot show these laws can tell it from the plausible alternative,
//! rather than passing on anything two-slot-shaped.
//!
//! Payloads are small integers so every law is checked by exact equality with no float slop.

use deep_causality_haft::{Foldable, Functor, LaxMonoidal, MonoidalApplicative, Semigroupal};
use deep_causality_num_dual::{Dual, DualWitness};

const ITERS: usize = 64;
const SEED: u64 = 0xD0A1_5EED;

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x9E37_79B9_7F4A_7C15)
    }
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 11
    }
    fn small(&mut self) -> i64 {
        (self.next() % 4001) as i64 - 2000
    }
    fn dual(&mut self) -> Dual<i64> {
        Dual {
            re: self.small(),
            du: self.small(),
        }
    }
}

// ---------------------------------------------------------------------------
// Functor
// ---------------------------------------------------------------------------

#[test]
fn functor_identity_and_composition() {
    let mut rng = Rng::new(SEED);
    for i in 0..ITERS {
        let a = rng.dual();
        assert_eq!(DualWitness::fmap(a, |x| x), a, "identity, iter {i}");

        let f = |x: i64| x * 3 - 1;
        let g = |x: i64| x + 7;
        assert_eq!(
            DualWitness::fmap(a, |x| g(f(x))),
            DualWitness::fmap(DualWitness::fmap(a, f), g),
            "composition, iter {i}"
        );
    }
}

#[test]
fn fmap_reaches_an_unrelated_payload_type() {
    let d = Dual {
        re: 3.0_f64,
        du: 1.0,
    };
    let labelled = DualWitness::fmap(d, |x| format!("<{x}>"));
    assert_eq!(labelled.re, "<3>");
    assert_eq!(labelled.du, "<1>");

    // A payload with no arithmetic at all, which the struct bound used to forbid.
    let v: Dual<Vec<u8>> = Dual {
        re: vec![1, 2],
        du: vec![3],
    };
    assert_eq!(DualWitness::fold(v, 0usize, |acc, xs| acc + xs.len()), 3);
}

#[test]
fn fold_visits_re_then_du() {
    let d = Dual { re: "re", du: "du" };
    let order = DualWitness::fold(d, String::new(), |mut acc, s| {
        acc.push_str(s);
        acc
    });
    assert_eq!(order, "redu");
}

// ---------------------------------------------------------------------------
// Lax monoidal laws
// ---------------------------------------------------------------------------

#[test]
fn zip_is_natural_in_both_arguments() {
    let mut rng = Rng::new(SEED ^ 0x1);
    let f = |a: i64| a * 3 - 1;
    let g = |b: i64| b + 7;
    for i in 0..ITERS {
        let (a, b) = (rng.dual(), rng.dual());
        assert_eq!(
            DualWitness::zip(DualWitness::fmap(a, f), DualWitness::fmap(b, g)),
            DualWitness::fmap(DualWitness::zip(a, b), |(x, y)| (f(x), g(y))),
            "naturality, seed {SEED:#x} iter {i}"
        );
    }
}

#[test]
fn zip_associates_modulo_the_associator() {
    let mut rng = Rng::new(SEED ^ 0x2);
    for i in 0..ITERS {
        let (a, b, c) = (rng.dual(), rng.dual(), rng.dual());
        let lhs = DualWitness::fmap(
            DualWitness::zip(DualWitness::zip(a, b), c),
            |((x, y), z)| (x, (y, z)),
        );
        assert_eq!(
            lhs,
            DualWitness::zip(a, DualWitness::zip(b, c)),
            "assoc, seed {SEED:#x} iter {i}"
        );
    }
}

#[test]
fn unit_is_a_two_sided_identity_for_zip() {
    let mut rng = Rng::new(SEED ^ 0x3);
    for i in 0..ITERS {
        let a = rng.dual();
        let left = DualWitness::fmap(DualWitness::zip(DualWitness::unit(), a), |((), x)| x);
        let right = DualWitness::fmap(DualWitness::zip(a, DualWitness::unit()), |(x, ())| x);
        assert_eq!(left, a, "left unit, seed {SEED:#x} iter {i}");
        assert_eq!(right, a, "right unit, seed {SEED:#x} iter {i}");
    }
}

#[test]
fn unit_is_the_sole_inhabitant_of_dual_unit() {
    assert_eq!(DualWitness::unit(), Dual { re: (), du: () });
}

#[test]
fn zip_is_the_derived_form_of_zip_with() {
    let mut rng = Rng::new(SEED ^ 0x4);
    for i in 0..ITERS {
        let (a, b) = (rng.dual(), rng.dual());
        assert_eq!(
            DualWitness::zip(a, b),
            DualWitness::zip_with(a, b, |x, y| (x, y)),
            "zip/zip_with, seed {SEED:#x} iter {i}"
        );
    }
}

#[test]
fn apply_pairs_each_function_with_its_own_channel() {
    let ff = Dual {
        re: (|x: i64| x + 10) as fn(i64) -> i64,
        du: (|x: i64| x * 100) as fn(i64) -> i64,
    };
    assert_eq!(
        DualWitness::apply(ff, Dual { re: 1, du: 2 }),
        Dual { re: 11, du: 200 }
    );
}

/// `apply` is Δ-free: it runs over a payload that is neither `Clone` nor `Copy`.
#[test]
fn zip_with_accepts_a_move_only_payload() {
    #[derive(PartialEq, Debug)]
    struct Moved(u32);

    let out = DualWitness::zip_with(
        Dual {
            re: Moved(1),
            du: Moved(2),
        },
        Dual {
            re: Moved(10),
            du: Moved(20),
        },
        |a, b| Moved(a.0 + b.0),
    );
    assert_eq!(out.re, Moved(11));
    assert_eq!(out.du, Moved(22));
}

// ---------------------------------------------------------------------------
// Mutation test
// ---------------------------------------------------------------------------

/// The swap variant is the plausible alternative pairing. It fails the left unit law, which is the
/// Yoneda argument made concrete: `u` and `v` are forced to the identity.
#[test]
fn the_swap_variant_fails_the_left_unit_law() {
    fn swapped<A, B>(fa: Dual<A>, fb: Dual<B>) -> Dual<(A, B)> {
        Dual {
            re: (fa.re, fb.du),
            du: (fa.du, fb.re),
        }
    }

    let a = Dual { re: 1i64, du: 2i64 };

    let under_swap = DualWitness::fmap(swapped(DualWitness::unit(), a), |((), x)| x);
    assert_ne!(
        under_swap, a,
        "the swap variant must fail the left unit law; if it passes, this test is not \
         discriminating and the uniqueness claim in the docs is untested"
    );
    assert_eq!(under_swap, Dual { re: 2, du: 1 });

    // The lawful pairing passes on the same input.
    let lawful = DualWitness::fmap(DualWitness::zip(DualWitness::unit(), a), |((), x)| x);
    assert_eq!(lawful, a);
}

// ---------------------------------------------------------------------------
// The struct bound removal does not disturb differentiation
// ---------------------------------------------------------------------------

/// Nesting rests on `impl<T: Real + Div<Output = T>> Real for Dual<T>`, which keeps its own bound,
/// not on the struct bound that this change removed.
#[test]
fn nested_duals_still_give_higher_derivatives() {
    // f(x) = x³ + 2x  =>  f(3) = 33, f'(3) = 29, f''(3) = 18
    let x: Dual<Dual<f64>> = Dual::variable(Dual::variable(3.0_f64));
    let y = x * x * x + x + x;
    assert_eq!(y.value().value(), 33.0);
    assert_eq!(y.value().derivative(), 29.0);
    assert_eq!(y.derivative().derivative(), 18.0);

    // g(x) = x⁴  =>  g(2) = 16, g'(2) = 32, g''(2) = 48
    let z: Dual<Dual<Dual<f64>>> = Dual::variable(Dual::variable(Dual::variable(2.0_f64)));
    let w = z * z * z * z;
    assert_eq!(w.value().value().value(), 16.0);
    assert_eq!(w.value().value().derivative(), 32.0);
    assert_eq!(w.value().derivative().derivative(), 48.0);
}

/// `fmap` is the pair functor, not differentiation. Pinned so the docstring's caveat has a test.
#[test]
fn fmap_carries_no_chain_rule() {
    // Differentiating x² at x = 3 gives 6 through the arithmetic.
    let x = Dual::variable(3.0_f64);
    let squared = x * x;
    assert_eq!(squared.derivative(), 6.0);

    // fmap with the same function does NOT: it squares both channels independently.
    let mapped = DualWitness::fmap(x, |v| v * v);
    assert_eq!(mapped.re, 9.0);
    assert_eq!(mapped.du, 1.0); // 1² = 1, the seed squared — not the derivative 6
    assert_ne!(mapped.du, squared.derivative());
}
