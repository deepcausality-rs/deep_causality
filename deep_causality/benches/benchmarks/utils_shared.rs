/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    BaseCausaloid, Causaloid, IdentificationValue, NumericalValue, PropagatingEffect,
};
use deep_causality::{CausalityError, EffectValue};

pub fn get_test_causaloid() -> BaseCausaloid {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";

    fn causal_fn(evidence: EffectValue) -> PropagatingEffect {
        let obs = match evidence {
            EffectValue::Numerical(val) => val,
            _ => {
                return PropagatingEffect::from_error(CausalityError(
                    "Causaloid Expected Numerical evidence. Found other variant as input".into(),
                ));
            }
        };

        if obs.is_sign_negative() {
            return PropagatingEffect::from_error(CausalityError("Observation is negative".into()));
        }

        let threshold: NumericalValue = 0.55;
        let is_active = obs.ge(&threshold);

        PropagatingEffect::from_deterministic(is_active)
    }

    Causaloid::new(id, causal_fn, description)
}

pub fn generate_sample_data<const N: usize>() -> [f64; N] {
    [0.99; N]
}
