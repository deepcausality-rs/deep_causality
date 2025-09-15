/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_data_structures::CausalTensorError;
use std::fmt::{Display, Formatter};

/// Defines the maximum order of interactions to consider in the SURD analysis.
#[derive(Debug, Clone, Copy, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub enum MaxOrder {
    /// Analyze only single variables and pairwise interactions (k=2).
    Min,
    /// Analyze interactions up to a specific order `k`.
    Some(usize),
    /// Perform the full decomposition, analyzing all 2^N - 1 interactions.
    Max,
}

impl MaxOrder {
    /// Resolves the `MaxOrder` enum variant into an actual `k` value, representing the maximum
    /// order of interactions to consider.
    ///
    /// This function also performs crucial safety checks to ensure the `k` value is valid
    /// in the context of the number of source variables (`n_vars`).
    ///
    /// # Arguments
    ///
    /// * `n_vars` - The total number of source variables in the system.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(usize)`: The resolved `k` value if the operation is successful and all checks pass.
    /// - `Err(CausalTensorError)`: An error if the `k` value is invalid (e.g., less than 2,
    ///   or greater than `n_vars`).
    pub fn get_k_max_order(&self, n_vars: usize) -> Result<usize, CausalTensorError> {
        match self {
            MaxOrder::Min => Ok(2),
            MaxOrder::Max => Ok(n_vars),
            MaxOrder::Some(val) => {
                if val < &2 {
                    return Err(CausalTensorError::InvalidParameter(
                        "Max order k must be at least 2.".to_string(),
                    ));
                }
                if val > &n_vars {
                    return Err(CausalTensorError::InvalidParameter(format!(
                        "Max order k ({}) cannot be greater than the number of source variables ({}).",
                        val, n_vars
                    )));
                }
                Ok(*val)
            }
        }
    }
}

impl Display for MaxOrder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MaxOrder::Min => write!(f, "Min"),
            MaxOrder::Some(k) => write!(f, "Some({})", k),
            MaxOrder::Max => write!(f, "Max"),
        }
    }
}
