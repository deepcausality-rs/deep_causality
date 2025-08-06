/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Causable, CausalityError};

pub(in crate::traits::causable_collection) fn _explain<T: Causable>(
    causes: Vec<&T>,
) -> Result<String, CausalityError> {
    if causes.is_empty() {
        return Err(CausalityError::new(
            "Causal Collection is empty".to_string(),
        ));
    }

    let mut explanation = String::new();
    for cause in causes {
        let cause_explanation = match cause.explain() {
            Ok(s) => s,
            Err(e) => {
                return Err(CausalityError(format!(
                    "[Error explaining cause {} ('{}')]",
                    cause.id(),
                    e
                )));
            }
        };

        explanation.push('\n');
        explanation.push_str(format!(" * {cause_explanation}").as_str());
        explanation.push('\n');
    }
    Ok(explanation)
}
