// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.



use deep_causality::prelude::Inference;
use deep_causality::protocols::inferable::Inferable;
use deep_causality::protocols::observable::ObservableReasoning;
use deep_causality::types::alias_types::DescriptionValue;
use deep_causality::utils::test_utils::get_test_obs_coll;

#[test]
fn test_inferable() {
    let question = "Test inference ".to_string() as DescriptionValue;
    let all_obs = get_test_obs_coll();
    let target_effect = 1.0;
    let target_threshold = 10.0;
    let observation = all_obs.percent_observation(target_threshold, target_effect);

    let threshold = 0.55; // Threshold above which the observations is considered an inference.
    let effect = 1.0; // expected effect of the inference once the threshold is reached or exceeded
    let infer = Inference::new(0, question, observation, threshold, effect, target_effect);
    assert!(infer.is_inferable())
}

#[test]
fn test_inverse_inferable() {
    let question = "Test inference  ".to_string() as DescriptionValue;
    let target_effect = 0.0;
    let target_threshold = 10.0;
    let all_obs = get_test_obs_coll();
    let observation = all_obs.percent_observation(target_threshold, target_effect);

    // inversion means, no observation means no effect hence inverted inference.
    let effect = 0.0; // no effect expected
    let threshold = 0.55; // threshold remains the same
    let infer = Inference::new(0, question, observation, threshold, effect, target_effect);

    assert!(infer.is_inverse_inferable())
}
