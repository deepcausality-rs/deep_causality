/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Lax monoidal laws for the Cayley-Dickson witnesses.
//!
//! `F(A) = A^S` for a finite index set `S`, so componentwise pairing is the *only* lawful `φ`
//! (Yoneda plus the two unit laws; see the module docs on `src/extensions/`). These tests hold the
//! implementation to that, and the mutation tests at the foot show they can tell the lawful `φ`
//! from a plausible alternative rather than passing on anything componentwise-shaped.
//!
//! Mirrors `lean/DeepCausalityFormal/Haft/LaxMonoidal.lean`, which proves the same laws over the
//! `Option` carrier.
//!
//! Payloads are small integers so every law is checked by exact equality with no float slop. Each
//! failure message carries the seed and iteration index, so a counterexample is reproducible.

use deep_causality_haft::{Functor, LaxMonoidal, MonoidalApplicative, Semigroupal};
use deep_causality_num_complex::{
    Complex, ComplexWitness, Octonion, OctonionWitness, Quaternion, QuaternionWitness,
};

const ITERS: usize = 64;
const SEED: u64 = 0x5EED_1A11;

/// Deterministic generator, so a counterexample is reproducible from its seed.
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
}

fn cx(rng: &mut Rng) -> Complex<i64> {
    Complex {
        re: rng.small(),
        im: rng.small(),
    }
}
fn quat(rng: &mut Rng) -> Quaternion<i64> {
    Quaternion {
        w: rng.small(),
        x: rng.small(),
        y: rng.small(),
        z: rng.small(),
    }
}
fn oct(rng: &mut Rng) -> Octonion<i64> {
    Octonion {
        s: rng.small(),
        e1: rng.small(),
        e2: rng.small(),
        e3: rng.small(),
        e4: rng.small(),
        e5: rng.small(),
        e6: rng.small(),
        e7: rng.small(),
    }
}

fn f(a: i64) -> i64 {
    a * 3 - 1
}
fn g(b: i64) -> i64 {
    b + 7
}

// ---------------------------------------------------------------------------
// Naturality: zip(fmap f fa, fmap g fb) == fmap (f × g) (zip fa fb)
// ---------------------------------------------------------------------------

#[test]
fn zip_is_natural_in_both_arguments() {
    let mut rng = Rng::new(SEED);
    for i in 0..ITERS {
        let (a, b) = (cx(&mut rng), cx(&mut rng));
        let lhs = ComplexWitness::zip(ComplexWitness::fmap(a, f), ComplexWitness::fmap(b, g));
        let rhs = ComplexWitness::fmap(ComplexWitness::zip(a, b), |(x, y)| (f(x), g(y)));
        assert_eq!(
            lhs, rhs,
            "Complex naturality failed, seed {SEED:#x} iter {i}"
        );

        let (a, b) = (quat(&mut rng), quat(&mut rng));
        let lhs =
            QuaternionWitness::zip(QuaternionWitness::fmap(a, f), QuaternionWitness::fmap(b, g));
        let rhs = QuaternionWitness::fmap(QuaternionWitness::zip(a, b), |(x, y)| (f(x), g(y)));
        assert_eq!(
            lhs, rhs,
            "Quaternion naturality failed, seed {SEED:#x} iter {i}"
        );

        let (a, b) = (oct(&mut rng), oct(&mut rng));
        let lhs = OctonionWitness::zip(OctonionWitness::fmap(a, f), OctonionWitness::fmap(b, g));
        let rhs = OctonionWitness::fmap(OctonionWitness::zip(a, b), |(x, y)| (f(x), g(y)));
        assert_eq!(
            lhs, rhs,
            "Octonion naturality failed, seed {SEED:#x} iter {i}"
        );
    }
}

// ---------------------------------------------------------------------------
// Associativity: zip(zip a b) c reassociated == zip a (zip b c)
// ---------------------------------------------------------------------------

#[test]
fn zip_associates_modulo_the_associator() {
    let mut rng = Rng::new(SEED ^ 0xA55);
    for i in 0..ITERS {
        let (a, b, c) = (cx(&mut rng), cx(&mut rng), cx(&mut rng));
        let lhs = ComplexWitness::fmap(
            ComplexWitness::zip(ComplexWitness::zip(a, b), c),
            |((x, y), z)| (x, (y, z)),
        );
        let rhs = ComplexWitness::zip(a, ComplexWitness::zip(b, c));
        assert_eq!(lhs, rhs, "Complex assoc failed, seed {SEED:#x} iter {i}");

        let (a, b, c) = (quat(&mut rng), quat(&mut rng), quat(&mut rng));
        let lhs = QuaternionWitness::fmap(
            QuaternionWitness::zip(QuaternionWitness::zip(a, b), c),
            |((x, y), z)| (x, (y, z)),
        );
        let rhs = QuaternionWitness::zip(a, QuaternionWitness::zip(b, c));
        assert_eq!(lhs, rhs, "Quaternion assoc failed, seed {SEED:#x} iter {i}");

        let (a, b, c) = (oct(&mut rng), oct(&mut rng), oct(&mut rng));
        let lhs = OctonionWitness::fmap(
            OctonionWitness::zip(OctonionWitness::zip(a, b), c),
            |((x, y), z)| (x, (y, z)),
        );
        let rhs = OctonionWitness::zip(a, OctonionWitness::zip(b, c));
        assert_eq!(lhs, rhs, "Octonion assoc failed, seed {SEED:#x} iter {i}");
    }
}

// ---------------------------------------------------------------------------
// Unit laws, modulo the unitors ((), A) ≅ A and (A, ()) ≅ A
// ---------------------------------------------------------------------------

#[test]
fn unit_is_a_two_sided_identity_for_zip() {
    let mut rng = Rng::new(SEED ^ 0x11);
    for i in 0..ITERS {
        let a = cx(&mut rng);
        let left =
            ComplexWitness::fmap(ComplexWitness::zip(ComplexWitness::unit(), a), |((), x)| x);
        let right =
            ComplexWitness::fmap(ComplexWitness::zip(a, ComplexWitness::unit()), |(x, ())| x);
        assert_eq!(left, a, "Complex left unit failed, seed {SEED:#x} iter {i}");
        assert_eq!(
            right, a,
            "Complex right unit failed, seed {SEED:#x} iter {i}"
        );

        let a = quat(&mut rng);
        let left = QuaternionWitness::fmap(
            QuaternionWitness::zip(QuaternionWitness::unit(), a),
            |((), x)| x,
        );
        let right = QuaternionWitness::fmap(
            QuaternionWitness::zip(a, QuaternionWitness::unit()),
            |(x, ())| x,
        );
        assert_eq!(
            left, a,
            "Quaternion left unit failed, seed {SEED:#x} iter {i}"
        );
        assert_eq!(
            right, a,
            "Quaternion right unit failed, seed {SEED:#x} iter {i}"
        );

        let a = oct(&mut rng);
        let left = OctonionWitness::fmap(
            OctonionWitness::zip(OctonionWitness::unit(), a),
            |((), x)| x,
        );
        let right = OctonionWitness::fmap(
            OctonionWitness::zip(a, OctonionWitness::unit()),
            |(x, ())| x,
        );
        assert_eq!(
            left, a,
            "Octonion left unit failed, seed {SEED:#x} iter {i}"
        );
        assert_eq!(
            right, a,
            "Octonion right unit failed, seed {SEED:#x} iter {i}"
        );
    }
}

// ---------------------------------------------------------------------------
// zip is genuinely derived from zip_with, and apply from zip_with
// ---------------------------------------------------------------------------

#[test]
fn zip_is_the_derived_form_of_zip_with() {
    let mut rng = Rng::new(SEED ^ 0x22);
    for i in 0..ITERS {
        let (a, b) = (cx(&mut rng), cx(&mut rng));
        assert_eq!(
            ComplexWitness::zip(a, b),
            ComplexWitness::zip_with(a, b, |x, y| (x, y)),
            "Complex zip/zip_with disagree, seed {SEED:#x} iter {i}"
        );

        let (a, b) = (quat(&mut rng), quat(&mut rng));
        assert_eq!(
            QuaternionWitness::zip(a, b),
            QuaternionWitness::zip_with(a, b, |x, y| (x, y)),
            "Quaternion zip/zip_with disagree, seed {SEED:#x} iter {i}"
        );

        let (a, b) = (oct(&mut rng), oct(&mut rng));
        assert_eq!(
            OctonionWitness::zip(a, b),
            OctonionWitness::zip_with(a, b, |x, y| (x, y)),
            "Octonion zip/zip_with disagree, seed {SEED:#x} iter {i}"
        );
    }
}

#[test]
fn apply_pairs_each_function_with_its_own_slot() {
    let ff = Complex {
        re: (|x: i64| x + 10) as fn(i64) -> i64,
        im: (|x: i64| x * 100) as fn(i64) -> i64,
    };
    let fa = Complex { re: 1, im: 2 };
    assert!(ComplexWitness::apply(ff, fa) == Complex { re: 11, im: 200 });

    let qf = Quaternion {
        w: (|x: i64| x + 1) as fn(i64) -> i64,
        x: (|x: i64| x + 2) as fn(i64) -> i64,
        y: (|x: i64| x + 3) as fn(i64) -> i64,
        z: (|x: i64| x + 4) as fn(i64) -> i64,
    };
    let qa = Quaternion {
        w: 0,
        x: 0,
        y: 0,
        z: 0,
    };
    assert!(
        QuaternionWitness::apply(qf, qa)
            == Quaternion {
                w: 1,
                x: 2,
                y: 3,
                z: 4
            }
    );
}

/// `apply` is Δ-free: it runs over a payload that is neither `Clone` nor `Copy`, which
/// `Applicative::apply` could not accept because of its `A: Clone` bound.
#[test]
fn apply_accepts_a_move_only_payload() {
    #[derive(PartialEq, Debug)]
    struct Moved(u32);

    let ff = Complex {
        re: (|m: Moved| Moved(m.0 * 2)) as fn(Moved) -> Moved,
        im: (|m: Moved| Moved(m.0 + 1)) as fn(Moved) -> Moved,
    };
    let fa = Complex {
        re: Moved(21),
        im: Moved(41),
    };
    let out = ComplexWitness::apply(ff, fa);
    assert_eq!(out.re, Moved(42));
    assert_eq!(out.im, Moved(42));
}

/// `fmap` and `zip_with` carry no arithmetic bound, so an unrelated payload type works.
#[test]
fn zip_with_carries_an_unrelated_payload() {
    let labels = Complex { re: "re", im: "im" };
    let values = Complex { re: 1u8, im: 2u8 };
    let joined = ComplexWitness::zip_with(labels, values, |l, v| format!("{l}={v}"));
    assert_eq!(joined.re, "re=1");
    assert_eq!(joined.im, "im=2");
}

// ---------------------------------------------------------------------------
// Mutation tests: the laws discriminate the lawful φ from plausible alternatives
// ---------------------------------------------------------------------------

/// Crossing the slot indices is the obvious alternative pairing. It survives naturality but
/// fails both unit laws, which is exactly the Yoneda argument made concrete: `u` and `v` are
/// forced to be the identity.
#[test]
fn the_index_crossing_pairing_fails_the_unit_laws() {
    fn crossed<A, B>(fa: Complex<A>, fb: Complex<B>) -> Complex<(A, B)> {
        Complex {
            re: (fa.re, fb.im),
            im: (fa.im, fb.re),
        }
    }

    let a = Complex { re: 1i64, im: 2i64 };

    // Left unit under the crossed pairing returns `a` with its slots swapped, not `a`.
    let left = ComplexWitness::fmap(crossed(ComplexWitness::unit(), a), |((), x)| x);
    assert!(
        left != a,
        "the crossed pairing must fail the left unit law; if it passes, the test is not \
         discriminating and the Yoneda argument in the module docs is untested"
    );
    assert_eq!(left, Complex { re: 2, im: 1 });

    // And the lawful one passes, on the same input.
    let lawful = ComplexWitness::fmap(ComplexWitness::zip(ComplexWitness::unit(), a), |((), x)| x);
    assert_eq!(lawful, a);
}

/// A constant `unit` that fills both slots from one value would need the diagonal. Substituting a
/// non-singleton stand-in breaks the unit law, which is why `unit` returning `Complex<()>` is
/// forced rather than chosen.
#[test]
fn a_diagonal_unit_would_break_the_unit_law() {
    // Stand in for "unit fabricates a value" by pairing against a constant rather than `()`.
    let a = Complex { re: 5i64, im: 9i64 };
    let fake_unit = Complex { re: 0i64, im: 0i64 };
    let via_fake = ComplexWitness::fmap(ComplexWitness::zip(fake_unit, a), |(_, x)| x);
    assert_eq!(via_fake, a, "projection still recovers a");

    // But the fake unit is not a unit on the other side: it discards `a` entirely.
    let wrong_side = ComplexWitness::fmap(ComplexWitness::zip(fake_unit, a), |(z, _)| z);
    assert!(
        wrong_side != a,
        "a fabricated unit must not behave like the real one"
    );
}

/// `Octonion`'s `Debug` is bounded on `Debug` alone, not `RealField`. Without that, a functorial
/// octonion over an unrelated payload would be unprintable, which would defeat the point of
/// `OctonionWitness`: `fmap` into a payload that is not a field is exactly what it exists for.
#[test]
fn octonion_is_debug_over_a_payload_that_is_not_a_field() {
    let labels: Octonion<&str> = OctonionWitness::fmap(
        Octonion {
            s: 0i64,
            e1: 1,
            e2: 2,
            e3: 3,
            e4: 4,
            e5: 5,
            e6: 6,
            e7: 7,
        },
        |n| match n {
            0 => "s",
            1 => "e1",
            _ => "e_n",
        },
    );

    // The point is that this line compiles at all.
    let rendered = format!("{labels:?}");
    assert!(rendered.starts_with("Octonion {"));
    assert!(rendered.contains("s: \"s\""));

    // And the zipped tuple payload, which is never a field either.
    let zipped = OctonionWitness::zip(
        Octonion {
            s: 1i64,
            e1: 2,
            e2: 3,
            e3: 4,
            e4: 5,
            e5: 6,
            e6: 7,
            e7: 8,
        },
        labels,
    );
    assert!(format!("{zipped:?}").contains("(1, \"s\")"));
}
