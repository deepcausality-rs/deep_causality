/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qpu")]

use deep_causality_quantum::{GateOp, QuantumCircuit};

#[test]
fn test_valid_bell_circuit() {
    let circuit = QuantumCircuit::new(
        2,
        vec![
            GateOp::H(0),
            GateOp::Cnot {
                control: 0,
                target: 1,
            },
        ],
        vec![0, 1],
    )
    .unwrap();
    assert_eq!(circuit.num_qubits(), 2);
    assert_eq!(circuit.ops().len(), 2);
    assert_eq!(circuit.measure(), &[0, 1]);
}

#[test]
fn test_zero_qubits_rejected() {
    assert!(QuantumCircuit::new(0, vec![], vec![]).is_err());
}

#[test]
fn test_out_of_range_gate_qubit_rejected() {
    assert!(QuantumCircuit::new(1, vec![GateOp::X(1)], vec![]).is_err());
    assert!(
        QuantumCircuit::new(
            2,
            vec![GateOp::Cnot {
                control: 0,
                target: 2
            }],
            vec![]
        )
        .is_err()
    );
}

#[test]
fn test_coincident_two_qubit_gate_rejected() {
    assert!(
        QuantumCircuit::new(
            2,
            vec![GateOp::Cnot {
                control: 1,
                target: 1
            }],
            vec![]
        )
        .is_err()
    );
    assert!(
        QuantumCircuit::new(
            2,
            vec![GateOp::Cz {
                control: 0,
                target: 0
            }],
            vec![]
        )
        .is_err()
    );
}

#[test]
fn test_out_of_range_measurement_rejected() {
    assert!(QuantumCircuit::new(2, vec![], vec![2]).is_err());
}

#[test]
fn test_gate_qubits_accessor() {
    assert_eq!(GateOp::H(3).qubits(), vec![3]);
    assert_eq!(
        GateOp::Cnot {
            control: 1,
            target: 4
        }
        .qubits(),
        vec![1, 4]
    );
}

// ---------------------------------------------------------------------------
// The extended alphabet: S†, T†, CS†, CCZ and the general C^(m-1)Z.
// ---------------------------------------------------------------------------

#[test]
fn test_gate_qubits_accessor_covers_the_extended_alphabet() {
    assert_eq!(GateOp::Sdg(2).qubits(), vec![2]);
    assert_eq!(GateOp::Tdg(5).qubits(), vec![5]);
    assert_eq!(
        GateOp::Csdg {
            control: 0,
            target: 3
        }
        .qubits(),
        vec![0, 3]
    );
    assert_eq!(
        GateOp::Ccz {
            q0: 4,
            q1: 1,
            q2: 2
        }
        .qubits(),
        vec![4, 1, 2]
    );
    assert_eq!(
        GateOp::Cmz {
            qubits: vec![7, 0, 3]
        }
        .qubits(),
        vec![7, 0, 3]
    );
}

#[test]
fn test_repeated_qubit_rejected_at_three_and_at_m() {
    // The defect this closes: the old check was `qs.len() == 2 && qs[0] == qs[1]`,
    // so it fired for two-qubit gates only. A repeat in a three-or-more-qubit
    // gate reduces its control set silently — Ccz{0,0,1} acts as Cz{0,1} — and
    // was accepted, then mis-applied by the simulator with nothing raised.
    assert!(
        QuantumCircuit::new(
            3,
            vec![GateOp::Ccz {
                q0: 0,
                q1: 0,
                q2: 1
            }],
            vec![]
        )
        .is_err()
    );
    // A repeat in a non-adjacent position is caught too, not just an adjacent pair.
    assert!(
        QuantumCircuit::new(
            3,
            vec![GateOp::Ccz {
                q0: 2,
                q1: 1,
                q2: 2
            }],
            vec![]
        )
        .is_err()
    );
    assert!(
        QuantumCircuit::new(
            4,
            vec![GateOp::Cmz {
                qubits: vec![0, 1, 2, 1]
            }],
            vec![]
        )
        .is_err()
    );
    assert!(
        QuantumCircuit::new(
            2,
            vec![GateOp::Csdg {
                control: 1,
                target: 1
            }],
            vec![]
        )
        .is_err()
    );
}

#[test]
fn test_distinct_qubits_at_three_and_at_m_accepted() {
    // The rejection above must discriminate: the same shapes with distinct
    // indices are valid, so the check is not simply refusing every wide gate.
    assert!(
        QuantumCircuit::new(
            3,
            vec![GateOp::Ccz {
                q0: 0,
                q1: 1,
                q2: 2
            }],
            vec![]
        )
        .is_ok()
    );
    assert!(
        QuantumCircuit::new(
            4,
            vec![GateOp::Cmz {
                qubits: vec![3, 0, 2, 1]
            }],
            vec![]
        )
        .is_ok()
    );
}

#[test]
fn test_empty_control_list_rejected() {
    // C^(m-1)Z over no qubits would phase the whole register by -1: a global
    // phase, unobservable and inexpressible as intent.
    assert!(QuantumCircuit::new(2, vec![GateOp::Cmz { qubits: vec![] }], vec![]).is_err());
}

#[test]
fn test_out_of_range_rejected_across_the_extended_alphabet() {
    assert!(QuantumCircuit::new(2, vec![GateOp::Sdg(2)], vec![]).is_err());
    assert!(QuantumCircuit::new(2, vec![GateOp::Tdg(9)], vec![]).is_err());
    assert!(
        QuantumCircuit::new(
            3,
            vec![GateOp::Ccz {
                q0: 0,
                q1: 1,
                q2: 3
            }],
            vec![]
        )
        .is_err()
    );
    assert!(
        QuantumCircuit::new(
            3,
            vec![GateOp::Cmz {
                qubits: vec![0, 1, 5]
            }],
            vec![]
        )
        .is_err()
    );
}
