/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The abstraction chains at their edges: the gates `encode_program` carries into an outer block,
//! the emitter it lacks, and an outer code with no logical qubits.

use deep_causality_quantum::utils_tests::{four_two_two, no_logical_qubits, three_three_one};
use deep_causality_quantum::{
    CodeAbstraction, GateOp, LogicalGate, QuantumErrorEnum, concatenated_code, encode_program,
    logical_cz, logical_multi_cz,
};

type W = u64;

#[test]
fn test_t_bar_concatenation_names_the_missing_controlled_s_dagger_emitter() {
    let complex = four_two_two();
    // T̄ on a weight-2 representative is `T T CS†` (Haruna Eq. 3.56); the outer code has no CS̄†
    // emitter, so the concatenation is refused by the emitter's name, not by a panic.
    let err = concatenated_code::<W, _, _, f64>(&complex, &complex, &LogicalGate::T(0))
        .expect_err("refused");
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("CS̄†") && m.contains("Csdg")),
        "{err}"
    );
}

#[test]
fn test_a_controlled_phase_within_one_block_is_encoded_by_the_outer_multi_cz() {
    let outer = CodeAbstraction::<W>::new(&three_three_one(), vec![]).unwrap();
    assert_eq!(outer.basis().num_logical_qubits(), 3);
    let gamma = outer.basis().homology();
    let ccz = encode_program::<W, f64>(
        &outer,
        &[GateOp::Ccz {
            q0: 0,
            q1: 1,
            q2: 2,
        }],
    )
    .unwrap();
    assert_eq!(
        ccz,
        logical_multi_cz(&[&gamma[0], &gamma[1], &gamma[2]]).unwrap()
    );
    let cmz = encode_program::<W, f64>(&outer, &[GateOp::Cmz { qubits: vec![0, 2] }]).unwrap();
    assert_eq!(cmz, logical_cz(&gamma[0], &gamma[2]).unwrap());
    // A second block shifts every emitted index by the outer code's length.
    let outer_two = CodeAbstraction::<W>::new(&four_two_two(), vec![]).unwrap();
    let gamma_two = outer_two.basis().homology();
    let shifted =
        encode_program::<W, f64>(&outer_two, &[GateOp::Cmz { qubits: vec![2, 3] }]).unwrap();
    let expected: Vec<GateOp> = logical_cz(&gamma_two[0], &gamma_two[1])
        .unwrap()
        .into_iter()
        .map(|op| match op {
            GateOp::Z(q) => GateOp::Z(q + 4),
            GateOp::Cz { control, target } => GateOp::Cz {
                control: control + 4,
                target: target + 4,
            },
            other => other,
        })
        .collect();
    assert_eq!(shifted, expected);
    // Across blocks the controlled-phase family is refused like CZ.
    let across = encode_program::<W, f64>(
        &outer_two,
        &[GateOp::Ccz {
            q0: 0,
            q1: 1,
            q2: 2,
        }],
    )
    .expect_err("refused");
    assert!(
        matches!(across.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("across outer blocks")),
        "{across}"
    );
}

#[test]
fn test_an_outer_code_with_no_logical_qubits_is_refused_before_the_modulo() {
    let outer = CodeAbstraction::<W>::new(&no_logical_qubits(), vec![]).unwrap();
    assert_eq!(outer.basis().num_logical_qubits(), 0);
    let err = encode_program::<W, f64>(&outer, &[GateOp::Z(0)]).expect_err("refused");
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("no logical qubits")),
        "{err}"
    );
    let err = concatenated_code::<W, _, _, f64>(
        &four_two_two(),
        &no_logical_qubits(),
        &LogicalGate::Z(0),
    )
    .expect_err("refused");
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("no logical qubits")),
        "{err}"
    );
}
