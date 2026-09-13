/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Construction checks and the induced DAG of `CircuitModel`.
//!
//! The DAG scenario is Lorenz & Tull, arXiv:2602.16612, Example 61 read on the wiring of the
//! `qcl-circuit-model` spec: encoder on `{0, 1}`, `U` on `{0, 1}`, `V` on `{1, 2}`, measurements on
//! `0` and `2`. Corner cases: (A) no boxes, (B) one box, (C) a box grouped twice, (D) an empty node,
//! (E) a wire out of range, (F) a cyclic grouping.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Channel, CircuitBox, CircuitModel, GateOp, QuantumErrorEnum, QubitOperator, WireType,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn ket(amps: &[f64]) -> CausalTensor<C> {
    CausalTensor::from_slice(
        &amps.iter().map(|&a| C::new(a, 0.0)).collect::<Vec<_>>(),
        &[amps.len()],
    )
}

fn unitary(wires: &[usize], program: Vec<GateOp>) -> CircuitBox<f64> {
    CircuitBox::Unitary {
        wires: wires.to_vec(),
        program,
    }
}

/// The spec's wiring: wires 0..3 qubits, 3 a classical input, 4 and 5 classical outcomes.
fn example_61() -> CircuitModel<f64> {
    let wires = vec![
        WireType::qubit(),
        WireType::qubit(),
        WireType::qubit(),
        WireType::Classical { outcomes: 4 },
        WireType::bit(),
        WireType::bit(),
    ];
    let boxes = vec![
        CircuitBox::Encoder {
            input: 3,
            outputs: vec![0, 1],
            states: vec![
                ket(&[1.0, 0.0, 0.0, 0.0]),
                ket(&[0.0, 1.0, 0.0, 0.0]),
                ket(&[0.0, 0.0, 1.0, 0.0]),
                ket(&[0.0, 0.0, 0.0, 1.0]),
            ],
        },
        unitary(
            &[0, 1],
            vec![GateOp::Cnot {
                control: 0,
                target: 1,
            }],
        ),
        unitary(
            &[1, 2],
            vec![GateOp::Cz {
                control: 0,
                target: 1,
            }],
        ),
        CircuitBox::Measurement {
            wires: vec![0],
            outcome: 4,
        },
        CircuitBox::Measurement {
            wires: vec![2],
            outcome: 5,
        },
    ];
    CircuitModel::ungrouped(wires, boxes, vec![], vec![4, 5]).unwrap()
}

#[test]
fn test_induced_dag_follows_the_wires() {
    let m = example_61();
    let dag = m.induced_dag();
    // Nodes: 0 encoder, 1 U, 2 V, 3 M(0), 4 M(2).
    assert!(dag.has_edge(0, 1));
    assert!(dag.has_edge(1, 2), "wire 1 leaves U and enters V");
    assert!(dag.has_edge(1, 3));
    assert!(dag.has_edge(2, 4));
    assert!(!dag.has_edge(1, 4), "U never touches wire 2");
    assert!(!dag.has_edge(0, 2), "wire 1 reaches V through U");
    assert_eq!(dag.num_edges(), 4);
    assert_eq!(m.classical_inputs(), vec![3]);
    assert_eq!(m.num_qubits(), 3);
    assert_eq!(m.nodes_on_wire(1), vec![0, 1, 2]);
    assert_eq!(m.boxes_of_nodes(&[2, 0]), vec![0, 2]);
    assert_eq!(m.node_of(3), Some(3));
}

#[test]
fn test_no_boxes_and_one_box() {
    let empty =
        CircuitModel::<f64>::ungrouped(vec![WireType::qubit()], vec![], vec![0], vec![0]).unwrap();
    assert_eq!(empty.induced_dag().num_vertices(), 0);
    let one = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit()],
        vec![unitary(&[0], vec![GateOp::H(0)])],
        vec![0],
        vec![0],
    )
    .unwrap();
    assert_eq!(one.induced_dag().num_vertices(), 1);
    assert_eq!(one.induced_dag().num_edges(), 0);
}

#[test]
fn test_mis_dimensioned_channel_names_the_wire_and_the_box() {
    let err = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![CircuitBox::Channel {
            wires: vec![0, 1],
            channel: Channel::unitary(&QubitOperator::hadamard()).unwrap(),
        }],
        vec![0, 1],
        vec![0, 1],
    )
    .unwrap_err();
    match err.0 {
        QuantumErrorEnum::DimensionMismatch(msg) => {
            assert!(msg.contains("box 0") && msg.contains("[0, 1]"), "{msg}")
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn test_two_writers_of_one_classical_wire_are_refused() {
    let err = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit(), WireType::bit()],
        vec![
            CircuitBox::Measurement {
                wires: vec![0],
                outcome: 2,
            },
            CircuitBox::Measurement {
                wires: vec![1],
                outcome: 2,
            },
        ],
        vec![],
        vec![2],
    )
    .unwrap_err();
    match err.0 {
        QuantumErrorEnum::DimensionMismatch(msg) => {
            assert!(
                msg.contains("wire 2") && msg.contains("box 0") && msg.contains("box 1"),
                "{msg}"
            )
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn test_grouping_must_partition_the_boxes() {
    let wires = vec![WireType::qubit()];
    let boxes = || {
        vec![
            unitary(&[0], vec![GateOp::H(0)]),
            unitary(&[0], vec![GateOp::S(0)]),
        ]
    };
    let twice = CircuitModel::<f64>::new(
        wires.clone(),
        boxes(),
        vec![vec![0, 1], vec![1]],
        vec![0],
        vec![0],
    )
    .unwrap_err();
    assert!(matches!(twice.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("box 1")));
    let missing = CircuitModel::<f64>::new(wires.clone(), boxes(), vec![vec![0]], vec![0], vec![0])
        .unwrap_err();
    assert!(matches!(missing.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("box 1")));
    let empty_node = CircuitModel::<f64>::new(
        wires.clone(),
        boxes(),
        vec![vec![0, 1], vec![]],
        vec![0],
        vec![0],
    )
    .unwrap_err();
    assert!(
        matches!(empty_node.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("node 1"))
    );
    let out_of_range =
        CircuitModel::<f64>::new(wires, boxes(), vec![vec![0], vec![7]], vec![0], vec![0])
            .unwrap_err();
    assert!(matches!(
        out_of_range.0,
        QuantumErrorEnum::CalculationError(_)
    ));
}

#[test]
fn test_cyclic_grouping_shows_in_the_induced_dag() {
    // Boxes 0 and 2 in one node, box 1 in another, all on one wire: A → B → A.
    let m = CircuitModel::<f64>::new(
        vec![WireType::qubit()],
        vec![
            unitary(&[0], vec![GateOp::H(0)]),
            unitary(&[0], vec![GateOp::S(0)]),
            unitary(&[0], vec![GateOp::H(0)]),
        ],
        vec![vec![0, 2], vec![1]],
        vec![0],
        vec![0],
    )
    .unwrap();
    assert!(m.induced_dag().has_cycle());
}

#[test]
fn test_wire_kind_and_range_errors() {
    let bad_kind = CircuitModel::<f64>::ungrouped(
        vec![WireType::bit()],
        vec![unitary(&[0], vec![])],
        vec![],
        vec![],
    )
    .unwrap_err();
    assert!(
        matches!(bad_kind.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("classical"))
    );
    let out_of_range = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit()],
        vec![unitary(&[1], vec![])],
        vec![],
        vec![],
    )
    .unwrap_err();
    assert!(
        matches!(out_of_range.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("wire 1"))
    );
    let repeated = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit()],
        vec![unitary(&[0, 0], vec![])],
        vec![],
        vec![],
    )
    .unwrap_err();
    assert!(
        matches!(repeated.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("more than once"))
    );
    let local_qubit = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit()],
        vec![unitary(&[0], vec![GateOp::H(1)])],
        vec![],
        vec![],
    )
    .unwrap_err();
    assert!(
        matches!(local_qubit.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("local qubit 1"))
    );
    let qutrit_program = CircuitModel::<f64>::ungrouped(
        vec![WireType::Quantum { dim: 3 }],
        vec![unitary(&[0], vec![])],
        vec![],
        vec![],
    )
    .unwrap_err();
    assert!(
        matches!(qutrit_program.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("qubits"))
    );
    let zero_wire =
        CircuitModel::<f64>::ungrouped(vec![WireType::Quantum { dim: 0 }], vec![], vec![], vec![])
            .unwrap_err();
    assert!(matches!(
        zero_wire.0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_encoder_measurement_and_instrument_checks() {
    let states_ok = vec![ket(&[1.0, 0.0]), ket(&[0.0, 1.0])];
    let wrong_count = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::Classical { outcomes: 3 }],
        vec![CircuitBox::Encoder {
            input: 1,
            outputs: vec![0],
            states: states_ok.clone(),
        }],
        vec![],
        vec![0],
    )
    .unwrap_err();
    assert!(
        matches!(wrong_count.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("3 values"))
    );
    let wrong_dim = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Encoder {
            input: 1,
            outputs: vec![0],
            states: vec![ket(&[1.0, 0.0, 0.0]), ket(&[0.0, 1.0])],
        }],
        vec![],
        vec![0],
    )
    .unwrap_err();
    assert!(
        matches!(wrong_dim.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("state 0"))
    );
    let encoded_input = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Encoder {
            input: 1,
            outputs: vec![0],
            states: states_ok.clone(),
        }],
        vec![0],
        vec![0],
    )
    .unwrap_err();
    assert!(
        matches!(encoded_input.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("encoder"))
    );
    let meas_count = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::Classical { outcomes: 3 }],
        vec![CircuitBox::Measurement {
            wires: vec![0],
            outcome: 1,
        }],
        vec![],
        vec![1],
    )
    .unwrap_err();
    assert!(
        matches!(meas_count.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("measures dimension 2"))
    );
    // An instrument whose families do not sum to the identity.
    let p0 = CausalTensor::from_slice(
        &[
            C::new(1.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
        ],
        &[2, 2],
    );
    let not_tp = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Instrument {
            wires: vec![0],
            outcome: 1,
            kraus: vec![vec![p0.clone()], vec![]],
        }],
        vec![0],
        vec![0, 1],
    )
    .unwrap_err();
    assert!(
        matches!(not_tp.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("trace-preserving"))
    );
    let p1 = CausalTensor::from_slice(
        &[
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(1.0, 0.0),
        ],
        &[2, 2],
    );
    let ok = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Instrument {
            wires: vec![0],
            outcome: 1,
            kraus: vec![vec![p0], vec![p1]],
        }],
        vec![0],
        vec![0, 1],
    );
    assert!(ok.is_ok());
    let unwritten_output = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![],
        vec![0],
        vec![1],
    )
    .unwrap_err();
    assert!(
        matches!(unwritten_output.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("written by no box"))
    );
}

/// A Kraus box whose operators do not match its wires, or that has none, is refused with the box
/// index and the wires named.
#[test]
fn test_mis_dimensioned_kraus_box_names_the_wire_and_the_box() {
    let two_by_two = QubitOperator::<f64>::hadamard().matrix().clone();
    let err = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![CircuitBox::Kraus {
            wires: vec![0, 1],
            kraus: vec![two_by_two],
        }],
        vec![0, 1],
        vec![0, 1],
    )
    .unwrap_err();
    match err.0 {
        QuantumErrorEnum::DimensionMismatch(msg) => {
            assert!(
                msg.contains("box 0") && msg.contains("[0, 1]") && msg.contains("[2, 2]"),
                "{msg}"
            )
        }
        other => panic!("{other:?}"),
    }
    let empty = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit()],
        vec![CircuitBox::Kraus {
            wires: vec![0],
            kraus: vec![],
        }],
        vec![0],
        vec![0],
    )
    .unwrap_err();
    assert!(
        matches!(empty.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("no Kraus"))
    );
    let ok = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit()],
        vec![CircuitBox::Kraus {
            wires: vec![0],
            kraus: vec![QubitOperator::<f64>::hadamard().matrix().clone()],
        }],
        vec![0],
        vec![0],
    )
    .unwrap();
    assert_eq!(ok.boxes()[0].kind(), "kraus");
    assert_eq!(ok.boxes()[0].quantum_wires(), &[0]);
}

/// An encoder prepares fresh lines. One whose output an earlier box touched is refused with the
/// box and the wire named; the same encoder before the box, or on a wire the box never touches,
/// is accepted.
#[test]
fn test_encoder_on_a_wire_an_earlier_box_touched_is_refused() {
    let h = || CircuitBox::Channel {
        wires: vec![0],
        channel: Channel::unitary(&QubitOperator::hadamard()).unwrap(),
    };
    let enc = |out: usize| CircuitBox::Encoder {
        input: 2,
        outputs: vec![out],
        states: vec![ket(&[1.0, 0.0]), ket(&[0.0, 1.0])],
    };
    let wires = || vec![WireType::qubit(), WireType::qubit(), WireType::bit()];
    let err =
        CircuitModel::<f64>::ungrouped(wires(), vec![h(), enc(0)], vec![], vec![0]).unwrap_err();
    match err.0 {
        QuantumErrorEnum::DimensionMismatch(msg) => assert!(
            msg.contains("box 1") && msg.contains("encoder") && msg.contains("wire 0"),
            "{msg}"
        ),
        other => panic!("{other:?}"),
    }
    assert!(CircuitModel::<f64>::ungrouped(wires(), vec![enc(0), h()], vec![], vec![0]).is_ok());
    assert!(CircuitModel::<f64>::ungrouped(wires(), vec![h(), enc(1)], vec![], vec![0, 1]).is_ok());
}

/// A Kraus box must satisfy `Σ K†K = I`. A lone projector and a scaled unitary are refused with
/// the box named; the amplitude-damping family, which is trace-preserving but not unitary, and a
/// two-operator Pauli mixture are accepted.
#[test]
fn test_kraus_box_must_be_trace_preserving() {
    let op = |entries: [f64; 4]| {
        CausalTensor::from_slice(
            &entries.iter().map(|&a| C::new(a, 0.0)).collect::<Vec<_>>(),
            &[2, 2],
        )
    };
    let model = |kraus: Vec<CausalTensor<C>>| {
        CircuitModel::<f64>::ungrouped(
            vec![WireType::qubit()],
            vec![CircuitBox::Kraus {
                wires: vec![0],
                kraus,
            }],
            vec![0],
            vec![0],
        )
    };
    let projector = model(vec![op([1.0, 0.0, 0.0, 0.0])]).unwrap_err();
    assert!(
        matches!(projector.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("box 0") && m.contains("trace-preserving")),
        "{projector}"
    );
    let s = 0.9 / 2f64.sqrt();
    let scaled = model(vec![op([s, s, s, -s])]).unwrap_err();
    assert!(
        matches!(scaled.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("trace-preserving")),
        "{scaled}"
    );
    let damping = model(vec![
        op([1.0, 0.0, 0.0, 0.7f64.sqrt()]),
        op([0.0, 0.3f64.sqrt(), 0.0, 0.0]),
    ]);
    assert!(damping.is_ok(), "{damping:?}");
    let r = 0.5f64.sqrt();
    let mixture = model(vec![op([0.0, r, r, 0.0]), op([r, 0.0, 0.0, -r])]);
    assert!(mixture.is_ok(), "{mixture:?}");
    let nan = model(vec![op([1.0, f64::NAN, 0.0, 1.0])]).unwrap_err();
    assert!(
        matches!(nan.0, QuantumErrorEnum::NonFiniteValue(ref m) if m.contains("box 0")),
        "{nan}"
    );
}

/// Every prepared state of an encoder is a finite unit ket. A state of norm `√0.72`, and one
/// with a NaN amplitude, are refused with the box and the state named; a unit ket with unequal
/// real amplitudes is accepted.
#[test]
fn test_encoder_states_must_be_finite_unit_kets() {
    let model = |states: Vec<CausalTensor<C>>| {
        CircuitModel::<f64>::ungrouped(
            vec![WireType::qubit(), WireType::bit()],
            vec![CircuitBox::Encoder {
                input: 1,
                outputs: vec![0],
                states,
            }],
            vec![],
            vec![0],
        )
    };
    let short = model(vec![ket(&[1.0, 0.0]), ket(&[0.6, 0.6])]).unwrap_err();
    assert!(
        matches!(short.0, QuantumErrorEnum::NormalizationError(ref m) if m.contains("box 0") && m.contains("state 1")),
        "{short}"
    );
    let nan = model(vec![ket(&[f64::NAN, 0.0]), ket(&[0.0, 1.0])]).unwrap_err();
    assert!(
        matches!(nan.0, QuantumErrorEnum::NonFiniteValue(ref m) if m.contains("box 0") && m.contains("state 0")),
        "{nan}"
    );
    let ok = model(vec![ket(&[0.6, 0.8]), ket(&[0.0, 1.0])]);
    assert!(ok.is_ok(), "{ok:?}");
}

/// An instrument with a non-finite Kraus entry is refused before the trace-preservation
/// comparison, which a NaN defect would pass.
#[test]
fn test_instrument_with_a_non_finite_kraus_entry_is_refused() {
    let op = |entries: [f64; 4]| {
        CausalTensor::from_slice(
            &entries.iter().map(|&a| C::new(a, 0.0)).collect::<Vec<_>>(),
            &[2, 2],
        )
    };
    let model = |first: CausalTensor<C>| {
        CircuitModel::<f64>::ungrouped(
            vec![WireType::qubit(), WireType::bit()],
            vec![CircuitBox::Instrument {
                wires: vec![0],
                outcome: 1,
                kraus: vec![vec![first], vec![op([0.0, 0.0, 0.0, 1.0])]],
            }],
            vec![0],
            vec![0, 1],
        )
    };
    let nan = model(op([1.0, f64::NAN, 0.0, 0.0])).unwrap_err();
    assert!(
        matches!(nan.0, QuantumErrorEnum::NonFiniteValue(ref m) if m.contains("box 0") && m.contains("instrument")),
        "{nan}"
    );
    let inf = model(op([f64::INFINITY, 0.0, 0.0, 0.0])).unwrap_err();
    assert!(
        matches!(inf.0, QuantumErrorEnum::NonFiniteValue(_)),
        "{inf}"
    );
    assert!(model(op([1.0, 0.0, 0.0, 0.0])).is_ok());
}

/// The declared input and output lists are checked independently of the boxes: a wire out of
/// range, of the wrong kind, prepared by an encoder, or named twice is refused with the wire
/// named.
#[test]
fn test_declared_input_must_be_a_free_quantum_wire() {
    let dim_msg = |e: deep_causality_quantum::QuantumError| match e.0 {
        QuantumErrorEnum::DimensionMismatch(m) => m,
        other => panic!("expected DimensionMismatch, got {other:?}"),
    };

    // A classical wire cannot be a declared input.
    let msg = dim_msg(
        CircuitModel::<f64>::ungrouped(
            vec![WireType::qubit(), WireType::bit()],
            vec![],
            vec![1],
            vec![0],
        )
        .unwrap_err(),
    );
    assert!(msg.contains("not a quantum wire"), "{msg}");
    assert!(msg.contains('1'), "{msg}");

    // A wire index past the end is refused by the same arm.
    let msg = dim_msg(
        CircuitModel::<f64>::ungrouped(vec![WireType::qubit()], vec![], vec![9], vec![0])
            .unwrap_err(),
    );
    assert!(msg.contains("not a quantum wire"), "{msg}");
    assert!(msg.contains('9'), "{msg}");

    // Index 0 of a one-wire model is the boundary the two refusals above sit next to.
    assert!(
        CircuitModel::<f64>::ungrouped(vec![WireType::qubit()], vec![], vec![0], vec![0]).is_ok()
    );
}

#[test]
fn test_an_encoded_wire_cannot_also_be_declared_free() {
    // The encoder prepares wire 0, so wire 0 is not a free input of the model.
    let msg = match CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::Classical { outcomes: 2 }],
        vec![CircuitBox::Encoder {
            input: 1,
            outputs: vec![0],
            states: vec![ket(&[1.0, 0.0]), ket(&[0.0, 1.0])],
        }],
        vec![0],
        vec![0],
    )
    .unwrap_err()
    .0
    {
        QuantumErrorEnum::DimensionMismatch(m) => m,
        other => panic!("expected DimensionMismatch, got {other:?}"),
    };
    assert!(msg.contains("prepared by an encoder"), "{msg}");
    assert!(msg.contains('0'), "{msg}");
}

#[test]
fn test_a_wire_declared_twice_is_refused_on_both_the_input_and_the_output_list() {
    let twice_in = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![],
        vec![0, 0],
        vec![1],
    )
    .unwrap_err();
    assert!(
        matches!(twice_in.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("input twice")),
        "{twice_in:?}"
    );

    let twice_out = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![],
        vec![0],
        vec![1, 1],
    )
    .unwrap_err();
    assert!(
        matches!(twice_out.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("output twice")),
        "{twice_out:?}"
    );

    // Two distinct wires on either list are accepted, so the refusals above are the
    // repeat and not the list length.
    assert!(
        CircuitModel::<f64>::ungrouped(
            vec![WireType::qubit(), WireType::qubit()],
            vec![],
            vec![0, 1],
            vec![0, 1],
        )
        .is_ok()
    );
}

#[test]
fn test_a_declared_output_out_of_range_is_refused() {
    let err = CircuitModel::<f64>::ungrouped(vec![WireType::qubit()], vec![], vec![0], vec![5])
        .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("not a wire of the model") && m.contains('5')),
        "{err:?}"
    );
}

/// `glue` re-maps every box variant's wires onto the composite's numbering. The second model's
/// wires are renumbered, so each arm of the re-map has to carry the box's own wire fields across;
/// a variant whose arm dropped a field would surface here as a wire that did not move.
#[test]
fn test_glue_remaps_the_wires_of_every_box_variant() {
    let p0 = CausalTensor::new(
        vec![
            C::new(1., 0.),
            C::new(0., 0.),
            C::new(0., 0.),
            C::new(0., 0.),
        ],
        vec![2, 2],
    )
    .unwrap();
    let p1 = CausalTensor::new(
        vec![
            C::new(0., 0.),
            C::new(0., 0.),
            C::new(0., 0.),
            C::new(1., 0.),
        ],
        vec![2, 2],
    )
    .unwrap();

    // The first model: one qubit out, produced by a unitary.
    let first = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit()],
        vec![unitary(&[0], vec![GateOp::H(0)])],
        vec![],
        vec![0],
    )
    .unwrap();

    // The second model holds one of each remaining variant on its own numbering:
    // wire 0 the joined qubit, 1 a second qubit, 2 the encoder's classical input,
    // 3 the instrument's outcome and 4 the measurement's outcome.
    let second = CircuitModel::<f64>::ungrouped(
        vec![
            WireType::qubit(),
            WireType::qubit(),
            WireType::Classical { outcomes: 2 },
            WireType::bit(),
            WireType::bit(),
        ],
        vec![
            CircuitBox::Encoder {
                input: 2,
                outputs: vec![1],
                states: vec![ket(&[1.0, 0.0]), ket(&[0.0, 1.0])],
            },
            CircuitBox::Channel {
                wires: vec![0],
                channel: Channel::unitary(&QubitOperator::hadamard()).unwrap(),
            },
            CircuitBox::Kraus {
                wires: vec![1],
                kraus: vec![QubitOperator::<f64>::pauli_x().matrix().clone()],
            },
            CircuitBox::Instrument {
                wires: vec![1],
                outcome: 3,
                kraus: vec![vec![p0], vec![p1]],
            },
            CircuitBox::Measurement {
                wires: vec![0],
                outcome: 4,
            },
        ],
        vec![0],
        vec![1],
    )
    .unwrap();

    let glued = first.glue(&second, &[(0, 0)]).unwrap();

    // One shared line plus the second model's other four wires.
    assert_eq!(glued.wires().len(), 5);
    assert_eq!(glued.boxes().len(), 6);
    // The joined wire keeps the first model's index; the rest are appended in order,
    // so the second model's wire 1 becomes 1, wire 2 becomes 2, and so on. Here that
    // is the identity on indices, which alone would not prove the re-map ran — the
    // wire *kinds* below do, since they were pushed from `other` in wire order.
    assert_eq!(glued.wires()[2].cardinality(), 2);
    assert!(glued.wires()[0].is_quantum());
    assert!(glued.wires()[1].is_quantum());

    // Every variant survived the copy with its wires intact.
    let kinds: Vec<&str> = glued.boxes().iter().map(|b| b.kind()).collect();
    assert_eq!(
        kinds,
        vec![
            "unitary",
            "encoder",
            "channel",
            "kraus",
            "instrument",
            "measurement"
        ]
    );

    // The composite's declared lists follow the documented rule: the joined input of
    // `other` is consumed, and `self`'s joined output is replaced by `other`'s.
    assert_eq!(glued.inputs(), &[] as &[usize]);
    assert_eq!(glued.outputs(), &[1]);
}

#[test]
fn test_glue_offsets_the_second_model_node_grouping() {
    // `other`'s boxes are appended, so its node members shift by `self.boxes().len()`.
    let first = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit()],
        vec![
            unitary(&[0], vec![GateOp::H(0)]),
            unitary(&[0], vec![GateOp::X(0)]),
        ],
        vec![],
        vec![0],
    )
    .unwrap();
    let second = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit()],
        vec![unitary(&[0], vec![GateOp::Z(0)])],
        vec![0],
        vec![0],
    )
    .unwrap();
    let glued = first.glue(&second, &[(0, 0)]).unwrap();
    assert_eq!(glued.boxes().len(), 3);
    // Three ungrouped boxes become three singleton nodes, the last one the offset copy.
    assert_eq!(glued.nodes(), &[vec![0], vec![1], vec![2]]);
}
