/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The generation regression and the two-path bridge for the code abstraction.
//!
//! On the exact path every Table 1 gate holds on `[[18,2,3]]` and `[[32,2,4]]`, and the verdict
//! agrees gate by gate with v1's `check_class_invariance` (diagonal gates) and
//! `check_clifford_action_on_qubit` (`H̄`). The one constructed failure, `S̄` with its `CZ` pairs
//! omitted, is rejected by both: the program's phase `n/4` is `1/2` at overlap two where a parity
//! function gives `0`. On `[[8,2,2]]` the numeric path agrees with the exact one on every gate.

use deep_causality_homology::ChainComplex;
use deep_causality_quantum::utils_tests::four_two_two;
use deep_causality_quantum::{
    AlignmentSide, CheckVerdict, CodeAbstraction, DiagonalPhase, GateOp, LogicalGate, NumericCaps,
    QuantumErrorEnum, Query, SemanticsPath,
};
use deep_causality_topology::LatticeComplex;

type W = u64;

fn torus(l: usize) -> LatticeComplex<2, f64> {
    LatticeComplex::<2, f64>::square_torus(l)
}

#[test]
fn test_table_one_holds_exactly_on_both_torus_fixtures() {
    for l in [3usize, 4] {
        let complex = torus(l);
        let gates = CodeAbstraction::<W>::table_one(2);
        assert_eq!(gates.len(), 11);
        let ca = CodeAbstraction::<W>::new(&complex, gates).unwrap();
        let exact = ca.check_naturality_exact::<f64>().unwrap();
        assert_eq!(exact.path, SemanticsPath::Exact);
        assert_eq!(exact.report.examined(), 11);
        assert_eq!(
            exact.report.verdict(),
            CheckVerdict::Accepted,
            "{:?}",
            exact.gates
        );
        assert!(exact.gates.iter().all(|g| g.holds && g.witness.is_none()));
        assert_eq!(ca.code().n(), complex.num_cells(1));
        assert_eq!(ca.duals().len(), 2);
    }
}

#[test]
fn test_regression_agrees_with_the_v1_predicates_on_every_gate() {
    for l in [3usize, 4] {
        let complex = torus(l);
        let ca = CodeAbstraction::<W>::new(&complex, CodeAbstraction::<W>::table_one(2)).unwrap();
        let exact = ca.check_naturality_exact::<f64>().unwrap();
        for verdict in &exact.gates {
            match &verdict.gate {
                LogicalGate::Z(i) | LogicalGate::S(i) | LogicalGate::T(i) => {
                    let gamma = ca.basis().homology()[*i].clone();
                    let phase = match verdict.gate {
                        LogicalGate::Z(_) => DiagonalPhase::z(gamma),
                        LogicalGate::S(_) => DiagonalPhase::s(gamma),
                        _ => DiagonalPhase::t(gamma),
                    };
                    let v1 = ca
                        .basis()
                        .check_class_invariance(&phase, ca.code().z_generators())
                        .unwrap();
                    assert_eq!(v1.holds, verdict.holds, "{}", verdict.gate.name());
                }
                LogicalGate::H(i) => {
                    let program = ca.program::<f64>(&verdict.gate).unwrap();
                    let v1 = ca
                        .basis()
                        .check_clifford_action_on_qubit(&program, *i, ca.duals())
                        .unwrap();
                    assert_eq!(v1.holds, verdict.holds);
                }
                LogicalGate::X(_) | LogicalGate::Cz(_, _) => assert!(verdict.holds),
            }
        }
    }
}

#[test]
fn test_omitting_the_cz_pairs_of_s_bar_fails_both_generations() {
    let complex = torus(3);
    let ca = CodeAbstraction::<W>::new(&complex, vec![LogicalGate::S(0)]).unwrap();
    let gamma = ca.basis().homology()[0].clone();
    let without_pairs: Vec<GateOp> = gamma.support().map(GateOp::S).collect();
    let verdict = ca
        .check_gate_program(&LogicalGate::S(0), &without_pairs)
        .unwrap();
    assert!(!verdict.holds);
    assert!(
        verdict
            .witness
            .as_deref()
            .unwrap()
            .contains("not a function of the block parities"),
        "{:?}",
        verdict.witness
    );
    // v1's predicate on the polynomial that program implements, n/4, rejects as well.
    let n_over_4 = DiagonalPhase::new(gamma, vec![0, 1], 2).unwrap();
    let v1 = ca
        .basis()
        .check_class_invariance(&n_over_4, ca.code().z_generators())
        .unwrap();
    assert!(!v1.holds);
    // The complete program holds.
    let complete = ca.program::<f64>(&LogicalGate::S(0)).unwrap();
    assert!(
        ca.check_gate_program(&LogicalGate::S(0), &complete)
            .unwrap()
            .holds
    );
    // The S̄ program handed in as H̄ fails the tableau side: Z̄(γ) is fixed, not sent to X̄(γ̃).
    let ca_h = CodeAbstraction::<W>::new(&complex, vec![LogicalGate::H(0)]).unwrap();
    let verdict = ca_h
        .check_gate_program(&LogicalGate::H(0), &complete)
        .unwrap();
    assert!(!verdict.holds);
    assert!(
        verdict.witness.as_deref().unwrap().contains("Z̄ ↦ X̄: false"),
        "{:?}",
        verdict.witness
    );
}

#[test]
fn test_numeric_path_agrees_with_the_exact_path_on_the_small_torus() {
    let complex = torus(2);
    let gates = vec![
        LogicalGate::Z(0),
        LogicalGate::X(1),
        LogicalGate::S(0),
        LogicalGate::T(1),
        LogicalGate::H(0),
        LogicalGate::Cz(0, 1),
    ];
    let ca = CodeAbstraction::<W>::new(&complex, gates.clone()).unwrap();
    let exact = ca.check_naturality_exact::<f64>().unwrap();
    assert_eq!(exact.report.verdict(), CheckVerdict::Accepted);
    let caps = NumericCaps::default();
    let numeric = ca.numeric_abstractions::<f64>().unwrap();
    assert_eq!(numeric.len(), gates.len());
    for ((gate, abstraction), verdict) in numeric.iter().zip(&exact.gates) {
        assert_eq!(gate, &verdict.gate);
        // The `Io` square only: opening the program doubles the register to ten qubits, which the
        // default cap refuses; that refusal is checked below.
        let r = abstraction
            .check_naturality_on(&[Query::Io], &caps)
            .unwrap();
        assert_eq!(r.report.examined(), 1);
        assert_eq!(r.path, SemanticsPath::Numeric);
        assert_eq!(
            r.report.verdict(),
            CheckVerdict::Accepted,
            "{}: residual {}",
            gate.name(),
            r.worst_residual()
        );
        assert!(
            r.worst_residual() < 1e-8,
            "{}: {}",
            gate.name(),
            r.worst_residual()
        );
        assert_eq!(r.report.accepted(), verdict.holds);
    }
}

#[test]
fn test_construction_errors() {
    let complex = torus(3);
    let err = CodeAbstraction::<W>::new(&complex, vec![LogicalGate::T(5)]).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("logical qubit 5"))
    );
    let err = CodeAbstraction::<W>::new(&complex, vec![LogicalGate::Cz(1, 1)]).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    assert_eq!(LogicalGate::Cz(0, 1).qubits(), vec![0, 1]);
    assert_eq!(LogicalGate::H(1).name(), "H̄(1)");
    let ca = CodeAbstraction::<W>::new(&complex, vec![]).unwrap();
    let exact = ca.check_naturality_exact::<f64>().unwrap();
    assert_eq!(exact.report.verdict(), CheckVerdict::Vacuous);
    assert!(ca.gates().is_empty());
}

/// Opening the physical program on `[[8,2,2]]` makes a ten-qubit register (two logical inputs and
/// eight fresh physical ones) and the working storage `2^16 · 2^10 = 2^26` exceeds the default cap;
/// the refusal names both counts and no matrix is formed.
#[test]
fn test_opening_the_eight_qubit_code_is_refused_by_the_cap() {
    let ca = CodeAbstraction::<W>::new(&torus(2), vec![LogicalGate::S(0)]).unwrap();
    let numeric = ca.numeric_abstractions::<f64>().unwrap();
    let (_, abstraction) = &numeric[0];
    assert_eq!(abstraction.signature().len(), 2);
    let err = abstraction
        .check_naturality(&NumericCaps::default())
        .unwrap_err();
    match err.0 {
        QuantumErrorEnum::NaturalityDimensionExceeded { n, k, entries, cap } => {
            assert_eq!((n, k), (10, 8));
            assert_eq!(entries, 1u64 << 26);
            assert_eq!(cap, 1u64 << 24);
        }
        other => panic!("{other:?}"),
    }
    let unknown = abstraction
        .check_naturality_on(&[Query::Observe(vec![0])], &NumericCaps::default())
        .unwrap_err();
    assert!(
        matches!(unknown.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("not in the signature"))
    );
}

/// On the `[[4,2,2]]` code both squares, `Io` and the opening of the logical gate against the
/// opening of the physical program, commute for the Pauli gates and `CZ̄`: the opened square is
/// `Tr_k ⊗ τ` on both sides.
#[test]
fn test_open_square_commutes_on_the_four_two_two_code() {
    let complex = four_two_two();
    let gates = vec![LogicalGate::Z(0), LogicalGate::X(1), LogicalGate::Cz(0, 1)];
    let ca = CodeAbstraction::<W>::new(&complex, gates.clone()).unwrap();
    let caps = NumericCaps::default();
    let numeric = ca.numeric_abstractions::<f64>().unwrap();
    assert_eq!(numeric.len(), gates.len());
    for (gate, abstraction) in &numeric {
        let r = abstraction.check_naturality(&caps).unwrap();
        assert_eq!(r.report.examined(), 2, "{}", gate.name());
        assert_eq!(
            r.report.verdict(),
            CheckVerdict::Accepted,
            "{}: {}",
            gate.name(),
            r.worst_residual()
        );
        assert!(
            r.worst_residual() < 1e-8,
            "{}: {}",
            gate.name(),
            r.worst_residual()
        );
        // The opened square is wider than the Io square: 2^(2+4) inputs against 2^2.
        let (left, _) = abstraction.square(&Query::Open(vec![0]), &caps).unwrap();
        assert_eq!(left.d_in(), 64);
        assert_eq!(left.d_out(), 4);
    }
}

/// The code alignment in the shape of Example 58: the input side aligns the two logical wires by
/// the identity, the output side aligns them with all eight physical qubits through the recovery.
#[test]
fn test_code_alignment_lists_all_physical_qubits_on_the_output_side() {
    let ca = CodeAbstraction::<W>::new(&torus(2), vec![LogicalGate::Z(0)]).unwrap();
    let numeric = ca.numeric_abstractions::<f64>().unwrap();
    let alignment = numeric[0].1.alignment();
    assert_eq!(alignment.entries().len(), 2);
    let input = &alignment.entries()[0];
    assert_eq!(input.side(), AlignmentSide::Input);
    assert_eq!(input.high(), &[0, 1]);
    assert_eq!(input.low(), &[0, 1]);
    assert_eq!((input.tau().d_in(), input.tau().d_out()), (4, 4));
    let output = &alignment.entries()[1];
    assert_eq!(output.side(), AlignmentSide::Output);
    assert_eq!(output.high(), &[0, 1]);
    assert_eq!(output.low(), &(0..8).collect::<Vec<_>>());
    assert_eq!((output.tau().d_in(), output.tau().d_out()), (256, 4));
    assert_eq!(
        (output.section().d_in(), output.section().d_out()),
        (4, 256)
    );
    let low = numeric[0].1.low();
    assert_eq!(low.inputs(), &[0, 1]);
    assert_eq!(low.outputs(), &(0..8).collect::<Vec<_>>());
    assert_eq!(low.boxes()[0].kind(), "kraus");
    assert_eq!(low.boxes()[1].kind(), "unitary");
}

/// A gate naming a logical qubit the code does not have is refused by every method that takes a
/// gate, as the constructor refuses it, rather than indexing past the basis.
#[test]
fn test_a_gate_outside_the_code_is_refused_by_every_gate_method() {
    let complex = torus(3);
    let ca = CodeAbstraction::<W>::new(&complex, vec![LogicalGate::Z(0)]).unwrap();
    let faults = deep_causality_quantum::FaultSet::pauli_weight(
        &[0, 1],
        Some(0),
        1,
        deep_causality_quantum::FAULT_SET_CAP,
    )
    .unwrap();
    for gate in [
        LogicalGate::Z(5),
        LogicalGate::X(2),
        LogicalGate::S(7),
        LogicalGate::T(2),
        LogicalGate::H(3),
        LogicalGate::Cz(0, 4),
        LogicalGate::Cz(1, 1),
    ] {
        let program = ca.program::<f64>(&gate).unwrap_err();
        assert!(
            matches!(program.0, QuantumErrorEnum::DimensionMismatch(_)),
            "{}: {program:?}",
            gate.name()
        );
        let checked = ca.check_gate_program(&gate, &[]).unwrap_err();
        assert!(
            matches!(checked.0, QuantumErrorEnum::DimensionMismatch(_)),
            "{}: {checked:?}",
            gate.name()
        );
        let exact = ca.exact_program::<f64>(&gate).unwrap_err();
        assert!(
            matches!(exact.0, QuantumErrorEnum::DimensionMismatch(_)),
            "{}: {exact:?}",
            gate.name()
        );
        let tolerance = ca.check_fault_tolerance::<f64>(&gate, &faults).unwrap_err();
        assert!(
            matches!(tolerance.0, QuantumErrorEnum::DimensionMismatch(_)),
            "{}: {tolerance:?}",
            gate.name()
        );
    }
    // A gate inside the code but outside the signature is still answered.
    assert!(ca.program::<f64>(&LogicalGate::X(1)).is_ok());
}

/// `X̄`'s verdict is read off the supplied program: the program must be a Pauli program whose
/// Pauli anticommutes with `Z̄(γᵢ)` alone and commutes with every other logical operator, up to
/// stabilizers. The emitted program of the other logical qubit, `Z̄(γ₀)` itself, a Pauli carrying
/// an extra `Z̄(γ₁)`, and a Hadamard all fail; the emitted program with an X-stabilizer appended
/// still holds.
#[test]
fn test_x_bar_verdict_is_read_off_the_supplied_program() {
    let complex = torus(3);
    let ca =
        CodeAbstraction::<W>::new(&complex, vec![LogicalGate::X(0), LogicalGate::X(1)]).unwrap();
    let x0 = ca.program::<f64>(&LogicalGate::X(0)).unwrap();
    let x1 = ca.program::<f64>(&LogicalGate::X(1)).unwrap();
    assert!(
        ca.check_gate_program(&LogicalGate::X(0), &x0)
            .unwrap()
            .holds
    );
    assert!(
        ca.check_gate_program(&LogicalGate::X(1), &x1)
            .unwrap()
            .holds
    );

    let other = ca.check_gate_program(&LogicalGate::X(0), &x1).unwrap();
    assert!(!other.holds, "X̄(γ̃₁) handed in as X̄(γ̃₀)");
    assert!(
        other.witness.as_deref().unwrap().contains("⟨γ_0, x⟩ = 0"),
        "{:?}",
        other.witness
    );

    let z0 = ca.program::<f64>(&LogicalGate::Z(0)).unwrap();
    let z_as_x = ca.check_gate_program(&LogicalGate::X(0), &z0).unwrap();
    assert!(!z_as_x.holds, "Z̄(γ₀) handed in as X̄(γ̃₀)");

    let mut x0_z1 = x0.clone();
    x0_z1.extend(ca.program::<f64>(&LogicalGate::Z(1)).unwrap());
    let carrying = ca.check_gate_program(&LogicalGate::X(0), &x0_z1).unwrap();
    assert!(!carrying.holds, "X̄(γ̃₀) Z̄(γ₁) carries a second logical");
    assert!(
        carrying
            .witness
            .as_deref()
            .unwrap()
            .contains("⟨γ̃_1, z⟩ = 1"),
        "{:?}",
        carrying.witness
    );

    let hadamard = ca
        .check_gate_program(&LogicalGate::X(0), &[GateOp::H(0)])
        .unwrap();
    assert!(!hadamard.holds);
    assert!(
        hadamard.witness.as_deref().unwrap().contains("not a Pauli"),
        "{:?}",
        hadamard.witness
    );

    let outside = ca
        .check_gate_program(&LogicalGate::X(0), &[GateOp::X(99)])
        .unwrap();
    assert!(!outside.holds);
    assert!(outside.witness.is_some());

    let mut with_stabilizer = x0.clone();
    with_stabilizer.extend(ca.code().x_generators()[0].support().map(GateOp::X));
    let still = ca
        .check_gate_program(&LogicalGate::X(0), &with_stabilizer)
        .unwrap();
    assert!(still.holds, "{:?}", still.witness);
    assert_eq!(still.program_len, with_stabilizer.len());
}
