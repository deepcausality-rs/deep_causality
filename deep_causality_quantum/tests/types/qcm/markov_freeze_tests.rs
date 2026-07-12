/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::{CausableGraph, CausalityGraphError, CausaloidGraph};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    CommutatorTolerance, FactorSupports, ProcessFactors, QuantumError, QuantumErrorEnum,
    freeze_quantum, quantum_markov_check,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn c(re: f64, im: f64) -> C {
    Complex::new(re, im)
}

fn mat(data: Vec<C>, d: usize) -> CausalTensor<C> {
    CausalTensor::new(data, vec![d, d]).unwrap()
}

fn sigma_x() -> CausalTensor<C> {
    mat(vec![c(0., 0.), c(1., 0.), c(1., 0.), c(0., 0.)], 2)
}

fn sigma_z() -> CausalTensor<C> {
    mat(vec![c(1., 0.), c(0., 0.), c(0., 0.), c(-1., 0.)], 2)
}

fn diag(a: f64, b: f64) -> CausalTensor<C> {
    mat(vec![c(a, 0.), c(0., 0.), c(0., 0.), c(b, 0.)], 2)
}

fn kron(a: &CausalTensor<C>, b: &CausalTensor<C>) -> CausalTensor<C> {
    a.kronecker(b).unwrap()
}

fn two_node_graph() -> CausaloidGraph<deep_causality::BaseCausaloid<f64, bool>> {
    let mut g = CausaloidGraph::new(0);
    let n0 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(0))
        .unwrap();
    let n1 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .unwrap();
    g.add_edge(n0, n1).unwrap();
    g
}

// =============================================================================
// The pure check (no graph): commuting passes, non-commuting names the pair.
// =============================================================================

#[test]
fn test_check_commuting_single_leg_passes() {
    // Two diagonal factors on the same leg commute.
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_z());
    pf.insert(1, diag(3.0, -1.0));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);

    let report = quantum_markov_check(&pf, &fs, &CommutatorTolerance::default()).unwrap();
    assert_eq!(report.tested_pairs(), 1);
    assert!(report.checks[0].commutes);
    assert!(report.worst_margin().unwrap() <= 1.0);
}

#[test]
fn test_check_noncommuting_names_the_pair() {
    // σx and σz on the same leg do not commute.
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_x());
    pf.insert(1, sigma_z());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);

    let err = quantum_markov_check(&pf, &fs, &CommutatorTolerance::default()).unwrap_err();
    match err.0 {
        QuantumErrorEnum::CommutatorNonZero { node_j, node_k, .. } => {
            assert_eq!((node_j, node_k), (0, 1));
        }
        other => panic!("expected CommutatorNonZero, got {:?}", other),
    }
}

#[test]
fn test_check_disjoint_supports_impose_no_obligation() {
    // σx and σz would not commute, but on disjoint legs no commutator is formed.
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_x());
    pf.insert(1, sigma_z());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[7]);

    let report = quantum_markov_check(&pf, &fs, &CommutatorTolerance::default()).unwrap();
    assert_eq!(report.tested_pairs(), 0);
}

#[test]
fn test_check_multi_leg_overlap_commuting() {
    // σz⊗σz on {0,1} and σz⊗σz on {1,2} share leg 1; both diagonal → commute.
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, kron(&sigma_z(), &sigma_z()));
    pf.insert(1, kron(&sigma_z(), &sigma_z()));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0, 1]);
    fs.declare(1, &[1, 2]);

    let report = quantum_markov_check(&pf, &fs, &CommutatorTolerance::default()).unwrap();
    assert_eq!(report.tested_pairs(), 1);
    assert!(report.checks[0].commutes);
}

#[test]
fn test_check_multi_leg_overlap_noncommuting() {
    // σz(0)⊗σx(1) on {0,1} and σz(1)⊗σx(2) on {1,2}: leg 1 carries σx vs σz,
    // which do not commute, so the embedded operators do not commute.
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, kron(&sigma_z(), &sigma_x()));
    pf.insert(1, kron(&sigma_z(), &sigma_x()));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0, 1]);
    fs.declare(1, &[1, 2]);

    let err = quantum_markov_check(&pf, &fs, &CommutatorTolerance::default()).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::CommutatorNonZero { .. }));
}

// =============================================================================
// The freeze integration: hook wiring, rollback, structured-error recovery.
// =============================================================================

#[test]
fn test_freeze_commuting_model_freezes() {
    let mut g = two_node_graph();
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_z());
    pf.insert(1, diag(2.0, 5.0));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);

    let report =
        freeze_quantum(&mut g, &[], &pf, &fs, &CommutatorTolerance::default(), None).unwrap();
    assert!(g.is_frozen());
    assert_eq!(report.tested_pairs(), 1);
}

#[test]
fn test_freeze_noncommuting_model_aborts_and_rolls_back() {
    let mut g = two_node_graph();
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_x());
    pf.insert(1, sigma_z());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);

    let err =
        freeze_quantum(&mut g, &[], &pf, &fs, &CommutatorTolerance::default(), None).unwrap_err();
    // The structured error survives the CausalityGraphError bridge and names the pair.
    match err.0 {
        QuantumErrorEnum::CommutatorNonZero { node_j, node_k, .. } => {
            assert_eq!((node_j, node_k), (0, 1));
        }
        other => panic!("expected CommutatorNonZero, got {:?}", other),
    }
    // Rolled back to the dynamic state.
    assert!(!g.is_frozen());
}

#[test]
fn test_from_quantum_error_bridge_preserves_message() {
    let qe = QuantumError::CommutatorNonZero(2, 5, "detail".into());
    let ce: CausalityGraphError = qe.into();
    let msg = format!("{}", ce);
    assert!(msg.contains("nodes 2 and 5"), "unexpected: {}", msg);
}

#[test]
fn test_freeze_shape_mismatch_reported_as_itself() {
    // A factor whose dim disagrees with its support fails validation up front,
    // not as a downstream freeze error.
    let mut g = two_node_graph();
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_x()); // 2x2
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0, 1]); // implies dim 4
    let err =
        freeze_quantum(&mut g, &[], &pf, &fs, &CommutatorTolerance::default(), None).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    assert!(!g.is_frozen());
}

#[test]
fn test_tolerance_admits_tiny_numerical_noise() {
    // A commutator at the machine-noise floor is admitted by the depth-aware
    // tolerance; the recorded margin is ≤ 1.
    let mut pf = ProcessFactors::<f64>::new();
    // Two nearly-diagonal factors with a 1e-15 off-diagonal perturbation.
    pf.insert(
        0,
        mat(vec![c(1., 0.), c(1e-15, 0.), c(1e-15, 0.), c(-1., 0.)], 2),
    );
    pf.insert(1, diag(3.0, -2.0));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);

    let report = quantum_markov_check(&pf, &fs, &CommutatorTolerance::default()).unwrap();
    assert!(report.checks[0].commutes);
    assert!(report.checks[0].margin <= 1.0);
}

// =============================================================================
// Tolerance builders and zero-threshold margin branches (llvm-cov gap closure).
// =============================================================================

#[test]
fn test_commutator_tolerance_builder_chain_admits_with_generous_budget() {
    // A 0.05 off-diagonal is far above machine noise, so the default policy
    // rejects the pair; an explicit forward-error budget on both nodes admits it.
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(
        0,
        mat(vec![c(1., 0.), c(0.05, 0.), c(0.05, 0.), c(-1., 0.)], 2),
    );
    pf.insert(1, diag(3.0, -2.0));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);

    assert!(quantum_markov_check(&pf, &fs, &CommutatorTolerance::default()).is_err());

    let tol = CommutatorTolerance::new()
        .with_safety_factor(8.0)
        .with_unit_roundoff(1e-12)
        .with_budget(0, 10.0)
        .with_budget(1, 10.0);
    let report = quantum_markov_check(&pf, &fs, &tol).unwrap();
    assert!(report.checks[0].commutes);
}

#[test]
fn test_zero_threshold_admits_exactly_commuting_pair() {
    // Two diagonal factors commute exactly (‖[·,·]‖_F = 0), so even a
    // zero-threshold policy admits them, recording a zero margin.
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_z());
    pf.insert(1, diag(3.0, -1.0));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);

    let tol = CommutatorTolerance::default().with_safety_factor(0.0);
    let report = quantum_markov_check(&pf, &fs, &tol).unwrap();
    assert_eq!(report.tested_pairs(), 1);
    assert!(report.checks[0].commutes);
    assert_eq!(report.checks[0].margin, 0.0);
}

#[test]
fn test_zero_threshold_rejects_any_nonzero_commutator() {
    // With a zero threshold, any non-zero commutator (margin = ‖·‖_F > 0) fails.
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_x());
    pf.insert(1, sigma_z());
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);

    let tol = CommutatorTolerance::default().with_safety_factor(0.0);
    let err = quantum_markov_check(&pf, &fs, &tol).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::CommutatorNonZero { .. }));
}

#[test]
fn test_freeze_builtin_check_failure_is_calculation_error() {
    // A valid, commuting quantum layout — but an invalid state-writer index makes
    // the engine's built-in single-writer check fail *before* the quantum hook
    // runs. freeze_quantum must surface that as a CalculationError (no structured
    // quantum error was stashed) and leave the graph unfrozen.
    let mut g = two_node_graph();
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_z());
    pf.insert(1, diag(2.0, 5.0));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);

    // Node index 99 names no node (the graph has 2) → single-writer check errors.
    let err = freeze_quantum(
        &mut g,
        &[99],
        &pf,
        &fs,
        &CommutatorTolerance::default(),
        None,
    )
    .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::CalculationError(_)));
    assert!(!g.is_frozen());
}

// =============================================================================
// C₃-exclusion faithfulness enforced at freeze (spec quantum-markov-freeze).
// =============================================================================

#[test]
fn test_freeze_rejects_c3_structure() {
    // A graph whose input→output reachability is the canonical C₃ (K_{3,3} minus
    // a perfect matching) must be rejected at freeze once the declared structure
    // is supplied. Empty factors make the commutativity check trivially pass, so
    // the abort is attributable to the C₃ faithfulness check.
    let mut g = CausaloidGraph::new(0);
    let nodes: Vec<usize> = (0..6)
        .map(|i| {
            g.add_causaloid(test_utils::get_test_causaloid_deterministic(i))
                .unwrap()
        })
        .collect();
    // inputs {0,1,2} → outputs {3,4,5}; the non-edges form the diagonal matching.
    for (i, o) in [(0, 4), (0, 5), (1, 3), (1, 5), (2, 3), (2, 4)] {
        g.add_edge(nodes[i], nodes[o]).unwrap();
    }

    let pf = ProcessFactors::<f64>::new();
    let fs = FactorSupports::new();
    let inputs = [nodes[0], nodes[1], nodes[2]];
    let outputs = [nodes[3], nodes[4], nodes[5]];

    let err = freeze_quantum(
        &mut g,
        &[],
        &pf,
        &fs,
        &CommutatorTolerance::default(),
        Some((&inputs, &outputs)),
    )
    .unwrap_err();
    assert!(matches!(
        err.0,
        QuantumErrorEnum::NotFaithfullyRepresentable(_)
    ));
    // The structured error survived the CausalityGraphError bridge, and the graph
    // rolled back to the dynamic state.
    assert!(!g.is_frozen());
}

#[test]
fn test_freeze_admits_faithful_structure() {
    // A 2-node chain 0 → 1 cannot contain a C₃; with the structure declared, the
    // commuting model still freezes cleanly.
    let mut g = two_node_graph();
    let mut pf = ProcessFactors::<f64>::new();
    pf.insert(0, sigma_z());
    pf.insert(1, diag(2.0, 5.0));
    let mut fs = FactorSupports::new();
    fs.declare(0, &[0]);
    fs.declare(1, &[0]);
    let inputs = [0usize];
    let outputs = [1usize];

    let report = freeze_quantum(
        &mut g,
        &[],
        &pf,
        &fs,
        &CommutatorTolerance::default(),
        Some((&inputs, &outputs)),
    )
    .unwrap();
    assert!(g.is_frozen());
    assert_eq!(report.tested_pairs(), 1);
}
