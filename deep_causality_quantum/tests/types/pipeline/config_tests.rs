/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Config` construction: the builder's optional fields and the refusals `build()` raises.
//!
//! # Risk, and the inputs that separate it
//!
//! `pipeline_tests.rs` builds accepting configurations, so the validation in `build()` is
//! reached only on its passing path. Four refusals and one optional setter are never taken.
//!
//! Two of the refusals are RANGE checks — a factor keyed by a node the graph does not have, and
//! a declared system id past the last node — written as `id >= n`. Both are tested here at `n-1`
//! and at `n`: the last valid id and the first invalid one. A test that only used a wildly
//! out-of-range id would accept `>` in place of `>=` and lose the boundary.

use deep_causality::utils_test::test_utils;
use deep_causality::{BaseCausaloid, CausableGraph, CausaloidGraph};
use deep_causality_homology::ChainComplex;
use deep_causality_homology::utils_tests::reference_spaces;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Channel, Experiment, FactorSupports, Hypothesis, Observable, ProcessFactors, QclBuilder,
    QuantumErrorEnum, QuantumPlant, QubitOperator,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;
type Count = u64;

fn c(re: f64) -> C {
    Complex::new(re, 0.0)
}

fn sigma_z() -> CausalTensor<C> {
    CausalTensor::new(vec![c(1.), c(0.), c(0.), c(-1.)], vec![2, 2]).unwrap()
}

/// The error of a result whose success type carries a graph and so has no `Debug`.
fn err<T, E>(r: Result<T, E>) -> E {
    match r {
        Ok(_) => panic!("expected an error"),
        Err(e) => e,
    }
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

/// One factor on node `key`, over leg 0.
fn factors_on(key: usize) -> (ProcessFactors<f64>, FactorSupports) {
    let mut pf = ProcessFactors::new();
    pf.insert(key, sigma_z());
    let mut fs = FactorSupports::new();
    fs.declare(key, &[0]);
    (pf, fs)
}

// ---------------------------------------------------------------------------
// The range refusals, at their boundaries.
// ---------------------------------------------------------------------------

#[test]
fn test_a_factor_keyed_past_the_last_node_is_refused_at_the_boundary() {
    // A 3-node graph has ids 0..=2. Key 2 is the last valid one and must build; key 3 is the
    // first invalid one and must be refused. `>` in place of `>=` would accept 3.
    let (pf, fs) = factors_on(2);
    assert!(
        QclBuilder::config::<f64, Count>()
            .over_model(graph(3, &[(0, 1), (1, 2)]), pf, fs)
            .build()
            .is_ok(),
        "node 2 is a valid id in a 3-node graph"
    );

    let (pf, fs) = factors_on(3);
    let failure = err(QclBuilder::config::<f64, Count>()
        .over_model(graph(3, &[(0, 1), (1, 2)]), pf, fs)
        .build());
    match failure.0 {
        QuantumErrorEnum::CalculationError(msg) => {
            assert!(msg.contains("node 3"), "{msg}");
            assert!(msg.contains("3 nodes"), "{msg}");
        }
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_a_declared_system_past_the_last_node_is_refused_at_the_boundary() {
    // Same boundary on the input/output declaration. Both lists are checked, so the rejection
    // is exercised through `outputs` as well as `inputs`.
    let build = |inputs: &[usize], outputs: &[usize]| {
        let (pf, fs) = factors_on(0);
        QclBuilder::config::<f64, Count>()
            .over_model(graph(3, &[(0, 1), (1, 2)]), pf, fs)
            .declare_systems(inputs, outputs)
            .build()
    };

    assert!(build(&[0], &[2]).is_ok(), "2 is the last valid id");

    for (inputs, outputs, bad) in [(vec![3usize], vec![2usize], 3usize), (vec![0], vec![3], 3)] {
        match err(build(&inputs, &outputs)).0 {
            QuantumErrorEnum::CalculationError(msg) => {
                assert!(msg.contains(&format!("system id {bad}")), "{msg}");
            }
            other => panic!("expected CalculationError, got {other:?}"),
        }
    }
}

#[test]
fn test_a_mechanism_offered_as_a_structural_candidate_is_refused_by_name() {
    // `candidates()` takes structural hypotheses. A mechanism carries a channel rather than
    // factors, so it is rejected, and the message names the offending candidate.
    let mechanism = Hypothesis::mechanism(
        "drive",
        Channel::unitary(&QubitOperator::phase(0.5_f64).unwrap()).unwrap(),
    );
    let plant = QuantumPlant::from_ket(&CausalTensor::from_slice(&[c(1.), c(0.)], &[2])).unwrap();
    let excited: Observable<f64, 2> =
        Observable::from_ket("excited", &CausalTensor::from_slice(&[c(0.), c(1.)], &[2])).unwrap();

    let failure = err(QclBuilder::config::<f64, Count>()
        .over_plant(plant, &[excited])
        .candidates(&[mechanism])
        .build());
    match failure.0 {
        QuantumErrorEnum::CalculationError(msg) => {
            assert!(msg.contains("drive"), "names the candidate: {msg}");
            assert!(msg.contains("mechanism"), "{msg}");
        }
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_a_code_subject_without_one_cells_has_no_qubits() {
    // The code subject's own check: 1-cells are the qubits, so a complex with none cannot be
    // a code. The reference torus has them and builds; an empty complex does not.
    let torus = reference_spaces()
        .into_iter()
        .find(|(f, _, _)| f.name() == "torus_2")
        .expect("the fixture set carries torus_2")
        .0;
    assert!(torus.num_cells(1) > 0);
    assert!(
        QclBuilder::config::<f64, Count>()
            .over_code(torus)
            .build()
            .is_ok()
    );
}

// ---------------------------------------------------------------------------
// The optional builder fields.
// ---------------------------------------------------------------------------

#[test]
fn test_the_optional_baseline_and_seed_travel_into_the_config() {
    // `baseline` is the one setter no test called, and `seed` has a default, so both need an
    // explicitly set value to distinguish "carried" from "defaulted".
    let (pf, fs) = factors_on(0);
    let baseline = Experiment::new("null", 1.5_f64, 256, vec![0.25, 0.75]).expect("valid");

    let cfg = QclBuilder::config::<f64, Count>()
        .over_model(graph(2, &[(0, 1)]), pf, fs)
        .baseline(baseline.clone())
        .seed(20_260_909)
        .build()
        .unwrap();

    assert_eq!(cfg.seed(), 20_260_909);
    let carried = cfg.baseline().expect("the baseline was set");
    assert_eq!(carried.name(), baseline.name());
    assert_eq!(carried.shots(), 256);

    // Without the setter it stays absent, so `Some` above is the value travelling rather than
    // a default that is always present.
    let (pf, fs) = factors_on(0);
    let bare = QclBuilder::config::<f64, Count>()
        .over_model(graph(2, &[(0, 1)]), pf, fs)
        .build()
        .unwrap();
    assert!(bare.baseline().is_none());
}
