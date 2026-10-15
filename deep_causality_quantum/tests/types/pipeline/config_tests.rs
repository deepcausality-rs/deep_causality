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
//! Two of the refusals are MEMBERSHIP checks: a factor keyed by an index that holds no causaloid,
//! and a declared system id that holds none. Indices are permanent, so after a removal an index
//! below `number_nodes()` can hold nothing while an index at or above it holds a causaloid. Each
//! check is tested on three indices: one past the table, a removed one below a live one, and the
//! live one at `number_nodes()`. A check written as `id >= number_nodes()` passes only the first.

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

/// `n` causaloids and the `edges` between them, unfrozen. Returns the graph and the index of
/// each causaloid.
fn build_graph(
    n: usize,
    edges: &[(usize, usize)],
) -> (CausaloidGraph<BaseCausaloid<f64, bool>>, Vec<usize>) {
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
    (g, nodes)
}

/// `build_graph`, frozen.
fn graph(n: usize, edges: &[(usize, usize)]) -> CausaloidGraph<BaseCausaloid<f64, bool>> {
    let (mut g, _) = build_graph(n, edges);
    g.freeze();
    g
}

/// `graph` with the causaloids at `removed` taken out after every node and edge is in place,
/// then frozen. A removed index stays unused, so a live index can sit at or above
/// `number_nodes()` and a removed one below it.
fn graph_with_removed(
    n: usize,
    removed: &[usize],
    edges: &[(usize, usize)],
) -> CausaloidGraph<BaseCausaloid<f64, bool>> {
    let (mut g, nodes) = build_graph(n, edges);
    for &r in removed {
        g.remove_causaloid(nodes[r]).unwrap();
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
// The membership refusals: an index past the table, a removed index, a live index above the
// count.
// ---------------------------------------------------------------------------

#[test]
fn test_a_factor_keyed_past_the_table_is_refused_and_the_last_issued_index_builds() {
    // Three causaloids at 0, 1, 2 and nothing removed. Key 2 holds a causaloid and builds; key 3
    // was never issued and is refused, the message naming node 3.
    let (pf, fs) = factors_on(2);
    assert!(
        QclBuilder::config::<f64, Count>()
            .over_model(graph(3, &[(0, 1), (1, 2)]), pf, fs)
            .build()
            .is_ok(),
        "index 2 holds a causaloid"
    );

    let (pf, fs) = factors_on(3);
    let failure = err(QclBuilder::config::<f64, Count>()
        .over_model(graph(3, &[(0, 1), (1, 2)]), pf, fs)
        .build());
    match failure.0 {
        QuantumErrorEnum::CalculationError(msg) => assert!(msg.contains("node 3"), "{msg}"),
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_a_factor_keyed_by_a_live_index_at_number_nodes_builds() {
    // Four causaloids, index 1 removed: number_nodes() is 3 and index 3 holds a causaloid.
    let g = graph_with_removed(4, &[1], &[(0, 2), (2, 3)]);
    assert_eq!(g.number_nodes(), 3);
    let (pf, fs) = factors_on(3);
    assert!(
        QclBuilder::config::<f64, Count>()
            .over_model(g, pf, fs)
            .build()
            .is_ok(),
        "index 3 holds a causaloid although 3 is not below number_nodes()"
    );
}

#[test]
fn test_a_factor_keyed_by_a_removed_index_below_the_count_is_refused() {
    // Four causaloids, index 1 removed: 1 < number_nodes() == 3, yet index 1 holds nothing.
    let g = graph_with_removed(4, &[1], &[(0, 2), (2, 3)]);
    let (pf, fs) = factors_on(1);
    match err(QclBuilder::config::<f64, Count>()
        .over_model(g, pf, fs)
        .build())
    .0
    {
        QuantumErrorEnum::CalculationError(msg) => assert!(msg.contains("node 1"), "{msg}"),
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_a_declared_system_past_the_table_is_refused_and_the_last_issued_index_builds() {
    // Three causaloids, nothing removed. Both lists are checked, so the refusal is exercised
    // through `outputs` as well as `inputs`.
    let build = |inputs: &[usize], outputs: &[usize]| {
        let (pf, fs) = factors_on(0);
        QclBuilder::config::<f64, Count>()
            .over_model(graph(3, &[(0, 1), (1, 2)]), pf, fs)
            .declare_systems(inputs, outputs)
            .build()
    };

    assert!(build(&[0], &[2]).is_ok(), "index 2 holds a causaloid");

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
fn test_a_declared_system_at_a_live_index_at_number_nodes_builds() {
    // Four causaloids, index 1 removed: number_nodes() is 3 and index 3 holds a causaloid.
    let (pf, fs) = factors_on(0);
    let g = graph_with_removed(4, &[1], &[(0, 2), (2, 3)]);
    assert_eq!(g.number_nodes(), 3);
    assert!(
        QclBuilder::config::<f64, Count>()
            .over_model(g, pf, fs)
            .declare_systems(&[0], &[3])
            .build()
            .is_ok(),
        "index 3 holds a causaloid although 3 is not below number_nodes()"
    );
}

#[test]
fn test_a_declared_system_at_a_removed_index_below_the_count_is_refused() {
    // Four causaloids, index 1 removed: 1 < number_nodes() == 3, yet index 1 holds nothing. Both
    // lists are checked.
    let build = |inputs: &[usize], outputs: &[usize]| {
        let (pf, fs) = factors_on(0);
        QclBuilder::config::<f64, Count>()
            .over_model(graph_with_removed(4, &[1], &[(0, 2), (2, 3)]), pf, fs)
            .declare_systems(inputs, outputs)
            .build()
    };
    for (inputs, outputs) in [(vec![1usize], vec![2usize]), (vec![0], vec![1])] {
        match err(build(&inputs, &outputs)).0 {
            QuantumErrorEnum::CalculationError(msg) => {
                assert!(msg.contains("system id 1"), "{msg}");
            }
            other => panic!("expected CalculationError, got {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// The cycle refusal with a removed index below the cycle.
// ---------------------------------------------------------------------------

#[test]
fn test_a_cycle_through_an_index_above_number_nodes_is_found() {
    // Five causaloids, index 0 removed: number_nodes() is 4. The cycle 2 → 4 → 2 runs through
    // index 4, which is not below the count. A search over 0..number_nodes() never reaches it.
    // The scenario holds only while 4 keeps its causaloid through the freeze and 0 holds none.
    let g = graph_with_removed(5, &[0], &[(2, 4), (4, 2)]);
    assert_eq!(g.number_nodes(), 4);
    assert!(
        g.contains_causaloid(4),
        "index 4 keeps its causaloid through the freeze"
    );
    assert!(!g.contains_causaloid(0), "the removed index 0 stays empty");
    let (pf, fs) = factors_on(2);
    match err(QclBuilder::config::<f64, Count>()
        .over_model(g, pf, fs)
        .build())
    .0
    {
        QuantumErrorEnum::CyclicStructureUnsupported(msg) => {
            assert!(msg.contains("scope"), "{msg}")
        }
        other => panic!("expected CyclicStructureUnsupported, got {other:?}"),
    }
}

#[test]
fn test_a_cycle_past_a_removed_middle_index_is_found() {
    // Four causaloids, index 1 removed: the cycle 0 → 3 → 0 spans the removed index, and 3 is
    // not below number_nodes() == 3. The scenario holds only while 3 keeps its causaloid
    // through the freeze and 1 holds none.
    let g = graph_with_removed(4, &[1], &[(0, 3), (3, 0)]);
    assert_eq!(g.number_nodes(), 3);
    assert!(
        g.contains_causaloid(3),
        "index 3 keeps its causaloid through the freeze"
    );
    assert!(!g.contains_causaloid(1), "the removed index 1 stays empty");
    let (pf, fs) = factors_on(0);
    match err(QclBuilder::config::<f64, Count>()
        .over_model(g, pf, fs)
        .build())
    .0
    {
        QuantumErrorEnum::CyclicStructureUnsupported(_) => {}
        other => panic!("expected CyclicStructureUnsupported, got {other:?}"),
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
