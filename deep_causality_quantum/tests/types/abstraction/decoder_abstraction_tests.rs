/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The decoder as an abstraction from the memory experiment to its detector error model.
//!
//! The circuit's record is a classical stochastic process (only `X` noise and `Z` readout), so
//! with the complete model every square commutes exactly. With the correlated `X ⊗ X` mechanism
//! omitted, every square carries the `p_c` mixing of the nominal record, since a fault shifts both
//! sides alike, and the square at the injected location fails by far the most: the phantom
//! mechanism predicts no change where the circuit shifts its record by `D1 L0`, a residual near
//! `√2`. Attribution ranks the injected fault first because the model already
//! assigns first-order probability `p` to each single-qubit pattern and only second-order weight
//! `p²` to the `D1 L0` pattern, so with `p_c < 2p` the correlated fault is the largest surprise.

use deep_causality_quantum::utils_tests::{
    memory_dem, memory_experiment, memory_experiment_caps, memory_locations, memory_tau,
};
use deep_causality_quantum::{
    CheckVerdict, DecoderAbstraction, Fault, FaultSet, PauliKind, QuantumErrorEnum, Query,
    stochastic_morphism,
};

const P: f64 = 0.05;
const P_C: f64 = 0.02;

fn locations(complete: bool) -> Vec<(Fault, Option<usize>)> {
    let faults = memory_locations();
    vec![
        (faults[0].clone(), Some(0)),
        (faults[1].clone(), Some(1)),
        (faults[2].clone(), Some(2)),
        (faults[3].clone(), if complete { Some(3) } else { None }),
    ]
}

#[test]
fn test_a_model_that_represents_every_mechanism_passes() {
    let caps = memory_experiment_caps();
    let decoder = DecoderAbstraction::new(
        memory_experiment::<f64>(P, P_C),
        memory_dem(P, P_C, true),
        &memory_tau(),
        locations(true),
    )
    .unwrap();
    let report = decoder.check(&caps).unwrap();
    assert!(report.holds(), "{report}");
    assert_eq!(
        report.naturality.report.examined(),
        5,
        "Io and four locations"
    );
    assert_eq!(report.naturality.report.verdict(), CheckVerdict::Accepted);
    assert!(report.naturality.worst_residual() < 1e-12);
    assert!(format!("{report}").contains("every square commutes"));
    assert_eq!(decoder.locations().len(), 4);
    assert!(decoder.locations().iter().all(|(_, modelled)| *modelled));
    assert!(decoder.abstraction().signature().len() == 5);
}

#[test]
fn test_an_omitted_correlated_mechanism_is_exposed_at_its_location() {
    let caps = memory_experiment_caps();
    let decoder = DecoderAbstraction::new(
        memory_experiment::<f64>(P, P_C),
        memory_dem(P, P_C, false),
        &memory_tau(),
        locations(false),
    )
    .unwrap();
    let report = decoder.check(&caps).unwrap();
    assert!(!report.holds());
    // Every square carries the p_c mixing of the nominal record, the same residual to 1e-12
    // because a fault shifts both sides alike; the injected location fails against a phantom by
    // an order of magnitude more, and the report names it as the worst.
    let injected = memory_locations()[3].clone();
    assert_eq!(report.failures.len(), 5);
    let worst = report.worst().unwrap();
    assert_eq!(worst.location.as_ref(), Some(&injected));
    assert!(!worst.modelled);
    assert!(
        worst.residual > 1.0,
        "a shifted record against an unshifted one: {}",
        worst.residual
    );
    let nominal = report
        .failures
        .iter()
        .find(|f| f.location.is_none())
        .unwrap();
    assert!(
        nominal.residual > 0.0 && nominal.residual < 0.1,
        "the p_c mixing: {}",
        nominal.residual
    );
    for f in report
        .failures
        .iter()
        .filter(|f| f.modelled && f.location.is_some())
    {
        assert!((f.residual - nominal.residual).abs() < 1e-12, "{report}");
    }
    assert!(worst.residual > 10.0 * nominal.residual);
    let shown = format!("{report}");
    assert!(
        shown.contains("worst at X0 X1 after node 3") && shown.contains("not in the model"),
        "{shown}"
    );
}

#[test]
fn test_attribution_ranks_the_injected_location_first() {
    let caps = memory_experiment_caps();
    let decoder = DecoderAbstraction::new(
        memory_experiment::<f64>(P, P_C),
        memory_dem(P, P_C, false),
        &memory_tau(),
        locations(false),
    )
    .unwrap();
    let mut faults = FaultSet::pauli_weight(&[0, 1, 2], Some(3), 1, 1 << 10)
        .unwrap()
        .faults()
        .to_vec();
    faults.push(memory_locations()[3].clone());
    let set = FaultSet::declared(&faults);
    assert_eq!(set.len(), 10);
    let attribution = decoder.attribute(&set, &caps).unwrap();
    assert_eq!(attribution.ranked.len(), 10);
    let (first, top) = attribution.first().unwrap();
    assert_eq!(first, &memory_locations()[3], "{attribution}");
    assert!(
        attribution.ranked[1..].iter().all(|(_, r)| *r < *top),
        "{attribution}"
    );
    // Z faults leave a Z-basis record unchanged: their residual is the nominal model mismatch.
    let z_residuals: Vec<f64> = attribution
        .ranked
        .iter()
        .filter(|(f, _)| f.errors()[0].1 == PauliKind::Z)
        .map(|(_, r)| *r)
        .collect();
    assert_eq!(z_residuals.len(), 3);
    assert!(z_residuals.iter().all(|r| *r < 0.1));
    // Descending order throughout.
    assert!(attribution.ranked.windows(2).all(|w| w[0].1 >= w[1].1));
}

#[test]
fn test_the_decoder_enters_as_a_stochastic_matrix_and_nothing_else() {
    let tau = memory_tau();
    let m = stochastic_morphism::<f64>(&tau, &[2, 2, 2, 2, 2], &[2, 2, 2]).unwrap();
    assert_eq!(m.operator_count(), 32, "one scalar block per low string");
    assert_eq!(m.classical_in(), &[2, 2, 2, 2, 2]);
    assert_eq!(m.classical_out(), &[2, 2, 2]);
    assert_eq!((m.d_in(), m.d_out()), (1, 1));
    // The low string (a0, a1, m0, m1, m2) = (1, 0, 1, 1, 0) reads as (D0, D1, L0) = (1, 0, 1).
    assert!(
        m.blocks()
            .contains_key(&(vec![1, 0, 1, 1, 0], vec![1, 0, 1]))
    );
    assert!(
        !m.blocks()
            .contains_key(&(vec![1, 0, 1, 1, 0], vec![0, 0, 0]))
    );
    // A soft decoder is allowed: a row split 0.7 / 0.3.
    let mut soft = tau.clone();
    soft[0] = vec![0.7, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let s = stochastic_morphism::<f64>(&soft, &[2, 2, 2, 2, 2], &[2, 2, 2]).unwrap();
    assert_eq!(s.operator_count(), 33);
    let mut short = tau.clone();
    short[3][0] = 0.5;
    let err = stochastic_morphism::<f64>(&short, &[2, 2, 2, 2, 2], &[2, 2, 2]).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::NonCptpChannel(ref m) if m.contains("row 3")));
    let wrong_shape = stochastic_morphism::<f64>(&tau, &[2, 2, 2, 2], &[2, 2, 2]).unwrap_err();
    assert!(matches!(
        wrong_shape.0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    let bad_mechanism = DecoderAbstraction::new(
        memory_experiment::<f64>(P, P_C),
        memory_dem(P, P_C, false),
        &memory_tau(),
        vec![(memory_locations()[0].clone(), Some(7))],
    )
    .unwrap_err();
    assert!(
        matches!(bad_mechanism.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("mechanism 7"))
    );
    let io_only = DecoderAbstraction::new(
        memory_experiment::<f64>(P, P_C),
        memory_dem(P, P_C, true),
        &memory_tau(),
        vec![],
    )
    .unwrap();
    assert_eq!(io_only.abstraction().signature().queries(), &[Query::Io]);
    assert!(io_only.check(&memory_experiment_caps()).unwrap().holds());
}

#[test]
fn test_outcome_counts_whose_product_overflows_are_refused() {
    let err = stochastic_morphism::<f64>(&[], &[usize::MAX, 2], &[2]).expect_err("refused");
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("overflow")),
        "{err}"
    );
    let err = stochastic_morphism::<f64>(&[], &[2], &[usize::MAX, 2]).expect_err("refused");
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("overflow")),
        "{err}"
    );
}
