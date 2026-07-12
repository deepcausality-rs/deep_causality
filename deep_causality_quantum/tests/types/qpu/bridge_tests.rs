/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qpu")]

use deep_causality_quantum::{
    CountHistogram, GateOp, QpuSampler, QuantumCircuit, ShotHistogram, SimQpu, qpu_effect,
    shots_to_observable, shots_to_qubit_bernoulli,
};

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
fn test_qubit_bernoulli_bridge() {
    let hist = SimQpu::new(0xC0FFEE).sample(&bell(), 4000).unwrap();
    // Qubit 0 is ~50/50 in a Bell state.
    let u = shots_to_qubit_bernoulli(&hist, 0).unwrap();
    let p_true = u.estimate_probability(1000).unwrap();
    assert!((0.4..0.6).contains(&p_true), "p(true) = {}", p_true);
}

#[test]
fn test_bernoulli_bridge_rejects_empty_and_bad_index() {
    let empty = CountHistogram::new(2);
    assert!(shots_to_qubit_bernoulli(&empty, 0).is_err());
    let hist = SimQpu::new(1).sample(&bell(), 10).unwrap();
    assert!(shots_to_qubit_bernoulli(&hist, 2).is_err()); // only 2 measured qubits
}

#[test]
fn test_observable_bridge_parity() {
    // Parity observable: +1 on even-weight outcomes, −1 on odd. A Bell state is
    // all even parity (00, 11) → mean ≈ +1.
    let hist = SimQpu::new(0xBEEF).sample(&bell(), 2000).unwrap();
    let obs = shots_to_observable(&hist, |outcome| {
        if (outcome as u32).count_ones().is_multiple_of(2) {
            1.0
        } else {
            -1.0
        }
    })
    .unwrap();
    let mean = obs.expected_value(1000).unwrap();
    assert!((mean - 1.0).abs() < 1e-9, "parity mean = {}", mean);
}

#[test]
fn test_observable_bridge_rejects_empty() {
    let empty = CountHistogram::new(1);
    assert!(shots_to_observable(&empty, |_| 0.0).is_err());
}

#[test]
fn test_qpu_effect_success_routes_channels() {
    let sim = SimQpu::new(0xC0FFEE);
    let effect = qpu_effect(&sim, &bell(), 500);
    assert!(effect.is_ok());
    // Value channel carries the shot histogram.
    let hist = effect.value().expect("shot histogram on the value channel");
    assert_eq!(hist.total(), 500);
    let (outcome, state, context, logs) = effect.into_parts();
    assert!(outcome.is_ok());
    // State channel carries the requested parameters.
    assert_eq!(state.num_qubits, 2);
    assert_eq!(state.shots, 500);
    assert_eq!(state.num_ops, 2);
    // Context carries the device calibration.
    let cal = context.expect("calibration on the context channel");
    assert_eq!(cal.seed, 0xC0FFEE);
    // Log carries provenance.
    assert!(logs.messages().next().is_some());
}

#[test]
fn test_qpu_effect_failure_routes_to_error_channel() {
    // A 25-qubit circuit is a valid QuantumCircuit but exceeds SimQpu's cap, so
    // the job fails; the error routes to the error channel with the value absent.
    let big = QuantumCircuit::new(25, vec![], vec![0]).unwrap();
    let sim = SimQpu::new(1);
    let effect = qpu_effect(&sim, &big, 10);
    assert!(effect.is_err());
    assert!(effect.value().is_none());
}
