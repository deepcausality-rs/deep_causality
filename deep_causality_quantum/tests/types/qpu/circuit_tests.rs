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
