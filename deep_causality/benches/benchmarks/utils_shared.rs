/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::CausalityError;
use deep_causality::{
    BaseCausaloid, CausalEffectLog, CausalFnOutput, Causaloid, IdentificationValue, NumericalValue,
};

pub fn get_test_causaloid() -> BaseCausaloid<f64, bool> {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";

    fn causal_fn(evidence: f64) -> Result<CausalFnOutput<bool>, CausalityError> {
        let mut log = CausalEffectLog::new();
        log.add_entry(&format!("Processing evidence: {}", evidence));

        if evidence.is_sign_negative() {
            log.add_entry("Observation is negative, returning error.");
            return Err(CausalityError("Observation is negative".into()));
        }

        let threshold: NumericalValue = 0.55;
        let is_active = evidence.ge(&threshold);
        log.add_entry(&format!(
            "Evidence {} >= threshold {}: {}",
            evidence, threshold, is_active
        ));

        Ok(CausalFnOutput {
            output: is_active,
            log,
        })
    }

    Causaloid::new(id, causal_fn, description)
}

pub fn generate_sample_data<const N: usize>() -> [f64; N] {
    [0.99; N]
}
