// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{Identifiable, Inference};
use deep_causality::protocols::inferable::Inferable;
use deep_causality::protocols::observable::ObservableReasoning;
use deep_causality::types::alias_types::DescriptionValue;

use crate::utils::test_utils::*;

#[test]
fn test_id() {
    let id = 0;
    let question = "Test inference ".to_string() as DescriptionValue;
    let all_obs = get_test_obs_vec();
    let target_effect = 1.0;
    let target_threshold = 10.0;
    let observation = all_obs.percent_observation(target_threshold, target_effect);

    let threshold = 0.55; // Threshold above which the observations is considered an inference.
    let effect = 1.0; // expected effect of the inference once the threshold is reached or exceeded
    let infer = Inference::new(
        id,
        question.clone(),
        observation,
        threshold,
        effect,
        target_effect,
    );
    assert_eq!(infer.id(), id);
}

#[test]
fn test_question() {
    let id = 0;
    let question = "Test inference ".to_string() as DescriptionValue;
    let all_obs = get_test_obs_vec();
    let target_effect = 1.0;
    let target_threshold = 10.0;
    let observation = all_obs.percent_observation(target_threshold, target_effect);

    let threshold = 0.55; // Threshold above which the observations is considered an inference.
    let effect = 1.0; // expected effect of the inference once the threshold is reached or exceeded
    let infer = Inference::new(
        id,
        question.clone(),
        observation,
        threshold,
        effect,
        target_effect,
    );
    assert_eq!(infer.id(), id);
    assert_eq!(infer.question(), question.clone());
}

#[test]
fn test_observation() {
    let id = 0;
    let question = "Test inference ".to_string() as DescriptionValue;
    let all_obs = get_test_obs_vec();
    let target_effect = 1.0;
    let target_threshold = 10.0;
    let observation = all_obs.percent_observation(target_threshold, target_effect);

    let threshold = 0.55; // Threshold above which the observations is considered an inference.
    let effect = 1.0; // expected effect of the inference once the threshold is reached or exceeded
    let infer = Inference::new(
        id,
        question.clone(),
        observation,
        threshold,
        effect,
        target_effect,
    );
    assert_eq!(infer.id(), id);
    assert_eq!(infer.question(), question.clone());
    assert_eq!(infer.observation(), observation);
}

#[test]
fn test_threshold() {
    let id = 0;
    let question = "Test inference ".to_string() as DescriptionValue;
    let all_obs = get_test_obs_vec();
    let target_effect = 1.0;
    let target_threshold = 10.0;
    let observation = all_obs.percent_observation(target_threshold, target_effect);

    let threshold = 0.55; // Threshold above which the observations is considered an inference.
    let effect = 1.0; // expected effect of the inference once the threshold is reached or exceeded
    let infer = Inference::new(
        id,
        question.clone(),
        observation,
        threshold,
        effect,
        target_effect,
    );
    assert_eq!(infer.id(), id);
    assert_eq!(infer.question(), question.clone());
    assert_eq!(infer.observation(), observation);
    assert_eq!(infer.threshold(), threshold);
}

#[test]
fn test_effect() {
    let id = 0;
    let question = "Test inference ".to_string() as DescriptionValue;
    let all_obs = get_test_obs_vec();
    let target_effect = 1.0;
    let target_threshold = 10.0;
    let observation = all_obs.percent_observation(target_threshold, target_effect);

    let threshold = 0.55; // Threshold above which the observations is considered an inference.
    let effect = 1.0; // expected effect of the inference once the threshold is reached or exceeded
    let infer = Inference::new(
        id,
        question.clone(),
        observation,
        threshold,
        effect,
        target_effect,
    );
    assert_eq!(infer.id(), id);
    assert_eq!(infer.question(), question.clone());
    assert_eq!(infer.observation(), observation);
    assert_eq!(infer.threshold(), threshold);
    assert_eq!(infer.effect(), effect);
}

#[test]
fn test_target() {
    let id = 0;
    let question = "Test inference ".to_string() as DescriptionValue;
    let all_obs = get_test_obs_vec();
    let target_effect = 1.0;
    let target_threshold = 10.0;
    let observation = all_obs.percent_observation(target_threshold, target_effect);

    let threshold = 0.55; // Threshold above which the observations is considered an inference.
    let effect = 1.0; // expected effect of the inference once the threshold is reached or exceeded
    let infer = Inference::new(
        id,
        question.clone(),
        observation,
        threshold,
        effect,
        target_effect,
    );
    assert_eq!(infer.id(), id);
    assert_eq!(infer.question(), question.clone());
    assert_eq!(infer.observation(), observation);
    assert_eq!(infer.threshold(), threshold);
    assert_eq!(infer.effect(), effect);
    assert_eq!(infer.target(), target_effect);
}

#[test]
fn test_inferable() {
    let question = "Test inference ".to_string() as DescriptionValue;
    let all_obs = get_test_obs_vec();
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
    let all_obs = get_test_obs_vec();
    let observation = all_obs.percent_observation(target_threshold, target_effect);

    // inversion means, no observation means no effect hence inverted inference.
    let effect = 0.0; // no effect expected
    let threshold = 0.55; // threshold remains the same
    let infer = Inference::new(0, question, observation, threshold, effect, target_effect);

    assert!(infer.is_inverse_inferable())
}

#[test]
fn test_to_string() {
    let id = 0;
    let question = "Test inference ".to_string() as DescriptionValue;
    let all_obs = get_test_obs_vec();
    let target_effect = 1.0;
    let target_threshold = 10.0;
    let observation = all_obs.percent_observation(target_threshold, target_effect);

    let threshold = 0.55; // Threshold above which the observations is considered an inference.
    let effect = 1.0; // expected effect of the inference once the threshold is reached or exceeded
    let infer = Inference::new(id, question, observation, threshold, effect, target_effect);

    let expected_string =
        "Inference: id: 0, question: Test inference , observation: 0.6, threshold: 0.55, effect: 1"
            .to_string();
    let actual_string = infer.to_string();
    assert_eq!(expected_string, actual_string);
}
