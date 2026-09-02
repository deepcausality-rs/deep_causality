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
    logical_cz, logical_hadamard, logical_s, logical_t,
};

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
