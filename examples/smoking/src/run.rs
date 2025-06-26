/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;

////////////////////////////////////////////
// Classic Smoking - Tar - Cancer example //
////////////////////////////////////////////

pub fn run() {
    // Infer whether there is a causal relationship between smoking / nicotine level and tar
    infer_smoke_tar_causal_relation();

    // Build causaloid to represent the link between nicotine level (smoking) and tar.
    println!("Build Causaloid representing the link between nicotine level (smoking) and tar.");
    let q_smoke_tar = build_smoke_tar_causaloid();

    // Infer whether there is a causal relationship between tar levels and cancer
    infer_tar_cancer_causaloid();

    // Build causaloid representing the link between tar levels and cancer.
    println!("Build Causaloid from known data representing the link between tar and cancer.");
    let q_tar_cancer = build_tar_cancer_causaloid();

    println!("Aggregate all known causes");
    let iter = [q_smoke_tar, q_tar_cancer];
    let all_known_causes: BaseCausaloidVec = Vec::from_iter(iter);
    println!();

    println!("Reason over all aggregated known causes...");
    println!();
    // From *hypothetical* studies, we know the following data:
    // 89% of heavy smokers have high tar levels;
    // 78% of high tar levels have lung cancer;
    // Adoption only requires to plug in either real study results or raw observational data.
    let nic_level = 0.82; // Heavy smoker ( high nicotine level )
    let tar_level = 0.87; // Tar in lung
                          // The order of these numbers must match the order of the causaloids above for correct auto-reasoning.
    let data = [nic_level, tar_level];

    println!(
        "Can we conclude from the data, that smoking, via tar, \
              causes lung cancer in the observed population?"
    );
    // Reason over all known causes using the data from above;
    println!();

    let final_conclusion = all_known_causes.reason_all_causes(&data).unwrap();
    println!("Does Smoking causes cancer: {}", final_conclusion);
    println!();

    println!("Apply Causaloid to a new patient A with new measurements");
    println!();
    println!(
        "Patient A arrives with: \
     \n * nicotine measurement of 77;\
     \n * tar measurement of 81.2;"
    );
    let data = [77.0, 81.2];

    apply_causal_model(&data, &all_known_causes);
    println!();

    println!("Explain your reasoning");
    println!("{}", &all_known_causes.explain());
}

fn apply_causal_model(data: &[NumericalValue], model: &BaseCausaloidVec) {
    let cancer_estimate = model.reason_all_causes(data).unwrap();
    println!("Has the patient a lung cancer risk: {}", cancer_estimate);
}

fn infer_smoke_tar_causal_relation() {
    let all_obs = get_all_observations();

    let target_threshold = 0.55;
    let target_effect = 1.0; //
    let percent_obs = all_obs.percent_observation(target_threshold, target_effect);
    let percent_non_obs = all_obs.percent_non_observation(target_threshold, target_effect);

    let question = "Do smokers have tar in the lung?".to_string() as DescriptionValue;
    let observation = percent_obs;
    let threshold = 0.55; // Threshold above which the observations is considered an inference.
    let effect = 1.0; // observed effect
    let target = 1.0; // expected effect of the inference once the threshold is reached or exceeded
    let inference = Inference::new(0, question, observation, threshold, effect, target);

    println!(
        "Can we infer from the data that smokers have tar in the lung? : {:?}",
        inference.is_inferable()
    );
    println!();

    let inv_question = "Do NON smokers have NO tar in the lung?".to_string() as DescriptionValue;
    let inv_observation = percent_non_obs;
    let inv_effect = 0.0; // observed effect
    let inv_target = 0.0; // expected effect of the inference once the threshold is reached or exceeded
    let inv_inference = Inference::new(
        0,
        inv_question,
        inv_observation,
        threshold,
        inv_effect,
        inv_target,
    );

    println!(
        "Can we infer from the data that NON smokers have NO tar in the lung? : {:?}",
        inv_inference.is_inverse_inferable()
    );
    println!()
}

fn build_smoke_tar_causaloid() -> BaseCausaloid {
    let id = 1;
    let description = "Causal relation between smoking and tar in the lung";

    Causaloid::new(id, causal_fn, description)
}

fn infer_tar_cancer_causaloid() {
    println!();
    let threshold = 0.55; // Threshold above which the observations is considered an inference.
    let question = "Do people with tar in the lung get cancer?".to_string() as DescriptionValue;
    let observation = 0.78;
    let effect = 1.0;
    let target = 1.0; // expected effect of the inference once the threshold is reached or exceeded
    let inference = Inference::new(0, question, observation, threshold, effect, target);
    println!(
        "Can we infer from the data that people with tar get lung cancer? : {:?}",
        inference.is_inferable()
    );
    println!();

    let inv_question =
        "Do people with NO tar in the lung get NO lung cancer?".to_string() as DescriptionValue;
    let inv_observation = 0.12;
    let inv_effect = 0.0;
    let inv_target = 0.0; // expected effect of the inference once the threshold is reached or exceeded
    let inv_inference = Inference::new(
        0,
        inv_question,
        inv_observation,
        threshold,
        inv_effect,
        inv_target,
    );
    println!(
        "Can we infer from the data that NON tar people have NO lung cancer? : {:?}",
        inv_inference.is_inverse_inferable()
    );
    println!();
}

fn build_tar_cancer_causaloid() -> BaseCausaloid {
    let id = 2;
    let description = "Causal relation tar in the lung and lung cancer";

    Causaloid::new(id, causal_fn, description)
}

fn causal_fn(obs: &NumericalValue) -> Result<bool, CausalityError> {
    if obs.is_nan() {
        return Err(CausalityError("Observation is NULL/NAN".into()));
    }

    let threshold: NumericalValue = 0.55;
    if !obs.ge(&threshold) {
        return Ok(false);
    };

    Ok(true)
}

fn get_all_observations() -> Vec<Observation> {
    let o1 = Observation::new(1, 0.89, 1.0);
    let o2 = Observation::new(2, 0.87, 1.0);
    let o3 = Observation::new(3, 0.78, 1.0);
    let o4 = Observation::new(4, 0.65, 1.0);
    let o5 = Observation::new(5, 0.55, 1.0);
    let o6 = Observation::new(6, 0.45, 0.0);
    let o7 = Observation::new(7, 0.45, 0.0);
    let o8 = Observation::new(8, 0.25, 0.0);
    let o9 = Observation::new(9, 0.15, 0.0);

    Vec::from_iter([o1, o2, o3, o4, o5, o6, o7, o8, o9])
}
