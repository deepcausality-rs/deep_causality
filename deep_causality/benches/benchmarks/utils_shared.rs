/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::errors::CausalityError;
use deep_causality::{
    BaseCausaloid, Causaloid, IdentificationValue, NumericalValue, PropagatingEffect,
};

pub fn get_test_causaloid() -> BaseCausaloid {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";

    fn causal_fn(evidence: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        let obs = match evidence {
            PropagatingEffect::Numerical(val) => *val,
            _ => return Err(CausalityError("Expected Numerical evidence.".into())),
        };

        if obs.is_sign_negative() {
            return Err(CausalityError("Observation is negative".into()));
        }

        let threshold: NumericalValue = 0.55;
        let is_active = obs.ge(&threshold);

        Ok(PropagatingEffect::Deterministic(is_active))
    }

    Causaloid::new(id, causal_fn, description)
}

pub fn generate_sample_data<const N: usize>() -> [f64; N] {
    [0.99; N]
}
