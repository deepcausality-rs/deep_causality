/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the symmetric-monoidal PROP (`SymMonoidal`): copy comonoid `Δ`/`ε`, merge
//! monoid `∇`/`η`, and symmetry `σ`.
//!
//! Mirrors `lean/DeepCausalityFormal/Haft/SymmetricMonoidal.lean`:
//!
//! * `haft.monoidal.comonoid_laws` — the diagonal comonoid: coassociativity (up to the associator),
//!   counit (discard one copied half ⇒ identity), cocommutativity (`swap ∘ copy = copy`).
//! * `haft.monoidal.merge_monoid_laws` — `∇`/`η` associativity + unit, which are the [`Monoid`]
//!   laws (witnessed over `Count`).
//! * `haft.monoidal.symmetry` — `swap` is its own inverse (`σ ∘ σ = id`).
//!
//! Plus the scoped copy–merge coherence (no separate Lean id): `Δ` is a monoid homomorphism
//! (bialgebra law), and over the commutative `Count` monoid `∇` is `swap`-invariant.

use deep_causality_algebra::Count;
use deep_causality_haft::SymMonoidal;

/// THEOREM_MAP: haft.monoidal.comonoid_laws
#[test]
fn test_monoidal_comonoid_laws() {
    for a in [0_i32, 7, -4] {
        // Coassociativity (up to the associator): flattening `(Δ ⊗ id) ∘ Δ` and `(id ⊗ Δ) ∘ Δ`
        // gives the same triple `(a, a, a)`.
        let (a1, a2) = SymMonoidal::copy(a);
        let ((a11, a12), a2_r) = (SymMonoidal::copy(a1), a2);
        let (a1_l, (a21, a22)) = (a1, SymMonoidal::copy(a2));
        assert_eq!((a11, a12, a2_r), (a1_l, a21, a22));

        // Counit: both copied halves equal the original (discard the other ⇒ identity).
        let (l, r) = SymMonoidal::copy(a);
        assert_eq!(l, a);
        assert_eq!(r, a);
        SymMonoidal::discard(r); // ε consumes the discarded half

        // Cocommutativity: swap ∘ copy = copy.
        assert_eq!(
            SymMonoidal::swap(SymMonoidal::copy(a)),
            SymMonoidal::copy(a)
        );
    }
}

/// THEOREM_MAP: haft.monoidal.merge_monoid_laws
#[test]
fn test_monoidal_merge_monoid_laws() {
    let xs = [Count(0), Count(1), Count(5), Count(9)];
    for &x in &xs {
        // Unit (η): merge with the unit on either side is the identity.
        assert_eq!(SymMonoidal::merge((SymMonoidal::unit(), x)), x);
        assert_eq!(SymMonoidal::merge((x, SymMonoidal::unit::<Count>())), x);
        for &y in &xs {
            for &z in &xs {
                // Associativity of ∇.
                let left = SymMonoidal::merge((SymMonoidal::merge((x, y)), z));
                let right = SymMonoidal::merge((x, SymMonoidal::merge((y, z))));
                assert_eq!(left, right);
            }
        }
    }
}

/// THEOREM_MAP: haft.monoidal.symmetry
#[test]
fn test_monoidal_symmetry() {
    // swap is its own inverse: σ ∘ σ = id.
    for p in [(1_i32, 'a'), (-3, 'z'), (0, ' ')] {
        assert_eq!(SymMonoidal::swap(SymMonoidal::swap(p)), p);
    }

    // Over the commutative Count monoid, ∇ is swap-invariant: ∇ ∘ σ = ∇.
    for x in [Count(1), Count(4)] {
        for y in [Count(2), Count(7)] {
            assert_eq!(
                SymMonoidal::merge(SymMonoidal::swap((x, y))),
                SymMonoidal::merge((x, y))
            );
        }
    }
}

/// THEOREM_MAP: haft.monoidal.comonoid_laws
///
/// Scoped copy–merge coherence: `Δ` is a monoid homomorphism (the bialgebra law
/// `Δ(x ∇ y) = Δx ∇ Δy`, componentwise), for every monoid.
#[test]
fn test_monoidal_copy_merge_bialgebra() {
    for x in [Count(0), Count(3), Count(6)] {
        for y in [Count(1), Count(8)] {
            // Δ(x ∇ y)
            let lhs = SymMonoidal::copy(SymMonoidal::merge((x, y)));
            // Δx ∇ Δy, componentwise
            let (x1, x2) = SymMonoidal::copy(x);
            let (y1, y2) = SymMonoidal::copy(y);
            let rhs = (SymMonoidal::merge((x1, y1)), SymMonoidal::merge((x2, y2)));
            assert_eq!(lhs, rhs);
        }
    }
}
