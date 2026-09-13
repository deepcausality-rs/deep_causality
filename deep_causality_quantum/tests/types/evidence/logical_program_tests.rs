/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{GateOp, LogicalProgram};

type FloatType = f64;

fn ops() -> Vec<GateOp> {
    vec![GateOp::H(0), GateOp::S(1), GateOp::Tdg(2)]
}

#[test]
fn test_new_carries_the_program_and_no_phase() {
    let p = LogicalProgram::<FloatType>::new(ops());
    assert_eq!(p.ops(), ops().as_slice());
    assert_eq!(p.global_phase(), None);
    assert_eq!(p.len(), 3);
    assert!(!p.is_empty());
}

#[test]
fn test_empty_program_is_empty_and_has_length_zero() {
    let p = LogicalProgram::<FloatType>::new(Vec::new());
    assert_eq!(p.len(), 0);
    assert!(p.is_empty());
    assert_eq!(p.ops(), &[]);
}

#[test]
fn test_with_global_phase_keeps_the_phase_beside_the_program() {
    // The Table 1 Hadamard phase e^{-iπ/4}: both parts non-zero and of opposite sign.
    let phase = Complex::new(
        (FloatType::from(0.5)).sqrt(),
        -(FloatType::from(0.5)).sqrt(),
    );
    let p = LogicalProgram::with_global_phase(ops(), phase);
    assert_eq!(p.global_phase(), Some(&phase));
    assert_eq!(p.len(), 3);
}

#[test]
fn test_global_phase_records_zero_and_negative_real_phases_distinctly() {
    // A zero phase is a recorded phase, not an absent one: `Some(0)` must not read as `None`.
    let zero = LogicalProgram::with_global_phase(ops(), Complex::new(0.0, 0.0));
    assert_eq!(zero.global_phase(), Some(&Complex::new(0.0, 0.0)));
    assert_ne!(zero.global_phase(), None);

    let minus_one = LogicalProgram::with_global_phase(ops(), Complex::new(-1.0, 0.0));
    assert_eq!(minus_one.global_phase(), Some(&Complex::new(-1.0, 0.0)));
    // The two phases differ, so the field is read and not defaulted.
    assert_ne!(zero.global_phase(), minus_one.global_phase());
}

#[test]
fn test_into_ops_returns_the_program_and_drops_the_phase() {
    let p = LogicalProgram::with_global_phase(ops(), Complex::new(0.0, 1.0));
    assert_eq!(p.into_ops(), ops());
}

#[test]
fn test_from_vec_builds_a_program_with_no_phase() {
    let p: LogicalProgram<FloatType> = ops().into();
    assert_eq!(p.ops(), ops().as_slice());
    assert_eq!(p.global_phase(), None);
    assert_eq!(p, LogicalProgram::new(ops()));
}

#[test]
fn test_default_is_the_empty_program_with_no_phase() {
    let p = LogicalProgram::<FloatType>::default();
    assert!(p.is_empty());
    assert_eq!(p.global_phase(), None);
    assert_eq!(p, LogicalProgram::new(Vec::new()));
}

#[test]
fn test_a_phase_distinguishes_two_otherwise_equal_programs() {
    let plain = LogicalProgram::<FloatType>::new(ops());
    let phased = LogicalProgram::with_global_phase(ops(), Complex::new(0.0, 1.0));
    assert_ne!(plain, phased);
    assert_eq!(plain.ops(), phased.ops());
}
