/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Logical equivalence of CSS-code Pauli operators.
//!
//! The code under test is the 2-torus, whose `H₁` is the toric code's logical space. Two external
//! facts pin the expectations, neither of them a reading of this workspace:
//!
//! - `β₁(T²) = 2` over 𝔽₂ (Hatcher, *Algebraic Topology*, Example 2.36), which the homology
//!   fixture carries and which is why the toric code encodes two logical qubits (Kitaev,
//!   *Fault-tolerant quantum computation by anyons*, 2003).
//! - Haruna arXiv:2511.15224, Theorem A.1 with B.1 to B.3: `O ~ I` exactly when `O` commutes with
//!   every logical `Z̄(γ)`, `γ ∈ H₁`, and every `X̄(γ̃)`, `γ̃ ∈ H¹`.
//!
//! The physics those combine into: a stabilizer acts trivially on the code space and a logical
//! operator does not. A boundary is a stabilizer and a non-trivial cycle is a logical operator, so
//! the predicate must separate exactly those two.

use deep_causality_homology::utils_tests::reference_spaces;
use deep_causality_homology::{ChainComplex, Gf2Chain};
use deep_causality_linear::{MatrixBuild, MatrixView, PackedGf2, csr_to_packed_gf2_mod2, rank_gf2};
use deep_causality_num::Gf2;
use deep_causality_quantum::{LogicalBasis, LogicalPauli, QuantumErrorEnum};

type W = u64;
type Chain = Gf2Chain<W>;

/// The 2-torus as a chain complex, which is the toric code.
fn torus() -> impl ChainComplex {
    reference_spaces()
        .into_iter()
        .find(|(f, _, _)| f.name() == "torus_2")
        .expect("the fixture set carries torus_2")
        .0
}

/// A basis at grade 1, where the toric code's logical qubits live.
fn basis() -> LogicalBasis<W> {
    LogicalBasis::from_complex(&torus(), 1).unwrap()
}

/// The all-zero 1-chain over the code's register.
fn zero_chain(code: &impl ChainComplex) -> Chain {
    Chain::zeros(code.num_cells(1), 1)
}

/// `Z̄(γ)`: a logical Z supported on `gamma`, with no X part.
fn logical_z_op(code: &impl ChainComplex, gamma: Chain) -> LogicalPauli<W> {
    LogicalPauli::new(zero_chain(code), gamma).unwrap()
}

#[test]
fn test_the_torus_code_encodes_two_logical_qubits() {
    // β₁(T²) = 2, so the toric code has k = 2. The basis reports the same on both sides.
    let b = basis();
    assert_eq!(b.num_logical_qubits(), 2);
    assert_eq!(b.homology().len(), 2);
    assert_eq!(b.cohomology().len(), 2);
}

#[test]
fn test_the_identity_is_logically_trivial() {
    let code = torus();
    let b = basis();
    let id = LogicalPauli::new(zero_chain(&code), zero_chain(&code)).unwrap();
    assert!(id.is_identity());
    assert!(b.is_logically_trivial(&id).unwrap());
}

#[test]
fn test_a_stabilizer_acts_trivially() {
    // A boundary is a stabilizer: it is ∂₂ of some face, so it pairs to zero with every cocycle and
    // commutes with every logical operator. This is the case that would break if the predicate
    // tested the cycle condition instead of the pairing.
    let code = torus();
    let d2 = csr_to_packed_gf2_mod2::<W>(&code.boundary_matrix(2));
    let b = basis();
    for col in 0..d2.cols() {
        let boundary = Chain::from_column(&d2, col, 1).unwrap();
        let stabilizer = logical_z_op(&code, boundary);
        assert!(
            b.is_logically_trivial(&stabilizer).unwrap(),
            "face {} produced a stabilizer the predicate called non-trivial",
            col
        );
    }
}

#[test]
fn test_a_logical_operator_does_not_act_trivially() {
    // Z̄(γ) for γ a non-trivial 1-class. The intersection pairing on the torus is non-degenerate,
    // so each class pairs to one with some cocycle and the operator anticommutes with that X̄.
    let code = torus();
    let b = basis();
    for gamma in b.homology() {
        let op = logical_z_op(&code, gamma.clone());
        assert!(
            !b.is_logically_trivial(&op).unwrap(),
            "a non-trivial homology class was reported as acting trivially"
        );
    }
}

#[test]
fn test_operators_differing_by_a_stabilizer_are_equivalent() {
    // The heart of B.1: a logical operator is defined up to stabilizers, so multiplying by a
    // boundary must not change the logical action.
    let code = torus();
    let b = basis();
    let d2 = csr_to_packed_gf2_mod2::<W>(&code.boundary_matrix(2));
    let gamma = b.homology()[0].clone();
    let op = logical_z_op(&code, gamma.clone());

    for col in 0..d2.cols() {
        let boundary = Chain::from_column(&d2, col, 1).unwrap();
        let shifted = logical_z_op(&code, gamma.add(&boundary).unwrap());
        assert!(
            b.are_logically_equivalent(&op, &shifted).unwrap(),
            "adding the boundary of face {} changed the logical class",
            col
        );
    }
}

#[test]
fn test_distinct_logical_classes_are_not_equivalent() {
    // The two torus classes are independent in H₁, so Z̄(γ₁) and Z̄(γ₂) act differently. Without
    // this the predicate could pass everything above by always answering "equivalent".
    let code = torus();
    let b = basis();
    let a = logical_z_op(&code, b.homology()[0].clone());
    let c = logical_z_op(&code, b.homology()[1].clone());
    assert!(!b.are_logically_equivalent(&a, &c).unwrap());
}

#[test]
fn test_equivalence_is_reflexive() {
    let code = torus();
    let b = basis();
    for gamma in b.homology() {
        let op = logical_z_op(&code, gamma.clone());
        assert!(b.are_logically_equivalent(&op, &op).unwrap());
    }
}

#[test]
fn test_an_x_type_logical_operator_is_detected_through_the_homology_side() {
    // X̄(γ̃) has its support in the X part, so it is the homology generators it must fail to
    // commute with. This exercises the other half of the criterion, which a predicate testing only
    // the Z part would pass vacuously.
    let code = torus();
    let b = basis();
    let gamma_tilde = b.cohomology()[0].clone();
    let x_op = LogicalPauli::new(gamma_tilde, zero_chain(&code)).unwrap();
    assert!(!b.is_logically_trivial(&x_op).unwrap());
}

#[test]
fn test_a_sphere_encodes_nothing_and_every_logical_operator_is_trivial() {
    // β₁(S²) = 0, so there is no logical space and the criterion has no generator to fail against.
    // The predicate must say so rather than dividing by an empty basis. The operator has to be a
    // logical one, though: a stabilizer, here a Z on a face boundary times an X on a vertex
    // coboundary, which is in the normalizer by construction. An arbitrary X on a stabilized
    // qubit is not, and is refused rather than misjudged (see the normalizer tests).
    let (sphere, _q, _f) = reference_spaces()
        .into_iter()
        .find(|(f, _, _)| f.name() == "sphere_2")
        .expect("the fixture set carries sphere_2");
    let b = LogicalBasis::<W>::from_complex(&sphere, 1).unwrap();
    assert_eq!(b.num_logical_qubits(), 0);
    let op = LogicalPauli::new(b.x_stabilizers()[0].clone(), b.stabilizers()[0].clone()).unwrap();
    assert!(b.is_logically_trivial(&op).unwrap());
}

#[test]
fn test_an_operator_outside_the_normalizer_is_refused_not_misjudged() {
    // A single-qubit X on an edge that some face boundary covers anticommutes with that
    // Z-stabilizer, so it leaves the code space. The commutation criterion alone would answer
    // `true`, since a weight-one X pairs to zero with a homology generator it misses; the
    // normalizer check answers with the obstruction instead.
    let code = torus();
    let b = basis();
    let n = code.num_cells(1);
    let covered = b.stabilizers()[0]
        .support()
        .next()
        .expect("a face boundary has edges");
    let x_on_edge = LogicalPauli::new(
        Chain::from_support(n, 1, &[covered]).unwrap(),
        Chain::zeros(n, 1),
    )
    .unwrap();
    let err = b.is_logically_trivial(&x_on_edge).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::NotInNormalizer { .. }),
        "expected NotInNormalizer, got {err:?}"
    );

    // And a Z on an edge some vertex coboundary covers anticommutes with an X-stabilizer.
    let covered = b.x_stabilizers()[0]
        .support()
        .next()
        .expect("a vertex coboundary has edges");
    let z_on_edge = LogicalPauli::new(
        Chain::zeros(n, 1),
        Chain::from_support(n, 1, &[covered]).unwrap(),
    )
    .unwrap();
    let err = b.is_logically_trivial(&z_on_edge).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::NotInNormalizer { .. }));
}

#[test]
fn test_the_x_generators_are_an_independent_basis_of_the_coboundaries() {
    // The X side of the stabilizer group. On the 3×3 torus `rank δ₀ = n₀ − β₀ = 9 − 1 = 8`, and
    // every X-generator must pair to zero with every Z-generator: that is `∂₁ ∘ ∂₂ = 0` read
    // through the pairing, `⟨∂₂ f, ∂₁ᵀ v⟩ = ⟨∂₁ ∂₂ f, v⟩`.
    let code = torus();
    let b = basis();
    let x = b.x_stabilizers();
    assert_eq!(x.len(), code.num_cells(0) - code.betti_number(0));

    let n = code.num_cells(1);
    let mut m = PackedGf2::<W>::zeros(n, x.len());
    for (c, g) in x.iter().enumerate() {
        for r in g.support() {
            m.set(r, c, Gf2::ONE).unwrap();
        }
    }
    assert_eq!(
        rank_gf2(&m).unwrap(),
        x.len(),
        "the X-generators are independent"
    );

    for t in x {
        for s in b.stabilizers() {
            assert_eq!(
                t.inner(s).unwrap(),
                Gf2::ZERO,
                "an X-generator met a Z-generator oddly"
            );
        }
    }
}

#[test]
fn test_a_register_width_mismatch_is_rejected() {
    let b = basis();
    let wrong = LogicalPauli::new(Chain::zeros(3, 1), Chain::zeros(3, 1)).unwrap();
    assert!(b.is_logically_trivial(&wrong).is_err());
    // And a Pauli whose two halves disagree cannot be built at all.
    assert!(LogicalPauli::<W>::new(Chain::zeros(4, 1), Chain::zeros(5, 1)).is_err());
}
