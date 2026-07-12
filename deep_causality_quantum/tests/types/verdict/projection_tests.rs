/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::Verdict;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::Projection;
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;
type P2 = Projection<f64, 2>;

fn c(re: f64, im: f64) -> C {
    Complex::new(re, im)
}

fn ket(v: Vec<C>) -> CausalTensor<C> {
    CausalTensor::new(v, vec![2, 1]).unwrap()
}

fn p0() -> P2 {
    Projection::from_ket(&ket(vec![c(1., 0.), c(0., 0.)])).unwrap()
}
fn p1() -> P2 {
    Projection::from_ket(&ket(vec![c(0., 0.), c(1., 0.)])).unwrap()
}
fn pplus() -> P2 {
    Projection::from_ket(&ket(vec![c(1., 0.), c(1., 0.)])).unwrap()
}
fn pi_state() -> P2 {
    // |i> = (|0> + i|1>)/√2
    Projection::from_ket(&ket(vec![c(1., 0.), c(0., 1.)])).unwrap()
}

fn approx_eq(a: &P2, b: &P2) -> bool {
    a.matrix()
        .as_slice()
        .iter()
        .zip(b.matrix().as_slice())
        .all(|(x, y)| (x.re - y.re).abs() < 1e-9 && (x.im - y.im).abs() < 1e-9)
}

// =============================================================================
// Construction and validation
// =============================================================================

#[test]
fn test_valid_projection_accepted() {
    let p = p0();
    assert_eq!(p.rank(), 1);
    assert_eq!(P2::zero().rank(), 0);
    assert_eq!(P2::one().rank(), 2);
}

#[test]
fn test_non_idempotent_rejected() {
    // diag(1, 0.5) is Hermitian but not a projection.
    let m = CausalTensor::new(
        vec![c(1., 0.), c(0., 0.), c(0., 0.), c(0.5, 0.)],
        vec![2, 2],
    )
    .unwrap();
    assert!(P2::new(m).is_err());
}

#[test]
fn test_non_hermitian_rejected() {
    let m =
        CausalTensor::new(vec![c(1., 0.), c(1., 0.), c(0., 0.), c(0., 0.)], vec![2, 2]).unwrap();
    assert!(P2::new(m).is_err());
}

#[test]
fn test_dim_mismatch_rejected() {
    let m3 = CausalTensor::new(vec![c(0., 0.); 9], vec![3, 3]).unwrap();
    assert!(P2::new(m3).is_err());
    // A 3-vector column against a D = 2 projection is rejected.
    let long = CausalTensor::new(vec![c(1., 0.), c(0., 0.), c(0., 0.)], vec![3, 1]).unwrap();
    assert!(Projection::<f64, 2>::from_ket(&long).is_err());
}

// =============================================================================
// Bounded lattice + orthocomplement laws
// =============================================================================

#[test]
fn test_bounded_lattice_identities() {
    let p = p0();
    let top = P2::top();
    let bot = P2::bottom();
    // P ∧ ⊤ = P, P ∨ ⊥ = P
    assert!(approx_eq(&p.clone().meet(top.clone()), &p));
    assert!(approx_eq(&p.clone().join(bot.clone()), &p));
    // P ∧ ⊥ = ⊥, P ∨ ⊤ = ⊤
    assert!(approx_eq(&p.clone().meet(bot.clone()), &bot));
    assert!(approx_eq(&p.clone().join(top.clone()), &top));
}

#[test]
fn test_complement_is_an_involution() {
    for p in [p0(), pplus(), pi_state(), P2::zero(), P2::one()] {
        let pp = p.clone().complement().complement();
        assert!(approx_eq(&pp, &p), "¬¬P ≠ P");
    }
    // ¬⊥ = ⊤, ¬⊤ = ⊥
    assert!(approx_eq(&P2::bottom().complement(), &P2::top()));
    assert!(approx_eq(&P2::top().complement(), &P2::bottom()));
}

#[test]
fn test_orthocomplement_laws() {
    for p in [p0(), pplus(), pi_state()] {
        let np = p.clone().complement();
        // P ∨ ¬P = ⊤
        assert!(
            approx_eq(&p.clone().join(np.clone()), &P2::top()),
            "P ∨ P^⊥ ≠ ⊤"
        );
        // P ∧ ¬P = ⊥
        assert!(
            approx_eq(&p.clone().meet(np.clone()), &P2::bottom()),
            "P ∧ P^⊥ ≠ ⊥"
        );
    }
}

#[test]
fn test_de_morgan() {
    let (p, q) = (p0(), pplus());
    // ¬(P ∧ Q) = ¬P ∨ ¬Q
    let lhs = p.clone().meet(q.clone()).complement();
    let rhs = p.clone().complement().join(q.clone().complement());
    assert!(approx_eq(&lhs, &rhs));
}

// =============================================================================
// The join/meet of distinct lines span/intersect correctly
// =============================================================================

#[test]
fn test_two_distinct_lines_span_the_space() {
    // Any two distinct 1-D subspaces of C² join to ⊤ and meet to ⊥.
    let pairs = [(p0(), p1()), (p0(), pplus()), (pplus(), pi_state())];
    for (a, b) in pairs {
        assert!(approx_eq(&a.clone().join(b.clone()), &P2::top()));
        assert!(approx_eq(&a.clone().meet(b.clone()), &P2::bottom()));
    }
}

#[test]
fn test_idempotent_lattice_ops() {
    let p = pplus();
    assert!(approx_eq(&p.clone().join(p.clone()), &p)); // P ∨ P = P
    assert!(approx_eq(&p.clone().meet(p.clone()), &p)); // P ∧ P = P
}

// =============================================================================
// Orthomodular law holds; distributivity FAILS
// =============================================================================

#[test]
fn test_orthomodular_law() {
    // a ≤ c ⟹ a ∨ (¬a ∧ c) = c. Take a = |0⟩, c = ⊤ (a ≤ ⊤ always).
    let a = p0();
    let c_top = P2::top();
    assert!(a.leq(&c_top));
    let lhs = a.clone().join(a.clone().complement().meet(c_top.clone()));
    assert!(approx_eq(&lhs, &c_top), "orthomodular law violated");

    // A non-trivial c: a = |0⟩ ≤ c = |0⟩ (a ≤ a). Then a ∨ (¬a ∧ a) = a ∨ ⊥ = a.
    let lhs2 = a.clone().join(a.clone().complement().meet(a.clone()));
    assert!(approx_eq(&lhs2, &a));
}

#[test]
fn test_distributivity_fails_on_general_position_triple() {
    // The canonical witness: a = |0⟩, b = |1⟩, c = |+⟩ pairwise non-commuting
    // enough that a ∧ (b ∨ c) ≠ (a ∧ b) ∨ (a ∧ c).
    let (a, b, cc) = (p0(), p1(), pplus());
    // b ∨ c = ⊤ (distinct lines) ⇒ a ∧ (b ∨ c) = a ∧ ⊤ = a.
    let lhs = a.clone().meet(b.clone().join(cc.clone()));
    // a ∧ b = ⊥, a ∧ c = ⊥ ⇒ (a∧b) ∨ (a∧c) = ⊥.
    let rhs = a.clone().meet(b.clone()).join(a.clone().meet(cc.clone()));
    assert!(approx_eq(&lhs, &a), "LHS should be |0⟩");
    assert!(approx_eq(&rhs, &P2::bottom()), "RHS should be ⊥");
    assert!(!approx_eq(&lhs, &rhs), "distributivity must FAIL here");
}

// =============================================================================
// Predicates
// =============================================================================

#[test]
fn test_leq_and_commutes() {
    let p = p0();
    // ⊥ ≤ P ≤ ⊤.
    assert!(P2::bottom().leq(&p));
    assert!(p.leq(&P2::top()));
    // Commuting family: |0⟩ and |1⟩ commute (both diagonal); |0⟩ and |+⟩ do not.
    assert!(p0().commutes_with(&p1()));
    assert!(!p0().commutes_with(&pplus()));
    // A projection commutes with itself, ⊥, and ⊤.
    assert!(p0().commutes_with(&p0()));
    assert!(p0().commutes_with(&P2::bottom()));
    assert!(p0().commutes_with(&P2::top()));
}

#[test]
fn test_distributivity_holds_within_a_commuting_family() {
    // For commuting |0⟩, |1⟩ (and any third commuting projection) distributivity
    // is restored — the lattice is Boolean on a single commuting family.
    let (a, b, cc) = (p0(), p1(), P2::top());
    assert!(a.commutes_with(&b) && a.commutes_with(&cc) && b.commutes_with(&cc));
    let lhs = a.clone().meet(b.clone().join(cc.clone()));
    let rhs = a.clone().meet(b.clone()).join(a.clone().meet(cc.clone()));
    assert!(approx_eq(&lhs, &rhs));
}
