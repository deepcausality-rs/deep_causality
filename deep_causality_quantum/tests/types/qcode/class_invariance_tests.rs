/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Diagonal logical gates, and the class-invariance check built on them.
//!
//! Junichi Haruna, arXiv:2511.15224, §3.1 and Appendix B. The gates are Table 1's gauge-field
//! column normalised to `exp(2πi·Q(n)/M)`; the invariance argument is Eq. (3.20), which decomposes
//! the difference between two representatives into factors and shows each is logically trivial.
//!
//! Three external anchors pin the expectations, none of them a reading of this workspace:
//!
//! - The single-qubit gate definitions. At `n = 1` the polynomials must give `Z = diag(1, −1)`,
//!   `S = diag(1, i)` and `T = diag(1, e^{iπ/4})` (Nielsen & Chuang §4.2).
//! - `β₁(T²) = 2` over 𝔽₂ (Hatcher, Example 2.36), so the toric code encodes two logical qubits.
//! - Haruna Eq. (3.22): `S(γ₁) ~ S(γ₂)` when `γ₁ = γ₂ + ∂₂f`, which is the property under test.

use deep_causality_homology::utils_tests::reference_spaces;
use deep_causality_homology::{ChainComplex, Gf2Chain};
use deep_causality_linear::{MatrixBuild, MatrixView, PackedGf2, csr_to_packed_gf2_mod2, rank_gf2};
use deep_causality_num::Gf2;
use deep_causality_quantum::{DiagonalPhase, LogicalBasis, LogicalPauli};

type W = u64;
type Chain = Gf2Chain<W>;

fn torus() -> impl ChainComplex {
    reference_spaces()
        .into_iter()
        .find(|(f, _, _)| f.name() == "torus_2")
        .expect("the fixture set carries torus_2")
        .0
}

fn basis() -> LogicalBasis<W> {
    LogicalBasis::from_complex(&torus(), 1).unwrap()
}

/// Every `∂₂` column: the boundaries, which are the stabilizers.
fn boundaries(code: &impl ChainComplex) -> Vec<Chain> {
    let d2 = csr_to_packed_gf2_mod2::<W>(&code.boundary_matrix(2));
    (0..d2.cols())
        .map(|c| Chain::from_column(&d2, c, 1).unwrap())
        .collect()
}

// ---------------------------------------------------------------------------
// The polynomials reproduce the gates they are named for.
// ---------------------------------------------------------------------------

#[test]
fn test_the_phase_polynomials_match_the_single_qubit_gates() {
    let one = Chain::from_support(4, 1, &[0]).unwrap();
    // Z = diag(1, −1): a half turn.
    assert_eq!(DiagonalPhase::z(one.clone()).phase_at(1), Rational(1, 2));
    // S = diag(1, i): a quarter turn.
    assert_eq!(DiagonalPhase::s(one.clone()).phase_at(1), Rational(1, 4));
    // T = diag(1, e^{iπ/4}): an eighth turn.
    assert_eq!(DiagonalPhase::t(one.clone()).phase_at(1), Rational(1, 8));
    // And all three are the identity on |0>.
    for g in [
        DiagonalPhase::z(one.clone()),
        DiagonalPhase::s(one.clone()),
        DiagonalPhase::t(one),
    ] {
        assert_eq!(g.phase_at(0), Rational(0, 1));
    }
}

#[allow(non_snake_case)]
fn Rational(n: i64, d: i64) -> deep_causality_num_rational::Rational<i64> {
    deep_causality_num_rational::Rational::new(n, d)
}

#[test]
fn test_the_t_polynomial_has_period_eight() {
    // T has order 8, so the phase must return to a whole turn after eight steps of the argument
    // that a single-qubit T advances. This pins the exponent, which a coefficient typo would move.
    let g = DiagonalPhase::t(Chain::from_support(16, 1, &[0]).unwrap());
    assert_eq!(g.phase_at(1) - g.phase_at(0), Rational(1, 8));
    assert_eq!(g.modulus(), 8);
}

// ---------------------------------------------------------------------------
// The new predicate must agree with the shipped Pauli one wherever both apply.
// ---------------------------------------------------------------------------

#[test]
fn test_the_diagonal_predicate_agrees_with_the_pauli_one_on_z() {
    // Z̄(γ) is both a Pauli and a diagonal gate, so the two deciders must never disagree. This is
    // the cross-check that keeps the generalisation honest: it is the one gate family where the
    // old, independently-tested path still applies.
    let code = torus();
    let b = basis();
    let n = code.num_cells(1);
    let zero = Chain::zeros(n, 1);

    let mut checked = 0;
    for gamma in b.homology().iter().chain(boundaries(&code).iter()) {
        let pauli = LogicalPauli::new(zero.clone(), gamma.clone()).unwrap();
        let diagonal = DiagonalPhase::z(gamma.clone());
        assert_eq!(
            b.is_logically_trivial(&pauli).unwrap(),
            b.is_diagonal_trivial(&diagonal).unwrap(),
            "the two deciders disagree on a Z gate"
        );
        checked += 1;
    }
    assert!(checked >= 4, "the agreement was checked on too few chains");
}

#[test]
fn test_a_nontrivial_class_is_not_trivial_and_a_boundary_is() {
    // The discriminating pair: a homology generator acts, a stabilizer does not. Both at S̄, where
    // the Pauli predicate cannot reach.
    let code = torus();
    let b = basis();
    for gamma in b.homology() {
        assert!(
            !b.is_diagonal_trivial(&DiagonalPhase::z(gamma.clone()))
                .unwrap(),
            "a non-trivial class must act"
        );
    }
    for boundary in boundaries(&code) {
        assert!(
            b.is_diagonal_trivial(&DiagonalPhase::s(boundary.clone()))
                .unwrap(),
            "Haruna Eq. (3.21): S(∂₂f) is logically trivial"
        );
    }
}

// ---------------------------------------------------------------------------
// Class invariance itself: Eq. (3.22).
// ---------------------------------------------------------------------------

#[test]
fn test_every_gate_acts_on_the_class_not_the_representative() {
    // The property the geometric-QEC consumer checks. For each of the two toric classes and each
    // gate, moving to another representative by any boundary must not change the logical action.
    let code = torus();
    let b = basis();
    let bs = boundaries(&code);
    assert!(!bs.is_empty(), "the torus must have boundaries to shift by");

    for gamma in b.homology() {
        for gate in [
            DiagonalPhase::z(gamma.clone()),
            DiagonalPhase::s(gamma.clone()),
            DiagonalPhase::t(gamma.clone()),
        ] {
            let report = b.check_class_invariance(&gate, &bs).unwrap();
            assert!(
                report.holds,
                "class invariance failed at {:?}",
                report.first_failure
            );
            assert!(report.tested > 0, "a vacuous pass must not read as a pass");
            assert_eq!(report.first_failure, None);
        }
    }
}

#[test]
fn test_the_check_reports_what_it_examined() {
    // Section 3.2's obligation: a gate that examined nothing has not agreed with you.
    let b = basis();
    let gamma = b.homology()[0].clone();
    let empty: [Chain; 0] = [];
    let report = b
        .check_class_invariance(&DiagonalPhase::s(gamma), &empty)
        .unwrap();
    assert!(report.holds);
    assert_eq!(report.tested, 0, "no boundaries means nothing was examined");
}

#[test]
fn test_a_shift_by_a_non_boundary_can_change_the_class() {
    // The check must discriminate. Shifting by a chain that is NOT a boundary moves to a different
    // homology class, and the gate's action is then permitted to differ — so a check that always
    // returned `holds` would pass everything above and fail here.
    let b = basis();
    let g0 = b.homology()[0].clone();
    let g1 = b.homology()[1].clone();
    // g1 is a non-trivial class, so shifting g0 by it lands in a different class.
    let report = b
        .check_class_invariance(&DiagonalPhase::z(g0), core::slice::from_ref(&g1))
        .unwrap();
    assert!(
        !report.holds,
        "shifting by an independent class must be detected as a change of class"
    );
    assert_eq!(report.tested, 1);
}

#[test]
fn test_register_width_mismatches_are_rejected() {
    let b = basis();
    let wrong = DiagonalPhase::z(Chain::zeros(3, 1));
    assert!(b.is_diagonal_trivial(&wrong).is_err());
    assert!(b.check_class_invariance(&wrong, &[]).is_err());
}

// ---------------------------------------------------------------------------
// The stabilizer generators the decision rests on.
// ---------------------------------------------------------------------------

#[test]
fn test_the_stabilizers_are_an_independent_basis_of_the_boundaries() {
    // `check_class_invariance` decides over the code space, and the code space is named by these
    // chains alone. A generating set that were merely the `∂₂` columns would carry a dependency for
    // every 2-cycle the complex has; the torus has one, so the basis must be one shorter than the
    // column count. That difference is the whole content of taking an image basis rather than the
    // raw columns, and `β₂(T²) = 1` over 𝔽₂ (Hatcher, Example 2.36) is what fixes it externally.
    let code = torus();
    let b = basis();
    let stabilizers = b.stabilizers();
    assert!(!stabilizers.is_empty(), "the torus has boundaries");

    let d2 = csr_to_packed_gf2_mod2::<W>(&code.boundary_matrix(2));
    assert_eq!(
        stabilizers.len(),
        d2.cols() - 1,
        "one 2-cycle means exactly one dependency among the columns"
    );

    // Independent: the rank equals the count.
    let n = code.num_cells(1);
    let mut m = PackedGf2::<W>::zeros(n, stabilizers.len());
    for (c, g) in stabilizers.iter().enumerate() {
        for r in g.support() {
            m.set(r, c, Gf2::ONE).unwrap();
        }
    }
    assert_eq!(rank_gf2(&m).unwrap(), stabilizers.len());
}

#[test]
fn test_every_stabilizer_acts_trivially_at_every_gate() {
    // Haruna Eq. (3.21) for the whole family, not just `S̄`. A chain that bounds induces a gate that
    // does nothing, and this must hold at `Z̄`, `S̄` and `T̄` alike — the property the class-invariance
    // argument consumes.
    let b = basis();
    for s in b.stabilizers() {
        for gate in [
            DiagonalPhase::z(s.clone()),
            DiagonalPhase::s(s.clone()),
            DiagonalPhase::t(s.clone()),
        ] {
            assert!(
                b.is_diagonal_trivial(&gate).unwrap(),
                "a boundary must induce a trivial gate"
            );
        }
    }
}

#[test]
fn test_the_witness_reproduces_the_failure_it_reports() {
    // A failure report is only worth what it lets you check by hand. The witness names three block
    // occupancies; recomputing the phase from them must reproduce the non-integrality that made the
    // check fail, with no appeal to the checker that produced it.
    let b = basis();
    let g0 = b.homology()[0].clone();
    let g1 = b.homology()[1].clone();
    let gate = DiagonalPhase::z(g0.clone());
    let report = b
        .check_class_invariance(&gate, core::slice::from_ref(&g1))
        .unwrap();

    let w = report.first_failure.expect("the shift changes the class");
    assert_eq!(w.boundary, 0);
    assert!(
        report.states_visited > 0,
        "a failure must have looked at something"
    );

    let shifted = gate.shifted_by(&g1).unwrap();
    let phase =
        shifted.phase_at(w.shift_only + w.gate_only) - gate.phase_at(w.shared + w.gate_only);
    assert!(
        !phase.is_integer(),
        "the reported occupancies must reproduce a non-integral phase, got {phase:?}"
    );

    // And the witness must lie inside the blocks it names.
    let both = g0.intersect(&g1).unwrap().weight() as u64;
    assert!(w.shared <= both);
    assert!(w.shift_only <= g1.weight() as u64 - both);
    assert!(w.gate_only <= g0.weight() as u64 - both);
}
