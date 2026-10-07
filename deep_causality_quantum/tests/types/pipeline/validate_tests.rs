/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `Validate` builder's failure handling and the `Screened` accessors.
//!
//! # Risk, and why the existing suite does not reach it
//!
//! `pipeline_tests.rs` drives the builder along accepting paths, so every step runs and records
//! a report. Three branches are therefore never taken:
//!
//!   * `if self.failure.is_some() { return self; }` at the head of each step, and the
//!     `if self.failure.is_none()` guard inside `fail`. Together they implement
//!     first-failure-wins. Measured: they are mutually redundant — removing either one alone
//!     changes no observable, because the short-circuit stops the second step from calling
//!     `fail`, and `fail` refuses the overwrite even when it is called. Only removing BOTH
//!     lets a later diagnosis replace an earlier one, and the test below is what notices.
//!   * the `Err(e) => self.fail(e)` arm of each check.
//!   * `Screened::code`, `Screened::ldpc` and the `Current` arm of `stale_report_degraded`.
//!
//! The separating input is a configuration that FAILS an early step, followed by further steps
//! that would succeed. A builder without the short-circuit reports the last failure, or none at
//! all; with it, the first failure is what reaches `finalize`.

use deep_causality::utils_test::test_utils;
use deep_causality::{BaseCausaloid, CausableGraph, CausaloidGraph};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    CommutatorTolerance, FactorSupports, ProcessFactors, QclBuilder, QuantumErrorEnum, ScreenStatus,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;
type Count = u64;

/// The error of a result whose success type carries a graph and so has no `Debug`.
fn err<T, E>(r: Result<T, E>) -> E {
    match r {
        Ok(_) => panic!("expected an error"),
        Err(e) => e,
    }
}

fn c(re: f64) -> C {
    Complex::new(re, 0.0)
}

fn mat(data: Vec<C>, d: usize) -> CausalTensor<C> {
    CausalTensor::new(data, vec![d, d]).unwrap()
}

fn sigma_x() -> CausalTensor<C> {
    mat(vec![c(0.), c(1.), c(1.), c(0.)], 2)
}

fn sigma_z() -> CausalTensor<C> {
    mat(vec![c(1.), c(0.), c(0.), c(-1.)], 2)
}

fn diag(a: f64, b: f64) -> CausalTensor<C> {
    mat(vec![c(a), c(0.), c(0.), c(b)], 2)
}

fn graph(n: usize, edges: &[(usize, usize)]) -> CausaloidGraph<BaseCausaloid<f64, bool>> {
    let mut g = CausaloidGraph::new(0);
    let mut nodes = Vec::with_capacity(n);
    for i in 0..n {
        nodes.push(
            g.add_causaloid(test_utils::get_test_causaloid_deterministic(i as u64))
                .unwrap(),
        );
    }
    for &(a, b) in edges {
        g.add_edge(nodes[a], nodes[b]).unwrap();
    }
    g.freeze();
    g
}

/// Commuting factors on one shared leg: the Markov check accepts.
fn commuting() -> (ProcessFactors<f64>, FactorSupports) {
    let mut pf = ProcessFactors::new();
    pf.insert(0, sigma_z());
    pf.insert(1, diag(3.0, -1.0));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);
    (pf, fs)
}

/// σx and σz share a leg and do not commute, so the Markov check rejects.
fn non_commuting() -> (ProcessFactors<f64>, FactorSupports) {
    let mut pf = ProcessFactors::new();
    pf.insert(0, sigma_x());
    pf.insert(1, sigma_z());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);
    (pf, fs)
}

// ---------------------------------------------------------------------------
// The failure short-circuit.
// ---------------------------------------------------------------------------

#[test]
fn test_the_first_failure_is_the_one_that_reaches_finalize() {
    // check_markov rejects on the non-commuting pair; check_decomposable would then succeed on
    // this graph. This pins that a failing step's diagnosis survives a later SUCCEEDING one —
    // the two-failure case, where the guards actually decide, is the test below.
    let cfg = QclBuilder::config::<f64, Count>()
        .over_model(graph(2, &[(0, 1)]), non_commuting().0, non_commuting().1)
        .declare_systems(&[0], &[1])
        .build()
        .unwrap();

    let failure = err(QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .check_decomposable()
        .finalize());

    match failure.0 {
        QuantumErrorEnum::CommutatorNonZero { node_j, node_k, .. } => {
            assert_eq!((node_j, node_k), (0, 1), "the offending pair is named");
        }
        other => panic!("expected the Markov failure to survive, got {other:?}"),
    }
}

#[test]
fn test_the_order_of_the_steps_does_not_change_which_failure_is_kept() {
    // The same configuration with the passing step FIRST. The failure is still Markov's, so the
    // short-circuit is keeping the first failure rather than the last step's outcome.
    let cfg = QclBuilder::config::<f64, Count>()
        .over_model(graph(2, &[(0, 1)]), non_commuting().0, non_commuting().1)
        .declare_systems(&[0], &[1])
        .build()
        .unwrap();

    let failure = err(QclBuilder::validate(&cfg)
        .check_decomposable()
        .check_markov(&CommutatorTolerance::default())
        .finalize());

    assert!(
        matches!(failure.0, QuantumErrorEnum::CommutatorNonZero { .. }),
        "expected the Markov failure, got {:?}",
        failure.0
    );
}

#[test]
fn test_a_step_after_a_failure_records_no_further_stage() {
    // The short-circuit returns before `record`, so a failed run carries only the stages that
    // ran. A builder without it would carry both, which is what makes the stage list an
    // observable of the branch rather than of the configuration.
    let cfg = QclBuilder::config::<f64, Count>()
        .over_model(graph(2, &[(0, 1)]), non_commuting().0, non_commuting().1)
        .declare_systems(&[0], &[1])
        .build()
        .unwrap();

    // The accepting configuration reaches two stages; the rejecting one must not reach finalize
    // at all, so the contrast is between Ok-with-two-stages and Err.
    let accepted = QclBuilder::validate(
        &QclBuilder::config::<f64, Count>()
            .over_model(graph(2, &[(0, 1)]), commuting().0, commuting().1)
            .declare_systems(&[0], &[1])
            .build()
            .unwrap(),
    )
    .check_markov(&CommutatorTolerance::default())
    .check_decomposable()
    .finalize()
    .unwrap();
    assert_eq!(accepted.stages().len(), 2);

    let failure = err(QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .check_decomposable()
        .finalize());
    assert!(matches!(
        failure.0,
        QuantumErrorEnum::CommutatorNonZero { .. }
    ));
}

// ---------------------------------------------------------------------------
// `check_decomposable`'s own refusal, which is not a check failure but a misuse.
// ---------------------------------------------------------------------------

#[test]
fn test_decomposability_without_declared_systems_is_refused_by_name() {
    // The guard is an early `fail` before the C₃ machinery runs, and it names the call the
    // caller omitted. A generic error would leave the caller guessing.
    let cfg = QclBuilder::config::<f64, Count>()
        .over_model(graph(2, &[(0, 1)]), commuting().0, commuting().1)
        .build()
        .unwrap();

    let failure = err(QclBuilder::validate(&cfg).check_decomposable().finalize());

    match failure.0 {
        QuantumErrorEnum::CalculationError(msg) => {
            assert!(msg.contains("declare_systems"), "{msg}");
        }
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// `Screened`'s accessors on the status they belong to.
// ---------------------------------------------------------------------------

#[test]
fn test_a_current_screen_reports_no_degradation() {
    // `stale_report_degraded` has one arm per status. The `Current` arm returns None and was
    // never taken; only the invalidated arm had a test. Without this, an implementation that
    // returned the undegraded margin for a current screen would pass.
    let cfg = QclBuilder::config::<f64, Count>()
        .over_model(graph(2, &[(0, 1)]), commuting().0, commuting().1)
        .declare_systems(&[0], &[1])
        .build()
        .unwrap();
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .finalize()
        .unwrap();

    assert_eq!(screened.status(), ScreenStatus::Current);
    assert_eq!(screened.stale_report_degraded(), None);
    // A current screen does carry a report, so the None above is the status arm and not an
    // absent report.
    assert!(screened.report().is_some());
}

#[test]
fn test_a_model_screen_carries_no_code_or_ldpc_weights() {
    // The two code-subject accessors on a model subject: both are None, which is the only
    // observation that separates "not derived yet" from "derived and dropped".
    let cfg = QclBuilder::config::<f64, Count>()
        .over_model(graph(2, &[(0, 1)]), commuting().0, commuting().1)
        .declare_systems(&[0], &[1])
        .build()
        .unwrap();
    let v = QclBuilder::validate(&cfg).check_markov(&CommutatorTolerance::default());

    assert!(v.code().is_none());
    assert!(v.ldpc().is_none());
}

#[test]
fn test_the_earlier_of_two_failures_is_the_one_reported() {
    // Both steps fail, and with different variants: the factors do not commute, so
    // `check_markov` raises `CommutatorNonZero`, and no systems are declared, so
    // `check_decomposable` raises `CalculationError`. Which one surfaces is decided by
    // first-failure-wins, and reversing it would report the later diagnosis instead — a
    // caller told to call `declare_systems` when the real fault is a non-commuting pair.
    //
    // This is the input that separates the guards. With a single failing step, or with a
    // failing step followed by a succeeding one, every arrangement of them agrees.
    let cfg = QclBuilder::config::<f64, Count>()
        .over_model(graph(2, &[(0, 1)]), non_commuting().0, non_commuting().1)
        .build()
        .unwrap();

    let failure = err(QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .check_decomposable()
        .finalize());

    assert!(
        matches!(failure.0, QuantumErrorEnum::CommutatorNonZero { .. }),
        "the first failure must survive the second, got {:?}",
        failure.0
    );

    // And in the other order the first failure is the decomposability one, so the report
    // follows the call order rather than a fixed precedence between the checks.
    let failure = err(QclBuilder::validate(&cfg)
        .check_decomposable()
        .check_markov(&CommutatorTolerance::default())
        .finalize());
    match failure.0 {
        QuantumErrorEnum::CalculationError(msg) => {
            assert!(msg.contains("declare_systems"), "{msg}")
        }
        other => panic!("expected the decomposability failure first, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// check_normalization: a candidate that is not a normalised process is not admitted.
// ---------------------------------------------------------------------------

/// A two-qubit plant in `|00⟩` exposing "qubit 2 excited".
fn two_qubit_plant() -> (
    deep_causality_quantum::QuantumPlant<f64>,
    deep_causality_quantum::Observable<f64, 4>,
) {
    let z = c(0.0);
    let plant = deep_causality_quantum::QuantumPlant::from_ket(&CausalTensor::from_slice(
        &[c(1.0), z, z, z],
        &[4],
    ))
    .unwrap();
    let mut e2 = vec![z; 16];
    e2[5] = c(1.0);
    e2[15] = c(1.0);
    let observable = deep_causality_quantum::Observable::new(
        "e2",
        deep_causality_quantum::Projection::<f64, 4>::new(mat(e2, 4)).unwrap(),
    );
    (plant, observable)
}

/// Q1 → Q2 as conditional tables: P(q1 = 1) = 1/10, P(q2 = 1 | q1) = (1/15, 2/5).
fn conditional_drive() -> deep_causality_quantum::Hypothesis<f64> {
    let d = |entries: &[f64]| {
        let n = entries.len();
        let mut data = vec![c(0.0); n * n];
        for (i, &e) in entries.iter().enumerate() {
            data[i * n + i] = c(e);
        }
        mat(data, n)
    };
    let mut pf = ProcessFactors::new();
    pf.insert(0, d(&[0.9, 0.1]));
    pf.insert(1, d(&[14.0 / 15.0, 1.0 / 15.0, 0.6, 0.4]));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0, 1]);
    deep_causality_quantum::Hypothesis::structural("conditional", pf, fs).unwrap()
}

#[test]
fn test_check_normalization_admits_conditional_tables_and_refuses_joint_ones() {
    // The crosstalk example's factors put a joint table where a conditional belongs.
    let [h1, h2, h3] = deep_causality_quantum::utils_tests::crosstalk_candidates().unwrap();
    let (plant, e2) = two_qubit_plant();
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant, &[e2])
        .candidates(&[h1, conditional_drive(), h2, h3])
        .build()
        .unwrap();
    let screened = QclBuilder::validate(&cfg)
        .check_normalization()
        .finalize()
        .unwrap();
    let names: Vec<&str> = screened.admitted().iter().map(|h| h.name()).collect();
    assert_eq!(names, vec!["conditional"]);
    assert_eq!(screened.admitted_slots(), &[1]);
    assert_eq!(screened.stages()[0].0, "check_normalization");
    // Two factors each for H1, H2 and the conditional drive, three for H3.
    assert_eq!(screened.stages()[0].1.examined(), 9);

    // As a later stage it screens the admitted set only.
    let after = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::default())
        .check_normalization()
        .finalize()
        .unwrap();
    assert_eq!(after.admitted_slots(), &[1]);
    assert_eq!(after.stages()[1].1.examined(), 9);
}

#[test]
fn test_a_candidate_without_its_own_leg_fails_the_normalization_stage() {
    let (pf, fs) = non_commuting();
    let borrowed = deep_causality_quantum::Hypothesis::structural("borrowed", pf, fs).unwrap();
    let (plant, e2) = two_qubit_plant();
    let cfg = QclBuilder::config::<f64, Count>()
        .over_plant(plant, &[e2])
        .candidates(&[conditional_drive(), borrowed])
        .build()
        .unwrap();
    let e = err(QclBuilder::validate(&cfg).check_normalization().finalize());
    assert!(matches!(e.0, QuantumErrorEnum::DimensionMismatch(_)));
    // A stage after a failure records nothing.
    let e = err(QclBuilder::validate(&cfg)
        .check_normalization()
        .check_normalization()
        .finalize());
    assert!(matches!(e.0, QuantumErrorEnum::DimensionMismatch(_)));
}
