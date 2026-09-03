/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Clifford tableau, and the `H̄` check built on it.
//!
//! Three external anchors, none of them a reading of this workspace:
//!
//! - The single-qubit conjugation rules `HXH = Z`, `SXS† = Y`, `CNOT: X_c ↦ X_c X_t` and
//!   `CZ: X_a ↦ X_a Z_b` (Nielsen & Chuang §4.2 and §10.5.2), which fix every tableau row.
//! - Haruna arXiv:2511.15224 §3.2: `H̄(γ)` "behaves as a Hadamard gate on the code space" under
//!   conjugation of the logical `Z` and `X`, for `γ̃` dual to `γ` with `⟨γ, γ̃⟩ = 1`.
//! - Haruna Eq. (3.20): `S̄(γ)` maps `X̄(γ̃)` to `X̄(γ̃) · Z̄(γ)^{⟨γ, γ̃⟩}` up to phase, which is
//!   `S X S† = Y` lifted to the code.

use deep_causality_homology::utils_tests::reference_spaces;
use deep_causality_homology::{ChainComplex, Gf2Chain};
use deep_causality_num::Gf2;
use deep_causality_quantum::{
    DiagonalPhase, GateOp, LogicalBasis, LogicalPauli, QuantumErrorEnum, clifford_conjugate,
    logical_cz, logical_hadamard, logical_s, logical_t, symplectic_dual_basis,
};
use deep_causality_topology::LatticeComplex;

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

/// A Pauli on a small register from its X and Z supports.
fn pauli(n: usize, x: &[usize], z: &[usize]) -> LogicalPauli<W> {
    LogicalPauli::new(
        Chain::from_support(n, 1, x).unwrap(),
        Chain::from_support(n, 1, z).unwrap(),
    )
    .unwrap()
}

/// A cochain dual to `homology[which]` and orthogonal to the other generator, as an 𝔽₂
/// combination of the cohomology basis. The torus has two classes, so three combinations to try.
fn dual_basis_element(b: &LogicalBasis<W>, which: usize) -> Chain {
    let h = b.homology();
    let c = b.cohomology();
    let candidates = [c[0].clone(), c[1].clone(), c[0].add(&c[1]).unwrap()];
    let other = 1 - which;
    candidates
        .into_iter()
        .find(|gt| {
            h[which].inner(gt).unwrap() == Gf2::ONE && h[other].inner(gt).unwrap() == Gf2::ZERO
        })
        .expect("the intersection pairing on the torus is non-degenerate")
}

// ---------------------------------------------------------------------------
// The tableau rows, one gate at a time, against the textbook rules.
// ---------------------------------------------------------------------------

#[test]
fn test_hadamard_swaps_x_and_z() {
    let x = pauli(2, &[0], &[]);
    assert_eq!(
        clifford_conjugate(&x, &[GateOp::H(0)]).unwrap(),
        pauli(2, &[], &[0])
    );
    let z = pauli(2, &[], &[0]);
    assert_eq!(
        clifford_conjugate(&z, &[GateOp::H(0)]).unwrap(),
        pauli(2, &[0], &[])
    );
    // And leaves the other qubit alone.
    let x1 = pauli(2, &[1], &[]);
    assert_eq!(clifford_conjugate(&x1, &[GateOp::H(0)]).unwrap(), x1);
}

#[test]
fn test_s_maps_x_to_y_and_fixes_z() {
    let x = pauli(1, &[0], &[]);
    // Y = iXZ is (1, 1) in symplectic form.
    assert_eq!(
        clifford_conjugate(&x, &[GateOp::S(0)]).unwrap(),
        pauli(1, &[0], &[0])
    );
    assert_eq!(
        clifford_conjugate(&x, &[GateOp::Sdg(0)]).unwrap(),
        pauli(1, &[0], &[0])
    );
    let z = pauli(1, &[], &[0]);
    assert_eq!(clifford_conjugate(&z, &[GateOp::S(0)]).unwrap(), z);
}

#[test]
fn test_cnot_spreads_x_forward_and_z_backward() {
    let cnot = [GateOp::Cnot {
        control: 0,
        target: 1,
    }];
    assert_eq!(
        clifford_conjugate(&pauli(2, &[0], &[]), &cnot).unwrap(),
        pauli(2, &[0, 1], &[])
    );
    assert_eq!(
        clifford_conjugate(&pauli(2, &[], &[1]), &cnot).unwrap(),
        pauli(2, &[], &[0, 1])
    );
    // X on the target and Z on the control are fixed.
    assert_eq!(
        clifford_conjugate(&pauli(2, &[1], &[]), &cnot).unwrap(),
        pauli(2, &[1], &[])
    );
    assert_eq!(
        clifford_conjugate(&pauli(2, &[], &[0]), &cnot).unwrap(),
        pauli(2, &[], &[0])
    );
}

#[test]
fn test_cz_attaches_a_z_to_the_other_qubit_and_fixes_z() {
    let cz = [GateOp::Cz {
        control: 0,
        target: 1,
    }];
    assert_eq!(
        clifford_conjugate(&pauli(2, &[0], &[]), &cz).unwrap(),
        pauli(2, &[0], &[1])
    );
    assert_eq!(
        clifford_conjugate(&pauli(2, &[1], &[]), &cz).unwrap(),
        pauli(2, &[1], &[0])
    );
    assert_eq!(
        clifford_conjugate(&pauli(2, &[], &[0]), &cz).unwrap(),
        pauli(2, &[], &[0])
    );
    // A two-qubit Cmz is a CZ.
    let cmz = [GateOp::Cmz { qubits: vec![0, 1] }];
    assert_eq!(
        clifford_conjugate(&pauli(2, &[0], &[]), &cmz).unwrap(),
        pauli(2, &[0], &[1])
    );
}

#[test]
fn test_paulis_in_the_program_change_nothing() {
    let y = pauli(2, &[0], &[0]);
    let program = [GateOp::X(0), GateOp::Y(1), GateOp::Z(0)];
    assert_eq!(clifford_conjugate(&y, &program).unwrap(), y);
}

#[test]
fn test_conjugation_composes_in_application_order() {
    // H then S on X: H gives Z, S fixes Z. S then H on X: S gives Y = (1,1), H swaps to (1,1).
    let x = pauli(1, &[0], &[]);
    assert_eq!(
        clifford_conjugate(&x, &[GateOp::H(0), GateOp::S(0)]).unwrap(),
        pauli(1, &[], &[0])
    );
    assert_eq!(
        clifford_conjugate(&x, &[GateOp::S(0), GateOp::H(0)]).unwrap(),
        pauli(1, &[0], &[0])
    );
}

#[test]
fn test_a_non_clifford_program_is_refused_not_misjudged() {
    let x = pauli(3, &[0], &[]);
    for bad in [
        vec![GateOp::T(0)],
        vec![GateOp::H(0), GateOp::Tdg(0)],
        vec![GateOp::Csdg {
            control: 0,
            target: 1,
        }],
        vec![GateOp::Ccz {
            q0: 0,
            q1: 1,
            q2: 2,
        }],
        vec![GateOp::Cmz {
            qubits: vec![0, 1, 2],
        }],
    ] {
        let err = clifford_conjugate(&x, &bad).unwrap_err();
        assert!(
            matches!(err.0, QuantumErrorEnum::NonCliffordGate(_)),
            "expected NonCliffordGate for {bad:?}, got {err:?}"
        );
    }
    // A T̄ program is refused by the check, and names the first non-Clifford gate.
    let b = basis();
    let gamma = b.homology()[0].clone();
    let gamma_tilde = dual_basis_element(&b, 0);
    let t_program = logical_t(&gamma).unwrap();
    let err = b
        .check_clifford_action(&t_program, &gamma, &gamma_tilde)
        .unwrap_err();
    match err.0 {
        QuantumErrorEnum::NonCliffordGate(msg) => assert!(msg.contains("position 0"), "{msg}"),
        other => panic!("expected NonCliffordGate, got {other:?}"),
    }
}

#[test]
fn test_a_gate_beyond_the_register_is_rejected() {
    let x = pauli(2, &[0], &[]);
    assert!(clifford_conjugate(&x, &[GateOp::H(5)]).is_err());
}

// ---------------------------------------------------------------------------
// H̄ on the torus: the paper's own criterion, §3.2.
// ---------------------------------------------------------------------------

#[test]
fn test_the_logical_hadamard_swaps_the_logical_paulis() {
    // For each of the two logical qubits, with γ̃ dual to γ: Z̄(γ) ↦ X̄(γ̃) and X̄(γ̃) ↦ Z̄(γ).
    let b = basis();
    for which in 0..2 {
        let gamma = b.homology()[which].clone();
        let gamma_tilde = dual_basis_element(&b, which);
        let (program, _phase) = logical_hadamard::<W, f64>(&gamma, &gamma_tilde).unwrap();
        let report = b
            .check_clifford_action(&program, &gamma, &gamma_tilde)
            .unwrap();
        assert!(
            report.z_to_x,
            "logical qubit {which}: Z̄(γ) did not map to X̄(γ̃)"
        );
        assert!(
            report.x_to_z,
            "logical qubit {which}: X̄(γ̃) did not map to Z̄(γ)"
        );
        assert!(report.holds);
        assert_eq!(report.gates_applied, program.len());
        assert!(
            report.gates_applied > 0,
            "a vacuous program has swapped nothing"
        );
    }
}

#[test]
fn test_the_logical_hadamard_leaves_the_other_logical_qubit_alone() {
    // With γ̃ pairing to zero with the other generator, H̄ on qubit 0 is the identity on qubit 1:
    // both of its logical Paulis come back logically equivalent to themselves.
    let b = basis();
    let gamma = b.homology()[0].clone();
    let gamma_tilde = dual_basis_element(&b, 0);
    let (program, _) = logical_hadamard::<W, f64>(&gamma, &gamma_tilde).unwrap();

    let n = b.len();
    let zero = Chain::zeros(n, 1);
    let other_z = LogicalPauli::new(zero.clone(), b.homology()[1].clone()).unwrap();
    let other_x = LogicalPauli::new(dual_basis_element(&b, 1), zero).unwrap();
    for op in [other_z, other_x] {
        let image = clifford_conjugate(&op, &program).unwrap();
        assert!(
            b.are_logically_equivalent(&image, &op).unwrap(),
            "H̄ on qubit 0 moved a logical Pauli of qubit 1"
        );
    }
}

#[test]
fn test_the_hadamard_check_needs_a_dual_cochain() {
    // ⟨γ, γ̃⟩ = 0 makes the question ill-posed, and the derivation in the doc block shows the
    // program then does something else. The check refuses rather than reporting a failure.
    let b = basis();
    let gamma = b.homology()[0].clone();
    let not_dual = dual_basis_element(&b, 1);
    assert_eq!(gamma.inner(&not_dual).unwrap(), Gf2::ZERO);
    let (program, _) = logical_hadamard::<W, f64>(&gamma, &not_dual).unwrap();
    let err = b
        .check_clifford_action(&program, &gamma, &not_dual)
        .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::CalculationError(_)));
}

#[test]
fn test_a_program_that_does_nothing_fails_the_check() {
    // The empty program is the identity; the identity is not a Hadamard. This is the test that
    // a check answering `holds` unconditionally would fail.
    let b = basis();
    let gamma = b.homology()[0].clone();
    let gamma_tilde = dual_basis_element(&b, 0);
    let report = b.check_clifford_action(&[], &gamma, &gamma_tilde).unwrap();
    assert!(!report.holds);
    assert!(!report.z_to_x);
    assert!(!report.x_to_z);
    assert_eq!(report.gates_applied, 0);
}

// ---------------------------------------------------------------------------
// The diagonal Cliffords, decided by both stages.
// ---------------------------------------------------------------------------

#[test]
fn test_the_diagonal_cliffords_agree_with_the_diagonal_check() {
    // S̄ and CZ̄ are Clifford and diagonal, so both predicates reach them. The diagonal check
    // accepts each as class-invariant; the tableau shows S̄ fixing Z̄(γ) and sending X̄(γ̃) to
    // X̄(γ̃)·Z̄(γ) up to phase, which is Eq. (3.20) read on the symplectic side.
    let code = torus();
    let b = basis();
    let n = code.num_cells(1);
    let zero = Chain::zeros(n, 1);
    let gamma = b.homology()[0].clone();
    let gamma_tilde = dual_basis_element(&b, 0);

    // Diagonal side: S̄(γ) is class-invariant on the code space.
    let d2 = deep_causality_linear::csr_to_packed_gf2_mod2::<W>(&code.boundary_matrix(2));
    use deep_causality_linear::MatrixView;
    let boundaries: Vec<Chain> = (0..d2.cols())
        .map(|c| Chain::from_column(&d2, c, 1).unwrap())
        .collect();
    assert!(
        b.check_class_invariance(&DiagonalPhase::s(gamma.clone()), &boundaries)
            .unwrap()
            .holds
    );

    // Symplectic side: S̄(γ) fixes Z̄(γ) and attaches Z̄(γ) to X̄(γ̃).
    let s_program = logical_s(&gamma);
    let z_bar = LogicalPauli::new(zero.clone(), gamma.clone()).unwrap();
    let x_bar = LogicalPauli::new(gamma_tilde.clone(), zero.clone()).unwrap();
    let z_image = clifford_conjugate(&z_bar, &s_program).unwrap();
    assert!(b.are_logically_equivalent(&z_image, &z_bar).unwrap());
    let x_image = clifford_conjugate(&x_bar, &s_program).unwrap();
    let expected = x_bar.compose(&z_bar).unwrap();
    assert!(
        b.are_logically_equivalent(&x_image, &expected).unwrap(),
        "S̄(γ) X̄(γ̃) S̄(γ)† should be X̄(γ̃) Z̄(γ) up to phase"
    );
    assert!(!b.are_logically_equivalent(&x_image, &x_bar).unwrap());

    // CZ̄(γ₁, γ₂) attaches Z̄(γ₂) to X̄(γ̃₁) and Z̄(γ₁) to X̄(γ̃₂), and fixes both Z̄.
    let gamma2 = b.homology()[1].clone();
    let gamma2_tilde = dual_basis_element(&b, 1);
    let cz_program = logical_cz(&gamma, &gamma2).unwrap();
    let z2_bar = LogicalPauli::new(zero.clone(), gamma2.clone()).unwrap();
    let x2_bar = LogicalPauli::new(gamma2_tilde, zero).unwrap();
    for z in [&z_bar, &z2_bar] {
        let image = clifford_conjugate(z, &cz_program).unwrap();
        assert!(b.are_logically_equivalent(&image, z).unwrap());
    }
    let image = clifford_conjugate(&x_bar, &cz_program).unwrap();
    assert!(
        b.are_logically_equivalent(&image, &x_bar.compose(&z2_bar).unwrap())
            .unwrap()
    );
    let image = clifford_conjugate(&x2_bar, &cz_program).unwrap();
    assert!(
        b.are_logically_equivalent(&image, &x2_bar.compose(&z_bar).unwrap())
            .unwrap()
    );
}

// ---------------------------------------------------------------------------
// H̄ on one qubit of two: the symplectic dual basis, and what the pair check cannot see.
// ---------------------------------------------------------------------------

/// The `[[32, 2]]` toric code the geometric example runs: the square torus with `L = 4`.
fn square_basis() -> LogicalBasis<W> {
    LogicalBasis::from_complex(&LatticeComplex::<2, f64>::square_torus(4), 1).unwrap()
}

/// The pairing matrix `⟨γ_i, d_j⟩` of a basis's homology generators against a list of cochains.
fn pairing_matrix(b: &LogicalBasis<W>, duals: &[Chain]) -> Vec<Vec<Gf2>> {
    b.homology()
        .iter()
        .map(|g| duals.iter().map(|d| g.inner(d).unwrap()).collect())
        .collect()
}

fn identity_2() -> Vec<Vec<Gf2>> {
    vec![vec![Gf2::ONE, Gf2::ZERO], vec![Gf2::ZERO, Gf2::ONE]]
}

#[test]
fn test_the_symplectic_dual_basis_pairs_as_the_identity() {
    // On both two-qubit codes: ⟨γ_i, γ̃_j⟩ = δ_ij. Each dual is a sum of cohomology generators, so
    // it is a cocycle and X̄(γ̃_j) is a logical operator: in the normalizer, and not trivial.
    for b in [basis(), square_basis()] {
        assert_eq!(b.num_logical_qubits(), 2);
        let duals = symplectic_dual_basis(b.homology(), b.cohomology()).unwrap();
        assert_eq!(duals.len(), 2);
        assert_eq!(pairing_matrix(&b, &duals), identity_2());
        let zero = Chain::zeros(b.len(), 1);
        for d in &duals {
            let x_bar = LogicalPauli::new(d.clone(), zero.clone()).unwrap();
            assert!(
                !b.is_logically_trivial(&x_bar).unwrap(),
                "a dual is a logical X, in the normalizer and not a stabilizer"
            );
        }
    }
}

#[test]
fn test_the_dual_basis_does_not_depend_on_the_cohomology_basis_chosen() {
    // Any invertible change of the cohomology basis spans the same cocycles, and the dual basis
    // is the unique one in that span pairing as the identity, so the same chains come back. The
    // swapped basis is the case where the elimination has to pivot.
    let b = square_basis();
    let c = b.cohomology();
    let reference = symplectic_dual_basis(b.homology(), c).unwrap();
    let mixed = c[0].add(&c[1]).unwrap();
    for skewed in [
        vec![c[1].clone(), c[0].clone()],
        vec![mixed.clone(), c[1].clone()],
        vec![c[0].clone(), mixed.clone()],
        vec![mixed.clone(), c[0].clone()],
    ] {
        let duals = symplectic_dual_basis(b.homology(), &skewed).unwrap();
        assert_eq!(pairing_matrix(&b, &duals), identity_2());
        assert_eq!(duals, reference, "the duals depend on H¹, not on its basis");
    }
}

#[test]
fn test_the_inverse_is_read_down_its_columns_on_a_hand_checked_pairing() {
    // γ = (e₀, e₁), c = (e₀ + e₁, e₁): M = [[1, 0], [1, 1]] and M⁻¹ = M over 𝔽₂. Reading M⁻¹ down
    // its columns gives γ̃₀ = c₀ + c₁ = e₀ and γ̃₁ = c₁ = e₁; reading it across rows would give
    // γ̃₁ = c₀ + c₁ as well, which pairs to one with γ₀.
    let e = |i: usize| Chain::from_support(3, 1, &[i]).unwrap();
    let c0 = e(0).add(&e(1)).unwrap();
    let duals = symplectic_dual_basis(&[e(0), e(1)], &[c0, e(1)]).unwrap();
    assert_eq!(duals, vec![e(0), e(1)]);
    // The swapped pairing M = [[0, 1], [1, 0]] needs a pivot exchange and inverts to itself.
    let duals = symplectic_dual_basis(&[e(0), e(1)], &[e(1), e(0)]).unwrap();
    assert_eq!(duals, vec![e(0), e(1)]);
}

#[test]
fn test_a_singular_pairing_is_an_error_rather_than_a_panic() {
    // Two independent generators against cochains that both read e₀: M = [[1, 1], [0, 0]].
    let e = |i: usize| Chain::from_support(3, 1, &[i]).unwrap();
    let err = symplectic_dual_basis(&[e(0), e(1)], &[e(0), e(0)]).unwrap_err();
    match err.0 {
        QuantumErrorEnum::CalculationError(msg) => {
            assert!(msg.contains("singular") && msg.contains("rank 1"), "{msg}")
        }
        other => panic!("expected CalculationError, got {other:?}"),
    }
    // A generator pairing to zero with everything is the same failure at rank 0.
    let err = symplectic_dual_basis(&[e(2)], &[e(0)]).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::CalculationError(_)));
    // A pairing that is not square is a shape error, and a register mismatch is one too.
    assert!(matches!(
        symplectic_dual_basis(&[e(0), e(1)], &[e(0)]).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    let wide = Chain::from_support(4, 1, &[0]).unwrap();
    assert!(matches!(
        symplectic_dual_basis(&[e(0)], &[wide]).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    // No logical qubits: no duals, and no error.
    assert!(symplectic_dual_basis::<W>(&[], &[]).unwrap().is_empty());
}

#[test]
fn test_the_logical_hadamard_on_each_qubit_fixes_the_other() {
    // On the square torus, H̄ built on (γ_i, γ̃_i) from the dual basis swaps its own pair and
    // leaves both logical generators of the other qubit where they were.
    let b = square_basis();
    let duals = symplectic_dual_basis(b.homology(), b.cohomology()).unwrap();
    for index in 0..2 {
        let (program, _) = logical_hadamard::<W, f64>(&b.homology()[index], &duals[index]).unwrap();
        let report = b
            .check_clifford_action_on_qubit(&program, index, &duals)
            .unwrap();
        assert!(
            report.z_to_x && report.x_to_z,
            "qubit {index}: the pair was not swapped"
        );
        assert_eq!(
            report.others_examined, 2,
            "Z̄ and X̄ of the other qubit were both examined"
        );
        assert!(
            report.others_fixed,
            "qubit {index}: H̄ moved the other qubit"
        );
        assert!(report.holds);
        assert_eq!(report.gates_applied, program.len());
    }
}

#[test]
fn test_a_dual_meeting_the_other_generator_passes_the_pair_check_and_is_refused_on_the_basis() {
    // γ̃₀ + γ̃₁ pairs to one with γ₀ and with γ₁. Haruna's derivation needs only ⟨γ, γ̃⟩ = 1, so the
    // pair check passes; the program it builds moves Z̄(γ₁), which the pair check never reads.
    let b = square_basis();
    let duals = symplectic_dual_basis(b.homology(), b.cohomology()).unwrap();
    let gamma = &b.homology()[0];
    let other = &b.homology()[1];
    let bad = duals[0].add(&duals[1]).unwrap();
    assert_eq!(gamma.inner(&bad).unwrap(), Gf2::ONE);
    assert_eq!(other.inner(&bad).unwrap(), Gf2::ONE);
    let (program, _) = logical_hadamard::<W, f64>(gamma, &bad).unwrap();

    // The pair check accepts it, having examined no other qubit.
    let pair = b.check_clifford_action(&program, gamma, &bad).unwrap();
    assert!(pair.holds && pair.z_to_x && pair.x_to_z);
    assert_eq!(pair.others_examined, 0);
    assert!(pair.others_fixed, "vacuously: nothing else was examined");

    // The program is a gate on two qubits: Z̄(γ₁) comes back as X̄(γ̃) Z̄(γ₀) Z̄(γ₁), up to phase.
    let zero = Chain::zeros(b.len(), 1);
    let z1 = LogicalPauli::new(zero.clone(), other.clone()).unwrap();
    let image = clifford_conjugate(&z1, &program).unwrap();
    assert!(!b.are_logically_equivalent(&image, &z1).unwrap());
    let expected = LogicalPauli::new(bad.clone(), gamma.add(other).unwrap()).unwrap();
    assert!(b.are_logically_equivalent(&image, &expected).unwrap());

    // The basis-aware check refuses the dual before pushing anything through.
    let mut bad_duals = duals.clone();
    bad_duals[0] = bad;
    let err = b
        .check_clifford_action_on_qubit(&program, 0, &bad_duals)
        .unwrap_err();
    match err.0 {
        QuantumErrorEnum::CalculationError(msg) => {
            assert!(msg.contains("⟨γ_1, γ̃_0⟩ = 1"), "{msg}")
        }
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_a_hadamard_on_both_qubits_is_not_a_hadamard_on_one() {
    // H̄₀ then H̄₁, each on its own dual: the pair on qubit 0 is swapped, so the pair check passes,
    // and qubit 1 is swapped too, which only the basis-aware check sees.
    let b = square_basis();
    let duals = symplectic_dual_basis(b.homology(), b.cohomology()).unwrap();
    let (mut program, _) = logical_hadamard::<W, f64>(&b.homology()[0], &duals[0]).unwrap();
    program.extend(
        logical_hadamard::<W, f64>(&b.homology()[1], &duals[1])
            .unwrap()
            .0,
    );
    let pair = b
        .check_clifford_action(&program, &b.homology()[0], &duals[0])
        .unwrap();
    assert!(pair.holds, "the pair check cannot see the second qubit");

    let report = b
        .check_clifford_action_on_qubit(&program, 0, &duals)
        .unwrap();
    assert!(report.z_to_x && report.x_to_z);
    assert!(!report.others_fixed);
    assert!(!report.holds);
    assert_eq!(
        report.others_examined, 1,
        "stopped at Z̄(γ₁), the first generator that moved"
    );
}

#[test]
fn test_the_basis_aware_check_refuses_a_wrong_index_or_dual_count() {
    let b = square_basis();
    let duals = symplectic_dual_basis(b.homology(), b.cohomology()).unwrap();
    let (program, _) = logical_hadamard::<W, f64>(&b.homology()[0], &duals[0]).unwrap();
    assert!(matches!(
        b.check_clifford_action_on_qubit(&program, 2, &duals)
            .unwrap_err()
            .0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    assert!(matches!(
        b.check_clifford_action_on_qubit(&program, 0, &duals[..1])
            .unwrap_err()
            .0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}
