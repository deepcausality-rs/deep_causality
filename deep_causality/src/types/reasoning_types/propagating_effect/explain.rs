/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalPropagatingEffect;
use std::fmt::Debug;

impl<Value: Debug, Error: Debug, Log: AsRef<str>> CausalPropagatingEffect<Value, Error, Log> {
    /// Generates a human-readable explanation of the causal computation's history.
    ///
    /// This method iterates over the accumulated logs, providing a comprehensive
    /// history of the computation, including the final value and any errors.
    ///
    /// # Returns
    ///
    /// A `String` containing the formatted explanation.
    pub fn explain(&self) -> String {
        let mut explanation = String::new();

        explanation.push_str(&format!("Final Value: {:?}\n", self.value));

        if let Some(ref error) = self.error {
            explanation.push_str(&format!("Error: {:?}\n", error));
        }

        if !self.logs.is_empty() {
            explanation.push_str("--- Logs ---\n");
            for log_entry in &self.logs {
                explanation.push_str(&format!("{}\n", log_entry.as_ref()));
            }
        }

        explanation
    }
}
