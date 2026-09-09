/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `QpuSampler` seam.
//!
//! # What there is to test
//!
//! The trait declares no bodies, so it instruments no regions and no coverage number can speak
//! about it. What it asserts is a TYPE-LEVEL contract, and compilation is the oracle:
//!
//!   * `Shots: ShotHistogram`, so an implementation can only return classical outcome counts at
//!     the Kleisli cut — never amplitudes. Dropping that bound would let a sampler hand back a
//!     state vector, which is the boundary the seam exists to hold.
//!   * usable as a generic bound `S: QpuSampler`, never as `dyn`, per the crate's static-dispatch
//!     rule.
//!   * the associated `Calibration` is free, so device metadata is not forced into one shape.
//!
//! The crate ships no adapter, so the only way to exercise the seam is to implement it. The
//! stub below is that implementation: it is the test, not a helper for one.

use deep_causality_quantum::{
    CountHistogram, GateOp, QpuSampler, QuantumCircuit, QuantumError, QuantumErrorEnum,
    ShotHistogram,
};

/// Device metadata of an arbitrary shape, to show `Calibration` constrains nothing.
#[derive(Debug, PartialEq)]
struct Topology {
    name: &'static str,
    couplings: usize,
}

/// A sampler that reports every shot as outcome 0, and refuses a circuit that measures nothing.
struct AlwaysZero {
    couplings: usize,
}

impl QpuSampler for AlwaysZero {
    type Shots = CountHistogram;
    type Calibration = Topology;

    fn sample(&self, circuit: &QuantumCircuit, shots: u64) -> Result<Self::Shots, QuantumError> {
        if circuit.measure().is_empty() {
            return Err(QuantumError::DimensionMismatch(
                "a sampled circuit must measure at least one qubit".into(),
            ));
        }
        let mut hist = CountHistogram::new(circuit.measure().len())?;
        hist.record_n(0, shots)?;
        Ok(hist)
    }

    fn calibration(&self) -> Self::Calibration {
        Topology {
            name: "always-zero",
            couplings: self.couplings,
        }
    }
}

/// The seam as it is meant to be consumed: a generic bound, resolved statically.
fn run<S: QpuSampler>(
    sampler: &S,
    circuit: &QuantumCircuit,
    shots: u64,
) -> Result<u64, QuantumError> {
    // `Shots: ShotHistogram` is what makes `total()` callable here without naming the concrete
    // type. Without the bound this function would not compile.
    sampler.sample(circuit, shots).map(|h| h.total())
}

fn circuit(measure: Vec<usize>) -> QuantumCircuit {
    QuantumCircuit::new(2, vec![GateOp::H(0), GateOp::X(1)], measure).expect("a valid circuit")
}

#[test]
fn test_the_seam_is_reachable_through_a_generic_bound() {
    let sampler = AlwaysZero { couplings: 3 };
    let c = circuit(vec![0, 1]);

    assert_eq!(run(&sampler, &c, 128).expect("the stub samples"), 128);

    // And the histogram is the classical count map, reached through the trait rather than the
    // concrete type.
    let shots = sampler.sample(&c, 128).expect("the stub samples");
    assert_eq!(shots.num_bits(), 2);
    assert_eq!(shots.count(0), 128);
    assert_eq!(shots.count(1), 0);
    assert_eq!(shots.entries(), vec![(0, 128)]);
}

#[test]
fn test_calibration_is_surfaced_unconstrained() {
    let sampler = AlwaysZero { couplings: 7 };
    assert_eq!(
        sampler.calibration(),
        Topology {
            name: "always-zero",
            couplings: 7
        }
    );
}

#[test]
fn test_a_failing_sampler_reports_a_typed_error() {
    // The seam is fallible, and the failure travels as a `QuantumError` rather than a panic or
    // an empty histogram that a caller would read as zero counts.
    let sampler = AlwaysZero { couplings: 1 };
    let err = run(&sampler, &circuit(Vec::new()), 128).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)),
        "expected DimensionMismatch, got {err:?}"
    );
}

#[test]
fn test_zero_shots_is_an_empty_histogram_not_an_error() {
    let sampler = AlwaysZero { couplings: 1 };
    let shots = sampler
        .sample(&circuit(vec![0]), 0)
        .expect("no shots is valid");
    assert_eq!(shots.total(), 0);
    assert!(shots.entries().is_empty());
}
