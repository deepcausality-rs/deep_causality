// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::errors::CausalityError;
use deep_causality::prelude::{Causaloid, IdentificationValue, NumericalValue};

use crate::benchmarks::utils_types::Causal;

pub fn get_test_causaloid<'l>() -> Causal<'l> {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    fn causal_fn(obs: NumericalValue) -> Result<bool, CausalityError> {
        if obs.is_sign_negative() {
            return Err(CausalityError("Observation is negative".into()));
        }

        let threshold: NumericalValue = 0.55;
        if !obs.ge(&threshold) {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    Causaloid::new(id, causal_fn, description)
}

pub fn generate_sample_data<const N: usize>() -> [f64; N] {
    [0.99; N]
}
