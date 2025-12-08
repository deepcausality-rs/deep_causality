/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils::*;
use deep_causality::{
    Assumable, ContextuableGraph, Identifiable, Inferable, MonadicCausable, Observable,
    PropagatingEffect,
};
use deep_causality_haft::LogSize;

#[test]
fn test_context_generators() {
    let ctx1 = get_context();
    assert_eq!(ctx1.id(), 1);
    assert_eq!(ctx1.name(), "base context");

    let ctx2 = get_base_context();
    assert_eq!(ctx2.id(), 1);
    assert_eq!(ctx2.size(), 1); // Has root node
    assert!(ctx2.contains_node(0)); // Root added at index 0

    let ctx3 = get_test_context();
    assert_eq!(ctx3.id(), 1);
    assert_eq!(ctx3.name(), "Test-Context");
    assert_eq!(ctx3.size(), 1);
}

#[test]
fn test_assumptions() {
    // Vector generator
    let vec = get_test_assumption_vec();
    assert_eq!(vec.len(), 3);

    // True assumption
    let a_true = get_test_assumption();
    assert_eq!(a_true.id(), 1);
    let data_present = vec![PropagatingEffect::pure(1.0)];
    assert!(a_true.verify_assumption(&data_present).unwrap());
    let data_empty: Vec<PropagatingEffect<f64>> = vec![];
    assert!(!a_true.verify_assumption(&data_empty).unwrap());

    // False assumption
    let a_false = get_test_assumption_false();
    assert!(!a_false.verify_assumption(&data_present).unwrap());

    // Error assumption
    let a_err = get_test_assumption_error();
    assert!(a_err.verify_assumption(&data_present).is_err());
}

#[test]
fn test_observations_inferences() {
    let obs_vec = get_test_obs_vec();
    assert_eq!(obs_vec.len(), 5);
    // Check percent_observation logic embedded inside get_test_inferable usage of obs vec
    // Just simple check on one
    let o1 = &obs_vec[0];
    assert_eq!(o1.observation(), 10.0);

    let inf_vec = get_test_inf_vec();
    assert_eq!(inf_vec.len(), 2);

    let inf1 = get_test_inferable(10, false);
    assert_eq!(inf1.threshold(), 0.55); // 1.0 >= 0.55

    let inf2 = get_test_inferable(11, true);
    assert_eq!(inf2.threshold(), 0.55); // 0.0 < 0.55 -> false, but verify logic is complex
    // verify_target_threshold checks if observation >= threshold usually?
    // Looking at impl: Inference::new(..., effect, target).
    // if inferable.effect() == inferable.target() return true.
    // In test utils:
    // false case: effect=1.0, target=1.0 -> TRUE
    // true (inverse) case: effect=0.0, target=0.0 -> TRUE
    assert_eq!(inf2.effect(), 0.0);
    assert_eq!(inf2.target(), 0.0);

    let single_obs = get_test_observation();
    assert_eq!(single_obs.observation(), 14.0);
}

#[test]
fn test_data_generators() {
    let eff = get_test_single_data(42.0);
    assert_eq!(eff.value.into_value().unwrap(), 42.0);

    let arr = get_test_num_array();
    assert_eq!(arr.len(), 10);
    assert_eq!(arr[0], 8.4);

    let sample: [f64; 5] = generate_sample_data();
    assert_eq!(sample.len(), 5);
    assert_eq!(sample[0], 0.99);
}

#[test]
fn test_causaloid_vectors() {
    let v_det = get_deterministic_test_causality_vec();
    assert_eq!(v_det.len(), 3);

    let v_prob = get_probabilistic_test_causality_vec();
    assert_eq!(v_prob.len(), 3);

    let v_ub = get_uncertain_bool_test_causality_vec();
    assert_eq!(v_ub.len(), 3);

    let v_uf = get_uncertain_float_test_causality_vec();
    assert_eq!(v_uf.len(), 3);
}

#[test]
fn test_deterministic_booleans() {
    let c_true = get_test_causaloid_deterministic_true();
    let res = c_true.evaluate(&PropagatingEffect::pure(false)); // input ignored
    assert!(res.value.into_value().unwrap());
    assert!(!res.logs.is_empty());

    let c_false = get_test_causaloid_deterministic_false();
    let res = c_false.evaluate(&PropagatingEffect::pure(true));
    assert!(!res.value.into_value().unwrap());

    let c_inv = get_test_causaloid_deterministic_input_output();
    let res = c_inv.evaluate(&PropagatingEffect::pure(true));
    assert!(!res.value.into_value().unwrap()); // !true = false
}

#[test]
fn test_probabilistic_causaloids() {
    let c_prob = get_test_causaloid_probabilistic();
    // Threshold 0.55
    let res = c_prob.evaluate(&PropagatingEffect::pure(0.6));
    assert_eq!(res.value.into_value().unwrap(), 1.0);

    let res = c_prob.evaluate(&PropagatingEffect::pure(0.5));
    assert_eq!(res.value.into_value().unwrap(), 0.0);

    let c_prob_bool = get_test_causaloid_probabilistic_bool_output();
    // Same logic 0.55
    assert_eq!(
        c_prob_bool
            .evaluate(&PropagatingEffect::pure(0.6))
            .value
            .into_value()
            .unwrap(),
        1.0
    );
}

#[test]
fn test_uncertain_causaloids() {
    let c_ub = get_test_causaloid_uncertain_bool();
    // > 0.55 -> true
    let res = c_ub.evaluate(&PropagatingEffect::pure(0.6));
    let ub = res.value.into_value().unwrap();
    // Point uncertain bool has prob 1.0 if true
    assert!(ub.to_bool(0.5, 0.95, 0.05, 100).unwrap());

    let res = c_ub.evaluate(&PropagatingEffect::pure(0.5));
    let ub = res.value.into_value().unwrap();
    assert!(!ub.to_bool(0.5, 0.95, 0.05, 100).unwrap());

    let c_uf = get_test_causaloid_uncertain_float();
    let res = c_uf.evaluate(&PropagatingEffect::pure(0.6));
    let uf = res.value.into_value().unwrap();
    // Point(1.0)
    assert!((uf.value() - 1.0).abs() < 1e-6);
}

#[test]
fn test_deterministic_comparisons() {
    let c = get_test_causaloid_deterministic(10);
    // > 0.55 -> true
    assert!(
        c.evaluate(&PropagatingEffect::pure(0.6))
            .value
            .into_value()
            .unwrap()
    );
    assert!(
        !c.evaluate(&PropagatingEffect::pure(0.5))
            .value
            .into_value()
            .unwrap()
    );
}

#[test]
fn test_context_causaloid() {
    let ctx = get_context(); // ID 1
    // Function logic: if context.id == 1 { return input } else { return !input }

    let c = get_test_causaloid_deterministic_with_context(ctx);
    // Should return input (id 1 matches)
    let res = c.evaluate(&PropagatingEffect::pure(true));
    assert!(res.value.into_value().unwrap());

    // Test with missing context (difficult to trigger via public evaluate if context is baked in,
    // but the closure handles it. The Causaloid created has context wrapped).

    // Let's create one with modified context ID manually to test else branch?
    // Helper function consumes context by value so we can pass a different one.
    let _ctx2 = get_context();
    // Hack context id if possible or create new with different ID? context ID is immutable or private?
    // Context::with_capacity(id, ...)
    let ctx2 = deep_causality::Context::with_capacity(2, "other", 10);
    let c2 = get_test_causaloid_deterministic_with_context(ctx2);

    // ID 2 -> should invert
    let res2 = c2.evaluate(&PropagatingEffect::pure(true));
    assert!(!res2.value.into_value().unwrap());
}

#[test]
fn test_error_and_logging_causaloids() {
    // Error Causaloid
    let c_err = get_test_error_causaloid();
    let res = c_err.evaluate(&PropagatingEffect::pure(true));
    assert!(res.is_err());
    assert!(res.error.unwrap().to_string().contains("Test error"));

    // Logging/Logic Causaloid (bool output)
    let c_log = get_test_causaloid(5);
    // Negative -> Error
    let res_neg = c_log.evaluate(&PropagatingEffect::pure(-1.0));
    assert!(res_neg.is_err());
    assert!(
        res_neg
            .error
            .unwrap()
            .to_string()
            .contains("Observation is negative")
    );
    assert!(!res_neg.logs.is_empty());

    // Positive > 0.55
    let res_pos = c_log.evaluate(&PropagatingEffect::pure(0.6));
    assert!(res_pos.value.into_value().unwrap());
    assert!(!res_pos.logs.is_empty());

    // Logging/Logic Causaloid (f64 output)
    let c_log_num = get_test_causaloid_num_input_output(6);
    // Negative
    let res_neg = c_log_num.evaluate(&PropagatingEffect::pure(-1.0));
    assert!(res_neg.is_err());
    // Positive
    let res_pos = c_log_num.evaluate(&PropagatingEffect::pure(0.6));
    assert_eq!(res_pos.value.into_value().unwrap(), 1.0);
}
