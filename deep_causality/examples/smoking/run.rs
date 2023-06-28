// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::*;
use deep_causality::types::reasoning_types;

type AllCauses = Vec<Causaloid>; // type alias for brevity

pub fn run()
{
    // Build causaloid from raw observations
    println!("Build Causaloid representing the link between nicotine level (smoking) and tar.");
    let q_smoke_tar = build_smoke_tar_causaloid();

    // Build causaloid from existing results i.e.from studies or meta-studies
    println!("Build Causaloid from known data representing the link between tar and cancer.");
    let q_tar_cancer = build_tar_cancer_causaloid();

    println!("Aggregate all known causes");
    let iter = [q_smoke_tar, q_tar_cancer];
    let all_known_causes: AllCauses = Vec::from_iter(iter);
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

fn apply_causal_model(
    data: &[NumericalValue],
    model: &AllCauses)
{
    let cancer_estimate = model.reason_all_causes(data).unwrap();
    println!("Has the patient a lung cancer risk: {}", cancer_estimate);
}

fn build_smoke_tar_causaloid() -> Causaloid
{
    println!();
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
    let inv_inference = Inference::new(0, inv_question, inv_observation, threshold, inv_effect, inv_target);

    println!(
        "Can we infer from the data that NON smokers have NO tar in the lung? : {:?}",
        inv_inference.is_inverse_inferable()
    );
    println!();

    // 1) Build inference collection
    let inferable_coll: Vec<Inference> = Vec::from_iter([inference]);
    let inverse_inferable_coll: Vec<Inference> = Vec::from_iter([inv_inference]);
    // 2) Describe the causal relation
    let id = 01;
    let description = "Causal relation between smoking and tar in the lung".to_string();
    let data_set_id = "Data set abc from study Homer et al. DOI: 122345 ".to_string();

    //3) Build the causaloid
    build_causaloid(
        id,
        causal_fn,
        description,
        data_set_id,
        &inferable_coll,
        &inverse_inferable_coll,
    ).unwrap()
}

fn build_tar_cancer_causaloid() -> Causaloid
{
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
    let inv_inference = Inference::new(0, inv_question, inv_observation, threshold, inv_effect, inv_target);
    println!(
        "Can we infer from the data that NON tar people have NO lung cancer? : {:?}",
        inv_inference.is_inverse_inferable()
    );
    println!();

    // 1) Build inference collection
    let inferable_coll: Vec<Inference> = Vec::from_iter([inference]);
    let inverse_inferable_coll: Vec<Inference> = Vec::from_iter([inv_inference]);

    // 2) Describe the causal relation
    let id = 02;
    let description = "Causal relation tar in the lung and lung cancer".to_string();
    let data_set_id = "Aggregated data set from meta study Parcel et al. DOI: 122345 ".to_string();

    //3) Build the causaloid
    reasoning_types::causable::build_causaloid(
        id,
        causal_fn,
        description,
        data_set_id,
        &inferable_coll,
        &inverse_inferable_coll,
    ).unwrap()
}

fn _build_cancer_death_causaloid() -> Causaloid {
    // regular inference
    let threshold = 0.55; // Threshold above which the observations is considered an inference.
    let question = "Do people with lung cancer die earlier compared to those without?".to_string();
    let observation = 0.89;
    let effect = 1.0;
    let target = 1.0; // expected effect of the inference once the threshold is reached or exceeded
    let inference = Inference::new(0, question, observation, threshold, effect, target);
    let inferable_coll: Vec<Inference> = Vec::from_iter([inference]);

    // inverse inference
    let inv_question = "Do people with NO lung cancer do NOT die earlier?".to_string();
    let inv_observation = 0.15;
    let inv_effect = 0.0;
    let inv_target = 0.0; // expected effect of the inference once the threshold is reached or exceeded
    let inv_inference = Inference::new(0, inv_question, inv_observation, threshold, inv_effect, inv_target);

    // Causaloid
    let id = 03;
    let description = "Causal relation lung cancer and early death".to_string();
    let data_set_id = "Aggregated data set from meta study Morbid et al. DOI: 122345 ".to_string();
    let inverse_inferable_coll: Vec<Inference> = Vec::from_iter([inv_inference]);

    reasoning_types::causable::build_causaloid(
        id,
        causal_fn,
        description,
        data_set_id,
        &inferable_coll,
        &inverse_inferable_coll,
    ).unwrap()
}

fn get_all_observations() -> Vec<Observation>
{
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


fn causal_fn(
    obs: NumericalValue
)
    -> Result<bool, CausalityError>
{
    if obs.is_nan() {
        return Err(CausalityError("Observation is NULL/NAN".into()));
    }

    let threshold: NumericalValue = 0.55;
    if !obs.ge(&threshold) {
        return Ok(false);
    };

    Ok(true)
}
