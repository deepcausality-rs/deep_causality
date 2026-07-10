/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the choice fragment `⊕` (ArrowChoice) — `Left`, `Right`, `Choice` (`+++`),
//! `Fanin` (`|||`) over `Either`.
//!
//! Mirrors `lean/DeepCausalityFormal/Haft/ArrowChoice.lean` (`haft.arrow_choice.laws`): the
//! ArrowChoice laws of Hughes 2000 §5 in the eager model, `fanin` as the coproduct elimination,
//! and the `⊗`-over-`⊕` distributivity equations used (Lorenz & Barrett 2021 §4). Lean proves ∀;
//! these tests pin the crate's real combinators to the same statements at representative inputs.

use deep_causality_haft::{Arrow, Either, Lift};

fn inc(x: i64) -> i64 {
    x + 1
}

fn double(x: i64) -> i64 {
    x * 2
}

fn neg(x: i64) -> i64 {
    -x
}

const SAMPLES: [i64; 4] = [0, 1, -3, 10];

/// THEOREM_MAP: haft.arrow_choice.laws
///
/// The routing laws: `left (arr f) = arr (f ⊕ id)`, functoriality `left (f >>> g) = left f >>>
/// left g`, the exchange law with a passive-summand map, and the injection unit law.
#[test]
fn test_arrow_choice_left_laws() {
    for x in SAMPLES {
        for input in [Either::<i64, i64>::Left(x), Either::Right(x)] {
            // left (arr f) = arr (f ⊕ id): `left` IS choice(f, id) on pure arrows.
            assert_eq!(
                Lift::new(inc).left::<i64>().run(input),
                Lift::new(inc).choice(Lift::new(|c: i64| c)).run(input)
            );

            // Functoriality: left (f >>> g) = left f >>> left g.
            assert_eq!(
                Lift::new(inc)
                    .compose(Lift::new(double))
                    .left::<i64>()
                    .run(input),
                Lift::new(inc)
                    .left::<i64>()
                    .compose(Lift::new(double).left::<i64>())
                    .run(input)
            );

            // Exchange: left f >>> (id +++ g) = (id +++ g) >>> left f.
            assert_eq!(
                Lift::new(inc)
                    .left::<i64>()
                    .compose(Lift::new(|b: i64| b).choice(Lift::new(neg)))
                    .run(input),
                Lift::new(|a: i64| a)
                    .choice(Lift::new(neg))
                    .compose(Lift::new(inc).left::<i64>())
                    .run(input)
            );
        }

        // Injection unit: f >>> arr Left = arr Left >>> left f.
        assert_eq!(
            Lift::new(inc).left::<i64>().run(Either::Left(x)),
            Either::<i64, i64>::Left(inc(x))
        );
        // Both composites of the injection unit law agree, not just the `Left`-arm value:
        // f >>> arr Left = arr Left >>> left f, built through the arrow API on each side.
        assert_eq!(
            Lift::new(inc)
                .compose(Lift::new(Either::<i64, i64>::Left))
                .run(x),
            Lift::new(Either::<i64, i64>::Left)
                .compose(Lift::new(inc).left::<i64>())
                .run(x)
        );

        // `right` mirrors `left` on the other summand.
        assert_eq!(
            Lift::new(inc)
                .right::<i64>()
                .run(Either::<i64, i64>::Right(x)),
            Either::Right(inc(x))
        );
        assert_eq!(
            Lift::new(inc)
                .right::<i64>()
                .run(Either::<i64, i64>::Left(x)),
            Either::Left(x)
        );
    }
}

/// THEOREM_MAP: haft.arrow_choice.laws
///
/// `fanin` (`|||`) is the coproduct elimination: computation on both injections, agreement with
/// any map satisfying the same arm equations (uniqueness, spot-checked), and the absorption law
/// `(f +++ g) >>> (h ||| k) = (f >>> h) ||| (g >>> k)`.
#[test]
fn test_arrow_choice_fanin_elimination() {
    let fanin = Lift::new(inc).fanin(Lift::new(neg));
    for x in SAMPLES {
        // Computation rules: (f ||| g) ∘ inl = f, (f ||| g) ∘ inr = g.
        assert_eq!(fanin.run(Either::Left(x)), inc(x));
        assert_eq!(fanin.run(Either::Right(x)), neg(x));

        // Uniqueness (spot-check): a by-hand eliminator with the same arms agrees everywhere.
        let by_hand = |e: Either<i64, i64>| match e {
            Either::Left(a) => inc(a),
            Either::Right(c) => neg(c),
        };
        assert_eq!(fanin.run(Either::Left(x)), by_hand(Either::Left(x)));
        assert_eq!(fanin.run(Either::Right(x)), by_hand(Either::Right(x)));

        // Absorption: (f +++ g) >>> (h ||| k) = (f >>> h) ||| (g >>> k).
        for input in [Either::<i64, i64>::Left(x), Either::Right(x)] {
            assert_eq!(
                Lift::new(inc)
                    .choice(Lift::new(double))
                    .compose(Lift::new(neg).fanin(Lift::new(inc)))
                    .run(input),
                Lift::new(inc)
                    .compose(Lift::new(neg))
                    .fanin(Lift::new(double).compose(Lift::new(inc)))
                    .run(input)
            );
        }
    }
}

/// THEOREM_MAP: haft.arrow_choice.laws
///
/// The `⊗`-over-`⊕` distributivity equations used: `distl : α × (β ⊕ γ) → (α × β) ⊕ (α × γ)` and
/// its inverse round-trip, and naturality of `distl` in all three components — pairs distribute
/// over sums, the rig coherence faithful direct-sum decompositions rely on.
#[test]
fn test_arrow_choice_distributivity() {
    fn distl(p: (i64, Either<i64, i64>)) -> Either<(i64, i64), (i64, i64)> {
        match p {
            (a, Either::Left(b)) => Either::Left((a, b)),
            (a, Either::Right(c)) => Either::Right((a, c)),
        }
    }
    fn undistl(s: Either<(i64, i64), (i64, i64)>) -> (i64, Either<i64, i64>) {
        match s {
            Either::Left((a, b)) => (a, Either::Left(b)),
            Either::Right((a, c)) => (a, Either::Right(c)),
        }
    }

    for a in SAMPLES {
        for x in SAMPLES {
            for s in [Either::<i64, i64>::Left(x), Either::Right(x)] {
                // Round trips: undistl ∘ distl = id and distl ∘ undistl = id.
                assert_eq!(undistl(distl((a, s))), (a, s));
                assert_eq!(distl(undistl(distl((a, s)))), distl((a, s)));
                // Full `distl ∘ undistl = id`, on independent sum values (not just the image
                // of `distl`): both injections of `(α × β) ⊕ (α × γ)` round-trip to themselves.
                let y_left = Either::<(i64, i64), (i64, i64)>::Left((a, x));
                assert_eq!(distl(undistl(y_left)), y_left);
                let y_right = Either::<(i64, i64), (i64, i64)>::Right((a, x));
                assert_eq!(distl(undistl(y_right)), y_right);

                // Naturality: distl ∘ (f ⊗ (g ⊕ h)) = ((f ⊗ g) ⊕ (f ⊗ h)) ∘ distl.
                let lhs = distl((inc(a), Lift::new(double).choice(Lift::new(neg)).run(s)));
                let rhs = match distl((a, s)) {
                    Either::Left((p, q)) => Either::Left((inc(p), double(q))),
                    Either::Right((p, q)) => Either::Right((inc(p), neg(q))),
                };
                assert_eq!(lhs, rhs);
            }
        }
    }
}
