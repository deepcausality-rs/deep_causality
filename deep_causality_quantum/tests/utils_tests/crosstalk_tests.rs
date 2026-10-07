/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qcm")]

use deep_causality_quantum::utils_tests::{
    CROSSTALK_BATH, CROSSTALK_Q1, CROSSTALK_Q2, CrosstalkModel, CrosstalkSetting,
    crosstalk_candidates,
};
use deep_causality_quantum::{
    Channel, Hypothesis, ObservedContext, QubitOperator, Response, ResponseModel,
};

#[test]
fn test_the_candidates_span_two_or_three_legs_and_are_normalised_processes() {
    let [h1, h2, h3] = crosstalk_candidates().unwrap();
    assert_eq!(h1.legs().unwrap().len(), 2);
    assert_eq!(h2.legs().unwrap().len(), 2);
    assert_eq!(h3.legs().unwrap().len(), 3);
    assert_eq!(
        (h1.name(), h2.name(), h3.name()),
        ("H1 Q1->Q2", "H2 Q2->Q1", "H3 Q1<-B->Q2")
    );
    for h in [&h1, &h2, &h3] {
        assert!(h.check_normalization().unwrap().accepted(), "{}", h.name());
    }
}

fn intervention(h: &Hypothesis<f64>, setting: CrosstalkSetting) -> (Vec<usize>, Vec<usize>) {
    let Response::Intervention {
        factors,
        instrument,
    } = CrosstalkModel.respond(h, &setting).unwrap()
    else {
        panic!("an intervention");
    };
    (
        factors.iter().map(|(node, _)| *node).collect(),
        instrument.shape().to_vec(),
    )
}

#[test]
fn test_each_setting_intervenes_where_it_should() {
    let [h1, h2, h3] = crosstalk_candidates().unwrap();
    let hold = CrosstalkSetting::Hold {
        node: CROSSTALK_Q1,
        read: CROSSTALK_Q2,
    };
    // Holding replaces the held qubit's factor; the instrument spans the candidate's legs.
    assert_eq!(intervention(&h2, hold), (vec![CROSSTALK_Q1], vec![4, 4]));
    assert_eq!(intervention(&h3, hold), (vec![CROSSTALK_Q1], vec![8, 8]));
    // Passive intervenes nowhere.
    assert_eq!(
        intervention(&h1, CrosstalkSetting::Passive),
        (vec![], vec![4, 4])
    );
    // The echo decouples the driven qubit of a direct coupling and leaves the bath alone.
    assert_eq!(
        intervention(&h1, CrosstalkSetting::Echo).0,
        vec![CROSSTALK_Q2]
    );
    assert_eq!(
        intervention(&h2, CrosstalkSetting::Echo).0,
        vec![CROSSTALK_Q1]
    );
    assert_eq!(
        intervention(&h3, CrosstalkSetting::Echo).0,
        Vec::<usize>::new()
    );
}

#[test]
fn test_the_model_refuses_a_mechanism_and_a_node_the_candidate_lacks() {
    let flip = Hypothesis::mechanism("flip", Channel::unitary(&QubitOperator::pauli_x()).unwrap());
    assert!(
        CrosstalkModel
            .respond(&flip, &CrosstalkSetting::Passive)
            .is_err()
    );
    let [h1, ..] = crosstalk_candidates().unwrap();
    let bath = CrosstalkSetting::Hold {
        node: CROSSTALK_BATH,
        read: CROSSTALK_Q1,
    };
    assert!(CrosstalkModel.respond(&h1, &bath).is_err());
}

#[test]
fn test_a_setting_records_no_context() {
    assert_eq!(CrosstalkSetting::Echo.context_snapshot().unwrap(), None);
}
