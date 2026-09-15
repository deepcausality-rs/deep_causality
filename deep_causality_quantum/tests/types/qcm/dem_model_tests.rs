/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The detector error model as a classical causal model. Hand values: two mechanisms with
//! probabilities `p₁ = 0.1` flipping `D0, D1` and `p₂ = 0.2` flipping `D1, L0` give
//! `P(000) = 0.72`, `P(110) = 0.08`, `P(011) = 0.18`, `P(101) = 0.02` over `(D0, D1, L0)`; firing
//! the second mechanism once more shifts every string by `011`.

use deep_causality_quantum::utils_tests::memory_dem;
use deep_causality_quantum::{
    DEM_MAX_MECHANISMS, DEM_MAX_VARIABLES, DemModel, DemRoles, Fault, Mechanism, NumericCaps,
    PauliKind, QcModel, QuantumErrorEnum, Query, flips_of,
};

type S = f64;

fn two_mechanisms() -> DemModel {
    DemModel::new(
        vec![
            Mechanism {
                probability: 0.1,
                detectors: vec![1, 0],
                observables: vec![],
            },
            Mechanism {
                probability: 0.2,
                detectors: vec![1],
                observables: vec![0],
            },
        ],
        2,
        1,
    )
    .unwrap()
}

fn weight(model: &DemModel, query: &Query, string: [usize; 3]) -> f64 {
    let m = QcModel::<S>::numeric_query(model, query, &NumericCaps::default()).unwrap();
    m.blocks()
        .get(&(vec![], string.to_vec()))
        .map(|k| {
            let a = k[0].as_slice()[0];
            a.re * a.re + a.im * a.im
        })
        .unwrap_or(0.0)
}

#[test]
fn test_the_distribution_matches_the_hand_values_and_shifts_under_a_fault() {
    let model = two_mechanisms();
    assert_eq!(flips_of(&model.mechanisms()[0]), "D0 D1");
    assert_eq!(flips_of(&model.mechanisms()[1]), "D1 L0");
    let io = Query::Io;
    assert!((weight(&model, &io, [0, 0, 0]) - 0.72).abs() < 1e-12);
    assert!((weight(&model, &io, [1, 1, 0]) - 0.08).abs() < 1e-12);
    assert!((weight(&model, &io, [0, 1, 1]) - 0.18).abs() < 1e-12);
    assert!((weight(&model, &io, [1, 0, 1]) - 0.02).abs() < 1e-12);
    assert_eq!(
        weight(&model, &io, [1, 0, 0]),
        0.0,
        "no subset gives D0 alone"
    );
    let fired = Query::Fault(Fault::new(None, vec![(1, PauliKind::X)]).unwrap());
    assert!((weight(&model, &fired, [0, 1, 1]) - 0.72).abs() < 1e-12);
    assert!((weight(&model, &fired, [0, 0, 0]) - 0.18).abs() < 1e-12);
    assert!((weight(&model, &fired, [1, 0, 1]) - 0.08).abs() < 1e-12);
    let m = QcModel::<S>::numeric_query(&model, &io, &NumericCaps::default()).unwrap();
    assert_eq!((m.d_in(), m.d_out()), (1, 1));
    assert_eq!(m.classical_out(), &[2, 2, 2]);
    assert!(m.classical_in().is_empty());
    let ty = QcModel::<S>::query_type(&model, &io).unwrap();
    assert_eq!(ty.classical_out, vec![0, 1, 2]);
    assert!(ty.quantum_in.is_empty() && ty.quantum_out.is_empty());
    assert!(QcModel::<S>::is_classical(&model));
    assert_eq!(QcModel::<S>::wire_cardinality(&model, 2), Some(2));
    assert_eq!(QcModel::<S>::wire_cardinality(&model, 3), None);
    assert!(
        QcModel::<S>::query_wire_map(&model, &fired)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn test_the_induced_dag_has_one_edge_per_flip() {
    let model = two_mechanisms();
    let dag = QcModel::<S>::induced_dag(&model);
    assert_eq!(
        dag.num_vertices(),
        5,
        "two mechanisms, two detectors, one observable"
    );
    assert_eq!(dag.num_edges(), 4);
    assert_eq!(dag.parents(2), vec![0], "D0 has mechanism 0 only");
    assert_eq!(dag.parents(3), vec![0, 1], "D1 has both");
    assert_eq!(dag.parents(4), vec![1], "L0 has mechanism 1");
    assert!(dag.parents(0).is_empty() && dag.parents(1).is_empty());
}

#[test]
fn test_queries_other_than_io_and_x_faults_are_refused_by_name() {
    let model = two_mechanisms();
    let caps = NumericCaps::default();
    let open = QcModel::<S>::numeric_query(&model, &Query::Open(vec![0]), &caps).unwrap_err();
    assert!(matches!(open.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("open")));
    let z = Query::Fault(Fault::new(None, vec![(0, PauliKind::Z)]).unwrap());
    let err = QcModel::<S>::numeric_query(&model, &z, &caps).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("must be X")));
    let placed = Query::Fault(Fault::new(Some(0), vec![(0, PauliKind::X)]).unwrap());
    let err = QcModel::<S>::query_type(&model, &placed).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("latent")));
    let wide = Query::Fault(Fault::new(None, vec![(5, PauliKind::X)]).unwrap());
    let err = QcModel::<S>::numeric_query(&model, &wide, &caps).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("mechanism 5"))
    );
}

#[test]
fn test_construction_errors() {
    let bad_p = DemModel::new(
        vec![Mechanism {
            probability: 1.5,
            detectors: vec![0],
            observables: vec![],
        }],
        1,
        0,
    )
    .unwrap_err();
    assert!(matches!(bad_p.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("1.5")));
    let bad_d = DemModel::new(
        vec![Mechanism {
            probability: 0.1,
            detectors: vec![2],
            observables: vec![],
        }],
        2,
        0,
    )
    .unwrap_err();
    assert!(
        matches!(bad_d.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("detector 2"))
    );
    let many: Vec<Mechanism> = (0..=DEM_MAX_MECHANISMS)
        .map(|_| Mechanism {
            probability: 0.1,
            detectors: vec![0],
            observables: vec![],
        })
        .collect();
    assert!(matches!(
        DemModel::new(many, 1, 0).unwrap_err().0,
        QuantumErrorEnum::CalculationError(_)
    ));
    let mut model = two_mechanisms();
    assert_eq!(model.push_phantom().unwrap(), 2);
    assert_eq!(model.mechanisms()[2].probability, 0.0);
    assert!(flips_of(&model.mechanisms()[2]).is_empty());
    assert!(
        (weight(&model, &Query::Io, [0, 0, 0]) - 0.72).abs() < 1e-12,
        "a phantom changes nothing"
    );
    let fired = Query::Fault(Fault::new(None, vec![(2, PauliKind::X)]).unwrap());
    assert!(
        (weight(&model, &fired, [0, 0, 0]) - 0.72).abs() < 1e-12,
        "firing it changes nothing"
    );
    assert_eq!(
        format!("{model}"),
        "detector error model: 3 mechanisms over 2 detectors and 1 observables"
    );
}

#[test]
fn test_a_model_from_a_frozen_graph_reads_the_flips_off_the_edges() {
    use deep_causality::utils_test::test_utils;
    use deep_causality::{BaseCausaloid, CausableGraph, CausaloidGraph};
    // Nodes: 0, 1 mechanisms; 2, 3, 4 detectors; 5 observable.
    let mut g: CausaloidGraph<BaseCausaloid<f64, bool>> = CausaloidGraph::new(0);
    let nodes: Vec<usize> = (0..6)
        .map(|i| {
            g.add_causaloid(test_utils::get_test_causaloid_deterministic(i as u64))
                .unwrap()
        })
        .collect();
    for (from, to) in [(0, 2), (0, 3), (1, 3), (1, 4), (1, 5)] {
        g.add_edge(nodes[from], nodes[to]).unwrap();
    }
    let roles = DemRoles {
        mechanisms: vec![(nodes[0], 0.1), (nodes[1], 0.2)],
        detectors: vec![nodes[2], nodes[3], nodes[4]],
        observables: vec![nodes[5]],
    };
    let unfrozen = DemModel::from_graph(&g, &roles).unwrap_err();
    assert!(
        matches!(unfrozen.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("frozen"))
    );
    g.freeze();
    let model = DemModel::from_graph(&g, &roles).unwrap();
    assert_eq!(model.num_detectors(), 3);
    assert_eq!(model.num_observables(), 1);
    assert_eq!(model.mechanisms().len(), 2);
    assert_eq!(model.mechanisms()[0].detectors, vec![0, 1]);
    assert!(model.mechanisms()[0].observables.is_empty());
    assert_eq!(model.mechanisms()[1].detectors, vec![1, 2]);
    assert_eq!(model.mechanisms()[1].observables, vec![0]);
    let dag = QcModel::<S>::induced_dag(&model);
    assert_eq!(dag.num_edges(), 5);
    assert_eq!(
        dag.parents(3),
        vec![0, 1],
        "detector D1 has both mechanisms"
    );
    let twice = DemRoles {
        mechanisms: vec![(nodes[0], 0.1)],
        detectors: vec![nodes[0]],
        observables: vec![],
    };
    assert!(matches!(
        DemModel::from_graph(&g, &twice).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("two roles")
    ));
    let outside = DemRoles {
        mechanisms: vec![(9, 0.1)],
        detectors: vec![],
        observables: vec![],
    };
    assert!(matches!(
        DemModel::from_graph(&g, &outside).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("node 9")
    ));
}

#[cfg(feature = "dem")]
#[test]
fn test_stim_text_parses_and_refuses_a_repeat_block() {
    let model =
        DemModel::from_stim_text("error(0.01) D0 D1\nerror(0.02) D1 L0\ndetector D0\n").unwrap();
    assert_eq!(model.mechanisms().len(), 2);
    assert_eq!(model.num_detectors(), 2);
    assert_eq!(model.num_observables(), 1);
    assert!((model.mechanisms()[0].probability - 0.01).abs() < 1e-15);
    assert!((model.mechanisms()[1].probability - 0.02).abs() < 1e-15);
    assert_eq!(model.mechanisms()[1].detectors, vec![1]);
    assert_eq!(model.mechanisms()[1].observables, vec![0]);
    let dag = QcModel::<S>::induced_dag(&model);
    assert_eq!(
        dag.parents(3),
        vec![0, 1],
        "D1 has both mechanisms as parents"
    );
    assert_eq!(dag.parents(4), vec![1], "L0 has the second");
    // Comments, blank lines and detector coordinates are accepted.
    let commented =
        DemModel::from_stim_text("# a comment\n\ndetector(1, 2) D3\nerror(0.5) D3 L1 # trailing\n")
            .unwrap();
    assert_eq!(commented.num_detectors(), 4);
    assert_eq!(commented.num_observables(), 2);
    let err =
        DemModel::from_stim_text("error(0.01) D0\nrepeat 3 {\n  error(0.1) D1\n}\n").unwrap_err();
    match err.0 {
        QuantumErrorEnum::CalculationError(m) => {
            assert!(m.contains("line 2") && m.contains("repeat 3 {"), "{m}")
        }
        other => panic!("{other:?}"),
    }
    let bad_target = DemModel::from_stim_text("error(0.01) D0 X1\n").unwrap_err();
    assert!(
        matches!(bad_target.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("line 1"))
    );
    let bad_p = DemModel::from_stim_text("error(abc) D0\n").unwrap_err();
    assert!(
        matches!(bad_p.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("not a number"))
    );
    let no_p = DemModel::from_stim_text("error D0\n").unwrap_err();
    assert!(
        matches!(no_p.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("no probability"))
    );
}

#[test]
fn test_the_memory_experiment_model_lists_its_mechanisms() {
    let incomplete = memory_dem(0.05, 0.02, false);
    let complete = memory_dem(0.05, 0.02, true);
    assert_eq!(incomplete.mechanisms().len(), 3);
    assert_eq!(complete.mechanisms().len(), 4);
    assert_eq!(flips_of(&complete.mechanisms()[3]), "D1 L0");
    // The nominal string carries the empty subset, (1 − p)³ (1 − p_c), and the one subset whose
    // flips cancel, {E0, E1, E01} with weight p² (1 − p) p_c.
    let nominal = 0.95f64.powi(3) * 0.98 + 0.05f64.powi(2) * 0.95 * 0.02;
    assert!((weight(&complete, &Query::Io, [0, 0, 0]) - nominal).abs() < 1e-12);
}

#[test]
fn test_variable_counts_are_capped_at_construction() {
    let overflow = DemModel::new(vec![], usize::MAX, 1).unwrap_err();
    assert!(
        matches!(overflow.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("overflow")),
        "{overflow}"
    );
    let too_many = DemModel::new(vec![], DEM_MAX_VARIABLES, 1).unwrap_err();
    assert!(
        matches!(too_many.0, QuantumErrorEnum::CalculationError(ref m) if m.contains(&DEM_MAX_VARIABLES.to_string())),
        "{too_many}"
    );
    let at_cap = DemModel::new(vec![], DEM_MAX_VARIABLES - 1, 1).unwrap();
    assert_eq!(at_cap.num_variables(), DEM_MAX_VARIABLES);
    // The Stim path counts one past the largest index and goes through the same check.
    let text = format!("error(0.1) D{}\n", DEM_MAX_VARIABLES);
    let err = DemModel::from_stim_text(&text).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(_)),
        "{err}"
    );
}

#[test]
fn test_stim_targets_with_a_multibyte_prefix_or_an_overflowing_index_are_refused_by_line() {
    let err = DemModel::from_stim_text("error(0.1) É0\n").unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("line 1") && m.contains("É0")),
        "{err}"
    );
    let err = DemModel::from_stim_text("error(0.1) D18446744073709551615\n").unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("line 1")),
        "{err}"
    );
    let err = DemModel::from_stim_text("error(0.1) D\n").unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("line 1")),
        "{err}"
    );
}

#[test]
fn test_numeric_query_refuses_before_allocating_above_the_caps() {
    let model = two_mechanisms();
    // Three variables open 2^3 = 8 outcome blocks; a cap of 4 operators refuses them.
    let small_operators = NumericCaps {
        max_entries: 1 << 24,
        max_operators: 4,
    };
    let err = QcModel::<S>::numeric_query(&model, &Query::Io, &small_operators).unwrap_err();
    assert!(
        matches!(
            err.0,
            QuantumErrorEnum::KrausFamilyExceeded {
                operators: 8,
                cap: 4
            }
        ),
        "{err}"
    );
    // Two mechanisms enumerate 2^2 = 4 subsets; a cap of 2 entries refuses them.
    let small_entries = NumericCaps {
        max_entries: 2,
        max_operators: 1 << 12,
    };
    let err = QcModel::<S>::numeric_query(&model, &Query::Io, &small_entries).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("4 subsets") && m.contains("2")),
        "{err}"
    );
    // At the caps the query answers.
    let exact = NumericCaps {
        max_entries: 4,
        max_operators: 8,
    };
    assert!(QcModel::<S>::numeric_query(&model, &Query::Io, &exact).is_ok());
}
