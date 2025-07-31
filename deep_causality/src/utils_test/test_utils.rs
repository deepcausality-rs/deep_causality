/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::*;

pub fn get_context() -> BaseContext {
    let id = 1;
    let name = "base context";
    let capacity = 10; // adjust as needed
    Context::with_capacity(id, name, capacity)
}

pub fn get_test_assumption_vec() -> Vec<Assumption> {
    let a1 = get_test_assumption();
    let a2 = get_test_assumption();
    let a3 = get_test_assumption();
    Vec::from_iter([a1, a2, a3])
}

pub fn get_test_obs_vec() -> Vec<Observation> {
    let o1 = Observation::new(0, 10.0, 1.0);
    let o2 = Observation::new(1, 10.0, 1.0);
    let o3 = Observation::new(2, 10.0, 1.0);
    let o4 = Observation::new(3, 12.0, 0.0);
    let o5 = Observation::new(4, 14.0, 0.0);
    Vec::from_iter([o1, o2, o3, o4, o5])
}

pub fn get_test_inf_vec() -> Vec<Inference> {
    let i1 = get_test_inferable(0, true);
    let i2 = get_test_inferable(1, false);
    Vec::from_iter([i1, i2])
}

pub fn get_test_causality_vec() -> BaseCausaloidVec {
    let q1 = get_test_causaloid();
    let q2 = get_test_causaloid();
    let q3 = get_test_causaloid();
    Vec::from_iter([q1, q2, q3])
}

pub fn get_test_single_data(val: NumericalValue) -> PropagatingEffect {
    PropagatingEffect::Numerical(val)
}

pub fn get_test_causaloid_deterministic_true() -> BaseCausaloid {
    let description = "tests nothing; always returns true";

    fn causal_fn(_effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        Ok(PropagatingEffect::Deterministic(true))
    }

    Causaloid::new(3, causal_fn, description)
}

pub fn get_test_causaloid_deterministic_false() -> BaseCausaloid {
    let description = "tests nothing; always returns true";

    fn causal_fn(_effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        Ok(PropagatingEffect::Deterministic(false))
    }

    Causaloid::new(3, causal_fn, description)
}

pub fn get_test_causaloid_probabilistic() -> BaseCausaloid {
    let description = "tests nothing; always returns 0.0";

    fn causal_fn(_effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        Ok(PropagatingEffect::Probabilistic(0.0))
    }

    Causaloid::new(5, causal_fn, description)
}

pub fn get_test_causaloid_halting() -> BaseCausaloid {
    let description = "tests nothing; always returns Halting";

    fn causal_fn(_effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        Ok(PropagatingEffect::Halting)
    }

    Causaloid::new(7, causal_fn, description)
}

pub fn get_test_causaloid_contextual_link() -> BaseCausaloid {
    let description = "tests nothing; always returns a contetual link";

    fn causal_fn(_effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        Ok(PropagatingEffect::ContextualLink(0, 1))
    }

    Causaloid::new(9, causal_fn, description)
}

pub fn get_test_causaloid() -> BaseCausaloid {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";

    fn causal_fn(effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        let obs =
            match effect {
                // If it's the Numerical variant, extract the inner value.
                PropagatingEffect::Numerical(val) => *val,
                // For any other type of effect, this function cannot proceed, so return an error.
                _ => return Err(CausalityError(
                    "Causal function expected Numerical effect but received a different variant."
                        .into(),
                )),
            };

        let threshold: NumericalValue = 0.55;
        if !obs.ge(&threshold) {
            Ok(PropagatingEffect::Deterministic(false))
        } else {
            Ok(PropagatingEffect::Deterministic(true))
        }
    }

    Causaloid::new(id, causal_fn, description)
}

// BaseContext is a type alias for a basic context that can be used for testing
// It matches the type signature of the base causaloid also uses in these tests.
// See src/types/alias_types/csm_types for definition.
pub fn get_base_context() -> BaseContext {
    let id = 1;
    let name = "base context";
    let mut context = Context::with_capacity(id, name, 10);
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid = Contextoid::new(id, ContextoidType::Root(root));
    let idx = context.add_node(contextoid).expect("Failed to add node");
    assert_eq!(idx, 0);
    assert_eq!(context.size(), 1);

    context
}

pub fn get_test_error_causaloid() -> BaseCausaloid {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";

    fn causal_fn(_obs: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        Err(CausalityError("Test error".into()))
    }

    Causaloid::new(id, causal_fn, description)
}

pub fn get_test_context() -> BaseContext {
    let mut context = Context::with_capacity(1, "Test-Context", 10);

    let id = 1;
    let root = Root::new(id);
    let contextoid = Contextoid::new(id, ContextoidType::Root(root));
    context.add_node(contextoid).expect("Failed to add node");

    context
}

pub fn get_test_inferable(id: IdentificationValue, inverse: bool) -> Inference {
    let question = "".to_string() as DescriptionValue;
    let all_obs = get_test_obs_vec();

    if inverse {
        let target_threshold = 11.0;
        let target_effect = 0.0;
        let observation = all_obs.percent_observation(target_threshold, target_effect);
        let threshold = 0.55;
        let effect = 0.0; // false
        let target = 0.0; // false

        Inference::new(id, question, observation, threshold, effect, target)
    } else {
        let target_threshold = 10.0;
        let target_effect = 1.0;
        let observation = all_obs.percent_observation(target_threshold, target_effect);
        let threshold = 0.55;
        let effect = 1.0; //true
        let target = 1.0; //true

        Inference::new(id, question, observation, threshold, effect, target)
    }
}

pub fn get_test_observation() -> Observation {
    Observation::new(0, 14.0, 1.0)
}

pub fn get_test_assumption() -> Assumption {
    let id: IdentificationValue = 1;
    let description: String = "Test assumption that data are there".to_string() as DescriptionValue;
    let assumption_fn: EvalFn = test_fn_has_data;

    Assumption::new(id, description, assumption_fn)
}

fn test_fn_has_data(data: &[PropagatingEffect]) -> Result<bool, AssumptionError> {
    Ok(!data.is_empty()) // Data is NOT empty i.e. true when it is 
}

pub fn get_test_assumption_false() -> Assumption {
    let id: IdentificationValue = 2;
    let description: String =
        "Test assumption that is always false".to_string() as DescriptionValue;
    let assumption_fn: EvalFn = test_fn_is_false;
    Assumption::new(id, description, assumption_fn)
}

fn test_fn_is_false(_data: &[PropagatingEffect]) -> Result<bool, AssumptionError> {
    Ok(false)
}

pub fn get_test_assumption_error() -> Assumption {
    let id: IdentificationValue = 2;
    let description: String =
        "Test assumption that raises an error".to_string() as DescriptionValue;
    let assumption_fn: EvalFn = test_fn_is_error;
    Assumption::new(id, description, assumption_fn)
}

fn test_fn_is_error(_data: &[PropagatingEffect]) -> Result<bool, AssumptionError> {
    Err(AssumptionError::AssumptionFailed(String::from(
        "Test error",
    )))
}

pub fn get_test_num_array() -> [NumericalValue; 10] {
    [8.4, 8.5, 9.1, 9.3, 9.4, 9.5, 9.7, 9.7, 9.9, 9.9]
}
