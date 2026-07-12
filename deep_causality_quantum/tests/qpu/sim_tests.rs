/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qpu")]

use deep_causality_quantum::{GateOp, QpuSampler, QuantumCircuit, ShotHistogram, SimQpu};

fn bell() -> QuantumCircuit {
    QuantumCircuit::new(
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
    .unwrap()
}

#[test]
fn test_bell_counts_sum_to_shots_and_are_correlated() {
    let sim = SimQpu::new(0xC0FFEE);
    let hist = sim.sample(&bell(), 2000).unwrap();
    assert_eq!(hist.total(), 2000);
    assert_eq!(hist.num_bits(), 2);
    // A Bell state only produces the correlated outcomes 00 and 11.
    assert_eq!(hist.count(0b01), 0);
    assert_eq!(hist.count(0b10), 0);
    assert!(hist.count(0b00) > 0);
    assert!(hist.count(0b11) > 0);
    assert_eq!(hist.count(0b00) + hist.count(0b11), 2000);
    // Roughly balanced (well within tolerance for 2000 shots).
    let p00 = hist.count(0b00) as f64 / 2000.0;
    assert!((0.4..0.6).contains(&p00), "unbalanced: {}", p00);
}

#[test]
fn test_same_seed_reproduces_same_histogram() {
    let a = SimQpu::new(42).sample(&bell(), 500).unwrap();
    let b = SimQpu::new(42).sample(&bell(), 500).unwrap();
    assert_eq!(a, b);
}

#[test]
fn test_different_seed_can_differ() {
    let a = SimQpu::new(1).sample(&bell(), 500).unwrap();
    let b = SimQpu::new(999_999).sample(&bell(), 500).unwrap();
    // Both are valid Bell histograms; the split differs across seeds.
    assert_ne!(a.count(0b00), b.count(0b00));
}

#[test]
fn test_x_gate_is_deterministic_one() {
    // X|0> = |1>: measuring qubit 0 always yields 1.
    let circuit = QuantumCircuit::new(1, vec![GateOp::X(0)], vec![0]).unwrap();
    let hist = SimQpu::new(7).sample(&circuit, 100).unwrap();
    assert_eq!(hist.count(1), 100);
    assert_eq!(hist.count(0), 0);
}

#[test]
fn test_zero_shots_yields_empty_histogram() {
    let hist = SimQpu::new(7).sample(&bell(), 0).unwrap();
    assert_eq!(hist.total(), 0);
    assert!(hist.entries().is_empty());
}

#[test]
fn test_no_amplitudes_are_exposed() {
    // The ShotHistogram surface is classical counts only — this test documents
    // that the public API offers no amplitude accessor (it would not compile).
    let hist = SimQpu::new(7).sample(&bell(), 10).unwrap();
    let _entries: Vec<(usize, u64)> = hist.entries();
    // (No `hist.amplitude(..)` exists.)
}

#[test]
fn test_gh_z_three_qubit_correlation() {
    // GHZ: H(0), CNOT(0,1), CNOT(1,2) → only 000 and 111.
    let circuit = QuantumCircuit::new(
        3,
        vec![
            GateOp::H(0),
            GateOp::Cnot {
                control: 0,
                target: 1,
            },
            GateOp::Cnot {
                control: 1,
                target: 2,
            },
        ],
        vec![0, 1, 2],
    )
    .unwrap();
    let hist = SimQpu::new(0xABCD).sample(&circuit, 1000).unwrap();
    assert_eq!(hist.count(0b000) + hist.count(0b111), 1000);
}
