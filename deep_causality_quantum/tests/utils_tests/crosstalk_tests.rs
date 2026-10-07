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
fn test_the_candidates_span_two_or_three_legs() {
    let [h1, h2, h3] = crosstalk_candidates().unwrap();
    assert_eq!(h1.legs().unwrap().len(), 2);
    assert_eq!(h2.legs().unwrap().len(), 2);
    assert_eq!(h3.legs().unwrap().len(), 3);
    assert_eq!(
        (h1.name(), h2.name(), h3.name()),
        ("H1 Q1->Q2", "H2 Q2->Q1", "H3 Q1<-B->Q2")
    );
}

#[test]
fn test_holding_a_qubit_replaces_its_factor_on_its_support() {
    let [_, h2, h3] = crosstalk_candidates().unwrap();
    let hold = CrosstalkSetting::Hold {
        node: CROSSTALK_Q1,
        read: CROSSTALK_Q2,
    };
    for (h, joint) in [(&h2, 4), (&h3, 8)] {
        let Response::Intervention {
            factors,
            instrument,
        } = CrosstalkModel.respond(h, &hold).unwrap()
        else {
            panic!("an intervention");
        };
        // Q1 has one parent in both, so its factor is on two legs; the instrument is on all.
        assert_eq!(factors.len(), 1);
        assert_eq!(factors[0].0, CROSSTALK_Q1);
        assert_eq!(factors[0].1.shape(), &[4, 4]);
        assert_eq!(instrument.shape(), &[joint, joint]);
    }
    let passive = CrosstalkSetting::Passive { read: CROSSTALK_Q1 };
    let Response::Intervention { factors, .. } = CrosstalkModel.respond(&h2, &passive).unwrap()
    else {
        panic!("an intervention");
    };
    assert!(factors.is_empty());
}

#[test]
fn test_the_model_refuses_a_mechanism_and_a_node_the_candidate_lacks() {
    let flip = Hypothesis::mechanism("flip", Channel::unitary(&QubitOperator::pauli_x()).unwrap());
    let passive = CrosstalkSetting::Passive { read: CROSSTALK_Q1 };
    assert!(CrosstalkModel.respond(&flip, &passive).is_err());
    let [h1, ..] = crosstalk_candidates().unwrap();
    let bath = CrosstalkSetting::Hold {
        node: CROSSTALK_BATH,
        read: CROSSTALK_Q1,
    };
    assert!(CrosstalkModel.respond(&h1, &bath).is_err());
}

#[test]
fn test_a_setting_records_no_context() {
    assert_eq!(
        CrosstalkSetting::Passive { read: CROSSTALK_Q2 }
            .context_snapshot()
            .unwrap(),
        None
    );
}
