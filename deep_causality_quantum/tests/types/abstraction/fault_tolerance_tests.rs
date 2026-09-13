/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Fault tolerance on both paths. The exact expectations come from Haruna's Eq. (3.63) as derived
//! in `notes/open-questions-resolved.md` §3: a Pauli fault `X^a Z^b` through `O_k(γ)` leaves the
//! remainder `exp(iπ/2^{k−1}) · O_{k−1}(γ)†` when `⟨a, γ⟩ = 1` and the identity otherwise, so `Z̄`
//! and `X̄` tolerate every weight-one fault, `S̄` leaves `i · Z̄(γ)` (table `[1/4, 3/4]`), `T̄` leaves
//! `exp(±iπ/4 Z̄(γ))` (table `[1/8, 7/8]`, two Pauli terms at every weight), `CZ̄(γ₁, γ₂)` leaves
//! `Z̄(γ₂)` under a fault on `γ₁`, and `H̄` spreads a fault to a Pauli of larger weight through its
//! tableau. The numeric path is checked on a two-wire circuit whose `τ` traces the second wire.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Abstraction, Axis, Channel, CheckVerdict, CircuitBox, CircuitModel, CodeAbstraction,
    FAULT_SET_CAP, Fault, FaultOrigin, FaultSet, GateOp, LogicalGate, NumericCaps, PauliKind,
    QcMorphism, QuantumErrorEnum, Query, QuerySignature, SemanticsPath, TypeAlignment, WireType,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::LatticeComplex;
use std::collections::BTreeSet;

type W = u64;
type C = Complex<f64>;

fn torus(l: usize) -> LatticeComplex<2, f64> {
    LatticeComplex::<2, f64>::square_torus(l)
}

fn weight_one(n: usize) -> FaultSet {
    FaultSet::pauli_weight(&(0..n).collect::<Vec<_>>(), Some(0), 1, FAULT_SET_CAP).unwrap()
}

fn support(ca: &CodeAbstraction<W>, i: usize) -> BTreeSet<usize> {
    ca.basis().homology()[i].support().collect()
}

fn on_support_with_x(f: &Fault, gamma: &BTreeSet<usize>) -> bool {
    f.errors()
        .iter()
        .any(|(w, p)| gamma.contains(w) && p.has_x())
}

#[test]
fn test_z_bar_tolerates_every_weight_one_fault_on_the_wide_torus() {
    let ca =
        CodeAbstraction::<W>::new(&torus(3), vec![LogicalGate::Z(0), LogicalGate::X(1)]).unwrap();
    let n = ca.basis().len();
    assert_eq!(n, 18);
    let faults = weight_one(n);
    for gate in ca.gates() {
        let r = ca.check_fault_tolerance::<f64>(gate, &faults).unwrap();
        assert_eq!(r.report.examined(), 54, "{}", gate.name());
        assert_eq!(r.count, 54);
        assert_eq!(r.origin, FaultOrigin::PauliWeight(1));
        assert!(r.holds(), "{}", gate.name());
        assert_eq!(r.tolerated(), 54);
        assert_eq!(r.paths(), (54, 0));
        assert!(r.records.iter().all(|rec| {
            rec.path == SemanticsPath::Exact
                && rec.remainder_terms == 1
                && rec.propagated_weight == Some(1)
                && rec.witness.is_none()
                && rec.residual == 0.0
        }));
        let shown = format!("{r}");
        assert!(
            shown.contains("pauli_weight(1)") && shown.contains("54 faults"),
            "{shown}"
        );
        assert!(shown.contains("54 tolerated, 0 not"), "{shown}");
        assert!(
            !shown.contains("threshold") && !shown.contains("distance"),
            "{shown}"
        );
    }
}

#[test]
fn test_t_bar_rejects_x_and_y_on_its_support_with_a_two_term_remainder_at_both_weights() {
    for (l, w) in [(3usize, 3usize), (4, 4)] {
        let ca = CodeAbstraction::<W>::new(&torus(l), vec![LogicalGate::T(0)]).unwrap();
        let n = ca.basis().len();
        let gamma = support(&ca, 0);
        assert_eq!(
            gamma.len(),
            w,
            "the representative's weight on the {l} × {l} torus"
        );
        let r = ca
            .check_fault_tolerance::<f64>(&LogicalGate::T(0), &weight_one(n))
            .unwrap();
        assert_eq!(r.report.examined(), 3 * n);
        assert!(!r.holds());
        let rejected: Vec<_> = r.records.iter().filter(|rec| !rec.tolerated).collect();
        assert_eq!(rejected.len(), 2 * w, "X and Y on each qubit of γ");
        for rec in &r.records {
            let expect_reject = on_support_with_x(&rec.fault, &gamma);
            assert_eq!(!rec.tolerated, expect_reject, "{}", rec.fault);
            assert_eq!(rec.path, SemanticsPath::Exact);
            assert_eq!(
                rec.propagated_weight,
                Some(1),
                "the diagonal path leaves the Pauli"
            );
            if expect_reject {
                assert_eq!(rec.remainder_terms, 2, "{}", rec.fault);
                let witness = rec.witness.as_deref().unwrap();
                assert!(witness.contains("[1/8, 7/8]"), "{witness}");
                assert!(witness.contains("0b1"), "{witness}");
                assert_eq!(rec.residual, 1.0);
            } else {
                assert_eq!(rec.remainder_terms, 1);
                assert!(rec.witness.is_none());
            }
        }
        assert_eq!(r.worst().map(|rec| rec.residual), Some(1.0));
    }
}

#[test]
fn test_s_bar_and_cz_bar_leave_the_logical_z_of_the_other_block() {
    let ca = CodeAbstraction::<W>::new(&torus(3), vec![LogicalGate::S(0), LogicalGate::Cz(0, 1)])
        .unwrap();
    let n = ca.basis().len();
    let faults = weight_one(n);
    let g0 = support(&ca, 0);
    let g1 = support(&ca, 1);
    assert!(g0.is_disjoint(&g1), "the two cycles share no edge");

    let s = ca
        .check_fault_tolerance::<f64>(&LogicalGate::S(0), &faults)
        .unwrap();
    assert_eq!(
        s.records.iter().filter(|r| !r.tolerated).count(),
        2 * g0.len()
    );
    for rec in s.records.iter().filter(|r| !r.tolerated) {
        assert!(on_support_with_x(&rec.fault, &g0));
        let witness = rec.witness.as_deref().unwrap();
        // exp(iπ/2) · Z̄(γ)†: the Z̄ table [0, 1/2] shifted by the global phase 1/4.
        assert!(witness.contains("[1/4, 3/4]"), "i · Z̄(γ): {witness}");
        assert_eq!(rec.remainder_terms, 1, "Z̄(γ) is one Pauli term");
    }

    let cz = ca
        .check_fault_tolerance::<f64>(&LogicalGate::Cz(0, 1), &faults)
        .unwrap();
    assert_eq!(
        cz.records.iter().filter(|r| !r.tolerated).count(),
        2 * (g0.len() + g1.len())
    );
    for rec in &cz.records {
        let on0 = on_support_with_x(&rec.fault, &g0);
        let on1 = on_support_with_x(&rec.fault, &g1);
        assert_eq!(!rec.tolerated, on0 || on1, "{}", rec.fault);
        if on0 {
            // Flipping block 0's parity leaves Z̄(γ₁): the table depends on parity bit 1 only.
            let witness = rec.witness.as_deref().unwrap();
            assert!(
                witness.contains("0b1 ") && witness.contains("[0/1, 0/1, 1/2, 1/2]"),
                "{witness}"
            );
        }
        if on1 {
            let witness = rec.witness.as_deref().unwrap();
            assert!(
                witness.contains("0b10 ") && witness.contains("[0/1, 1/2, 0/1, 1/2]"),
                "{witness}"
            );
        }
    }
}

#[test]
fn test_h_bar_spreads_a_fault_through_its_tableau() {
    let ca = CodeAbstraction::<W>::new(&torus(3), vec![LogicalGate::H(0)]).unwrap();
    let n = ca.basis().len();
    let r = ca
        .check_fault_tolerance::<f64>(&LogicalGate::H(0), &weight_one(n))
        .unwrap();
    assert!(!r.holds());
    assert_eq!(r.report.examined(), 54);
    let rejected: Vec<_> = r.records.iter().filter(|rec| !rec.tolerated).collect();
    assert!(!rejected.is_empty());
    for rec in &rejected {
        assert_eq!(
            rec.remainder_terms, 1,
            "a Clifford program leaves no remainder"
        );
        assert!(rec.propagated_weight.unwrap() > 1, "{}", rec.fault);
        let witness = rec.witness.as_deref().unwrap();
        assert!(
            witness.contains("weight") && witness.contains("logical operator"),
            "{witness}"
        );
    }
    // A Z fault off both cycles' transversal-H support may pass; every fault is decided exactly.
    assert!(r.records.iter().all(|rec| rec.path == SemanticsPath::Exact));
}

#[test]
fn test_haruna_filter_matches_the_derivation_on_both_tori() {
    for l in [3usize, 4] {
        let ca = CodeAbstraction::<W>::new(&torus(l), CodeAbstraction::<W>::table_one(2)).unwrap();
        let filter = ca.haruna_filter::<f64>().unwrap();
        assert_eq!(filter.verdicts.len(), 11);
        assert_eq!(
            filter.holds,
            vec![
                LogicalGate::Z(0),
                LogicalGate::X(0),
                LogicalGate::Z(1),
                LogicalGate::X(1)
            ],
            "torus {l}"
        );
        for (gate, report) in &filter.verdicts {
            assert_eq!(report.count, 3 * ca.basis().len());
            assert_eq!(report.paths().1, 0, "{}: every record exact", gate.name());
            if !report.holds() {
                assert!(report.worst().unwrap().witness.is_some(), "{}", gate.name());
            }
            if let LogicalGate::T(_) = gate {
                let terms: BTreeSet<usize> = report
                    .records
                    .iter()
                    .filter(|r| !r.tolerated)
                    .map(|r| r.remainder_terms)
                    .collect();
                assert_eq!(terms, BTreeSet::from([2]), "two terms at weight {l}");
            }
        }
        let shown = format!("{filter}");
        assert_eq!(shown.lines().count(), 11);
        assert!(shown.contains("holds") && shown.contains("fails") && shown.contains("exact"));
    }
}

#[test]
fn test_an_empty_fault_set_is_vacuous() {
    let ca = CodeAbstraction::<W>::new(&torus(3), vec![LogicalGate::T(0)]).unwrap();
    let r = ca
        .check_fault_tolerance::<f64>(&LogicalGate::T(0), &FaultSet::declared(&[]))
        .unwrap();
    assert_eq!(r.report.verdict(), CheckVerdict::Vacuous);
    assert_eq!(r.report.examined(), 0);
    assert_eq!(r.count, 0);
    assert!(r.worst().is_none());
    assert_eq!(r.paths(), (0, 0));
    let shown = format!("{r}");
    assert!(
        shown.contains("declared (0 faults)") && shown.contains("Vacuous"),
        "{shown}"
    );
    // A fault over the wrong register is an error, not a verdict.
    let wide = FaultSet::declared(&[Fault::new(None, vec![(18, PauliKind::X)]).unwrap()]);
    let err = ca
        .check_fault_tolerance::<f64>(&LogicalGate::T(0), &wide)
        .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
}

// ---------------------------------------------------------------------------
// The numeric path.
// ---------------------------------------------------------------------------

fn ry(wire: usize, theta: f64) -> CircuitBox<f64> {
    CircuitBox::Channel {
        wires: vec![wire],
        channel: Channel::unitary(
            &deep_causality_quantum::QubitOperator::rotation(Axis::Y, theta).unwrap(),
        )
        .unwrap(),
    }
}

/// `R_y(θ) ⊗ I` on both wires.
fn ry_block(theta: f64) -> CircuitBox<f64> {
    let r = deep_causality_quantum::QubitOperator::rotation(Axis::Y, theta).unwrap();
    let one = C::new(1.0, 0.0);
    let zero = C::new(0.0, 0.0);
    let identity = CausalTensor::from_slice(&[one, zero, zero, one], &[2, 2]);
    CircuitBox::Channel {
        wires: vec![0, 1],
        channel: Channel::from_kraus(&[r.matrix().kronecker(&identity).unwrap()]).unwrap(),
    }
}

fn trace_b() -> QcMorphism<f64> {
    let zero = C::new(0.0, 0.0);
    let one = C::new(1.0, 0.0);
    let k0 = CausalTensor::from_slice(&[one, zero, zero, zero, zero, zero, one, zero], &[2, 4]);
    let k1 = CausalTensor::from_slice(&[zero, one, zero, zero, zero, zero, zero, one], &[2, 4]);
    QcMorphism::from_kraus(&[k0, k1]).unwrap()
}

fn prepare_b() -> QcMorphism<f64> {
    let zero = C::new(0.0, 0.0);
    let one = C::new(1.0, 0.0);
    QcMorphism::from_kraus(&[CausalTensor::from_slice(
        &[one, zero, zero, zero, zero, one, zero, zero],
        &[4, 2],
    )])
    .unwrap()
}

fn low() -> CircuitModel<f64> {
    CircuitModel::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![
            ry_block(0.7),
            CircuitBox::Unitary {
                wires: vec![0, 1],
                program: vec![GateOp::X(1)],
            },
        ],
        vec![0, 1],
        vec![0, 1],
    )
    .unwrap()
}

fn two_wire_abstraction() -> Abstraction<f64, CircuitModel<f64>, CircuitModel<f64>> {
    let high = CircuitModel::ungrouped(vec![WireType::qubit()], vec![ry(0, 0.7)], vec![0], vec![0])
        .unwrap();
    Abstraction::new(
        low(),
        high,
        TypeAlignment::new(vec![(vec![0], vec![0, 1], trace_b(), prepare_b())]).unwrap(),
        vec![(Query::Io, Query::Io)],
    )
    .unwrap()
}

#[test]
fn test_numeric_path_tolerates_faults_that_tau_traces_out() {
    let caps = NumericCaps::default();
    let a = two_wire_abstraction();
    let traced_z = Fault::new(Some(0), vec![(1, PauliKind::Z)]).unwrap();
    let traced_x_at_inputs = Fault::new(None, vec![(1, PauliKind::X)]).unwrap();
    let kept_x = Fault::new(Some(0), vec![(0, PauliKind::X)]).unwrap();
    let kept_z_at_inputs = Fault::new(None, vec![(0, PauliKind::Z)]).unwrap();
    let set = FaultSet::declared(&[traced_z, traced_x_at_inputs, kept_x, kept_z_at_inputs]);
    let r = a.check_fault_tolerance(&set, &caps).unwrap();
    assert_eq!(r.report.examined(), 4);
    assert_eq!(r.origin, FaultOrigin::Declared);
    assert_eq!(r.paths(), (0, 4));
    let verdicts: Vec<bool> = r.records.iter().map(|rec| rec.tolerated).collect();
    assert_eq!(verdicts, vec![true, true, false, false]);
    assert!(r.records[0].residual < 1e-12 && r.records[1].residual < 1e-12);
    // An X on the kept wire after R_y ⊗ I: the square differs by ‖J(X R_y) − J(R_y)‖_F on one
    // qubit, which is 2√2 · sin(θ'/2) for the rotation angle θ' between X R_y and R_y; the
    // residual is strictly positive and the witness names the norm.
    assert!(r.records[2].residual > 1.0, "{}", r.records[2].residual);
    assert!(r.records[3].residual > 1.0, "{}", r.records[3].residual);
    assert!(
        r.records[2]
            .witness
            .as_deref()
            .unwrap()
            .contains("frobenius-on-choi")
    );
    assert!(!r.worst().unwrap().tolerated);
    assert!(!r.holds());
    assert_eq!(
        r.records
            .iter()
            .map(|rec| rec.remainder_terms)
            .sum::<usize>(),
        0
    );
}

#[test]
fn test_the_faulted_model_inserts_one_node_at_the_location() {
    let model = low();
    let after_first = Fault::new(Some(0), vec![(1, PauliKind::Y)]).unwrap();
    let f = model.faulted(&after_first).unwrap();
    assert_eq!(f.boxes().len(), 3);
    assert_eq!(f.nodes().len(), 3);
    assert_eq!(f.boxes()[1].kind(), "unitary");
    assert_eq!(f.boxes()[1].quantum_wires(), &[1]);
    assert!(
        matches!(&f.boxes()[1], CircuitBox::Unitary { program, .. } if program == &vec![GateOp::Y(0)])
    );
    assert_eq!(f.nodes()[0], vec![0]);
    assert_eq!(f.nodes()[1], vec![2], "the old second box moved up");
    assert_eq!(f.nodes()[2], vec![1], "the fault is the new last node");
    assert_eq!(f.inputs(), model.inputs());
    assert_eq!(f.outputs(), model.outputs());
    let at_inputs = Fault::new(None, vec![(0, PauliKind::X), (1, PauliKind::Z)]).unwrap();
    let g = model.faulted(&at_inputs).unwrap();
    assert_eq!(g.boxes()[0].quantum_wires(), &[0, 1]);
    assert!(
        matches!(&g.boxes()[0], CircuitBox::Unitary { program, .. } if program == &vec![GateOp::X(0), GateOp::Z(1)])
    );
    assert_eq!(g.nodes()[2], vec![0]);
    let missing = model
        .faulted(&Fault::new(Some(2), vec![(0, PauliKind::X)]).unwrap())
        .unwrap_err();
    assert!(
        matches!(missing.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("node 2"))
    );
    let classical = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Measurement {
            wires: vec![0],
            outcome: 1,
        }],
        vec![0],
        vec![1],
    )
    .unwrap();
    let err = classical
        .faulted(&Fault::new(None, vec![(1, PauliKind::X)]).unwrap())
        .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("not a quantum wire"))
    );
    // The signature validates the location against the DAG.
    let dag = model.induced_dag();
    let ok = QuerySignature::new(&dag, vec![Query::Fault(after_first)]).unwrap();
    assert_eq!(ok.len(), 1);
    let bad = QuerySignature::new(
        &dag,
        vec![Query::Fault(
            Fault::new(Some(7), vec![(0, PauliKind::X)]).unwrap(),
        )],
    )
    .unwrap_err();
    assert!(matches!(bad.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("node 7")));
}

/// Every record's verdict is the verdict of its check, so `holds` and `tolerated` agree on the
/// numeric path; the boundary case, a residual equal to the tolerance, is not constructible from
/// a Pauli fault, so the agreement is pinned record by record.
#[test]
fn test_numeric_records_agree_with_their_checks() {
    let caps = NumericCaps::default();
    let a = two_wire_abstraction();
    let set = FaultSet::declared(&[
        Fault::new(Some(0), vec![(1, PauliKind::Z)]).unwrap(),
        Fault::new(Some(0), vec![(0, PauliKind::X)]).unwrap(),
    ]);
    let r = a.check_fault_tolerance(&set, &caps).unwrap();
    for (rec, check) in r.records.iter().zip(r.report.checks()) {
        assert_eq!(rec.tolerated, check.accepted, "{}", rec.fault);
        assert_eq!(rec.residual, check.measured);
        assert_eq!(rec.witness.is_none(), check.accepted);
    }
    assert_eq!(r.holds(), r.tolerated() == r.count);
    assert!(!r.holds());
}
