/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ActionParameterValue, CausalityError, UncertainParameter};
use deep_causality_core::CausalityErrorEnum;
use deep_causality_uncertain::{UncertainBool, UncertainF64};

/// Trait for types that can be evaluated to a boolean decision in a CSM.
pub trait CsmEvaluable {
    /// Determines if the state is active based on the value and optional parameters.
    fn is_active(&self, params: Option<&UncertainParameter>) -> Result<bool, CausalityError>;
    /// Converts the value to an ActionParameterValue for use in actions.
    fn to_action_param(&self) -> ActionParameterValue;
}

impl CsmEvaluable for bool {
    fn is_active(&self, _params: Option<&UncertainParameter>) -> Result<bool, CausalityError> {
        Ok(*self)
    }

    fn to_action_param(&self) -> ActionParameterValue {
        ActionParameterValue::Boolean(*self)
    }
}

impl CsmEvaluable for UncertainBool {
    fn is_active(&self, params: Option<&UncertainParameter>) -> Result<bool, CausalityError> {
        if let Some(p) = params {
            self.probability_exceeds(p.threshold(), p.confidence(), p.epsilon(), p.max_samples())
                .map_err(|e| {
                    CausalityError(CausalityErrorEnum::Custom(format!(
                        "Failed to evaluate uncertain boolean: {}",
                        e
                    )))
                })
        } else {
            self.implicit_conditional().map_err(|e| {
                CausalityError(CausalityErrorEnum::Custom(format!(
                    "Failed to evaluate uncertain boolean: {}",
                    e
                )))
            })
        }
    }

    fn to_action_param(&self) -> ActionParameterValue {
        ActionParameterValue::Boolean(self.value())
    }
}

impl CsmEvaluable for UncertainF64 {
    fn is_active(&self, params: Option<&UncertainParameter>) -> Result<bool, CausalityError> {
        if let Some(p) = params {
            let comparison = self.greater_than(p.threshold());
            comparison
                .probability_exceeds(0.5, p.confidence(), p.epsilon(), p.max_samples())
                .map_err(|e| {
                    CausalityError(CausalityErrorEnum::Custom(format!(
                        "Failed to evaluate uncertain float: {}",
                        e
                    )))
                })
        } else {
            Err(CausalityError(CausalityErrorEnum::Custom(
                "UncertainFloat effect requires UncertainParameter on CausalState".into(),
            )))
        }
    }

    fn to_action_param(&self) -> ActionParameterValue {
        ActionParameterValue::Number(self.value())
    }
}
