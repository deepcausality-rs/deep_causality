/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the Lean orthomodular-lattice proofs.
//!
//! Mirrors `lean/DeepCausalityFormal/Quantum/Orthomodular.lean`:
//! `subspace_lattice_orthomodular` (bounded lattice, antitone involutive
//! orthocomplement, complements, the orthomodular law) and
//! `subspace_lattice_not_distributive`. The Lean statements are on subspaces;
//! the `Projection` carrier stores the projection onto each one, so these
//! witnesses compare projections.
//!
//! The orthomodular law is checked in three dimensions, where a line can sit
//! strictly inside a plane that is not the whole space. In two dimensions every
//! comparable pair is trivial (`⊥`, equal, or `⊤`), and there the law holds in
//! any orthocomplemented lattice.

use deep_causality_algebra::Verdict;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::Projection;
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;
type P2 = Projection<f64, 2>;
type P3 = Projection<f64, 3>;

fn line<const D: usize>(v: [f64; D]) -> Projection<f64, D> {
    let data: Vec<C> = v.iter().map(|&x| Complex::new(x, 0.0)).collect();
    Projection::from_ket(&CausalTensor::new(data, vec![D, 1]).unwrap()).unwrap()
}

fn approx_eq<const D: usize>(a: &Projection<f64, D>, b: &Projection<f64, D>) -> bool {
    a.matrix()
        .as_slice()
        .iter()
        .zip(b.matrix().as_slice())
        .all(|(x, y)| (x.re - y.re).abs() < 1e-9 && (x.im - y.im).abs() < 1e-9)
}

/// The plane spanned by |0⟩ and |1⟩ in ℂ³: neither ⊥ nor ⊤.
fn plane01() -> P3 {
    line([1.0, 0.0, 0.0]).join(line([0.0, 1.0, 0.0]))
}

// THEOREM_MAP: quantum.verdict.orthomodular
#[test]
fn test_the_orthocomplement_laws_hold_on_a_plane_in_three_dimensions() {
    // Lean: subspace_lattice_orthomodular, its first four conjuncts.
    let k = plane01();
    let kc = k.clone().complement();
    assert!(!approx_eq(&k, &P3::bottom()) && !approx_eq(&k, &P3::top()));

    // ⊥ ≤ K ≤ ⊤.
    assert!(P3::bottom().leq(&k) && k.leq(&P3::top()));
    // Kᗮᗮ = K.
    assert!(approx_eq(&kc.clone().complement(), &k));
    // K ⊓ Kᗮ = ⊥ and K ⊔ Kᗮ = ⊤.
    assert!(approx_eq(&k.clone().meet(kc.clone()), &P3::bottom()));
    assert!(approx_eq(&k.clone().join(kc.clone()), &P3::top()));
    // Antitone: L ≤ K implies Kᗮ ≤ Lᗮ, with L the |+⟩ line inside K.
    let l = line([1.0, 1.0, 0.0]);
    assert!(l.leq(&k));
    assert!(kc.leq(&l.complement()));
}

// THEOREM_MAP: quantum.verdict.orthomodular
#[test]
fn test_the_orthomodular_law_holds_for_a_line_inside_a_plane() {
    // Lean: subspace_lattice_orthomodular, the law K₁ ≤ K₂ → K₁ ⊔ (K₁ᗮ ⊓ K₂) = K₂,
    // with K₁ the |+⟩ line strictly inside K₂ = span{|0⟩, |1⟩}.
    let k1 = line([1.0, 1.0, 0.0]);
    let k2 = plane01();
    assert!(k1.leq(&k2));
    assert!(!approx_eq(&k1, &k2));

    // K₁ᗮ ⊓ K₂ is the |−⟩ line: the part of K₂ the law has to recover.
    let middle = k1.clone().complement().meet(k2.clone());
    assert!(approx_eq(&middle, &line([1.0, -1.0, 0.0])));
    assert!(approx_eq(&k1.join(middle), &k2), "orthomodular law");
}

// THEOREM_MAP: quantum.verdict.orthomodular
#[test]
fn test_distributivity_fails_on_two_independent_lines_and_their_sum() {
    // Lean: subspace_lattice_not_distributive with u = |0⟩, v = |1⟩, so the
    // third line is the one through u + v, the line of |+⟩.
    let ku: P2 = line([1.0, 0.0]);
    let kv: P2 = line([0.0, 1.0]);
    let kp: P2 = line([1.0, 1.0]);
    let lhs = ku.clone().meet(kv.clone().join(kp.clone()));
    let rhs = ku.clone().meet(kv).join(ku.clone().meet(kp));
    assert!(approx_eq(&lhs, &ku), "K_u ⊓ (K_v ⊔ K_(u+v)) contains u");
    assert!(approx_eq(&rhs, &P2::bottom()), "both pairwise meets are ⊥");
    assert!(!approx_eq(&lhs, &rhs), "distributivity must fail");
}
