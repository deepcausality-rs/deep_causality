/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use std::collections::HashMap;
use std::fmt::Debug;

/// A structured result for the SURD state decomposition.
///
/// This provides a type-safe and clear way to access the various components
/// of the causal decomposition, separating causal (positive) and non-causal (negative)
/// state-dependent information increments.
#[derive(Debug)]
pub struct SurdResult<T> {
    // Aggregate information (total increments)
    redundant_info: HashMap<Vec<usize>, f64>,
    synergistic_info: HashMap<Vec<usize>, f64>,
    mutual_info: HashMap<Vec<usize>, f64>,
    info_leak: f64,

    // State-dependent causal maps (positive increments)
    causal_redundant_states: HashMap<Vec<usize>, CausalTensor<T>>,
    causal_unique_states: HashMap<Vec<usize>, CausalTensor<T>>,
    causal_synergistic_states: HashMap<Vec<usize>, CausalTensor<T>>,

    // State-dependent non-causal maps (negative increments)
    non_causal_redundant_states: HashMap<Vec<usize>, CausalTensor<T>>,
    non_causal_unique_states: HashMap<Vec<usize>, CausalTensor<T>>,
    non_causal_synergistic_states: HashMap<Vec<usize>, CausalTensor<T>>,
}

impl<T> SurdResult<T> {
    #[allow(clippy::too_many_arguments)] // Internal constructor with many fields is acceptable here
    pub fn new(
        redundant_info: HashMap<Vec<usize>, f64>,
        synergistic_info: HashMap<Vec<usize>, f64>,
        mutual_info: HashMap<Vec<usize>, f64>,
        info_leak: f64,
        causal_redundant_states: HashMap<Vec<usize>, CausalTensor<T>>,
        causal_unique_states: HashMap<Vec<usize>, CausalTensor<T>>,
        causal_synergistic_states: HashMap<Vec<usize>, CausalTensor<T>>,
        non_causal_redundant_states: HashMap<Vec<usize>, CausalTensor<T>>,
        non_causal_unique_states: HashMap<Vec<usize>, CausalTensor<T>>,
        non_causal_synergistic_states: HashMap<Vec<usize>, CausalTensor<T>>,
    ) -> Self {
        Self {
            redundant_info,
            synergistic_info,
            mutual_info,
            info_leak,
            causal_redundant_states,
            causal_unique_states,
            causal_synergistic_states,
            non_causal_redundant_states,
            non_causal_unique_states,
            non_causal_synergistic_states,
        }
    }
}

// --- Getters for aggregate information ---
impl<T> SurdResult<T> {
    pub fn redundant_info(&self) -> &HashMap<Vec<usize>, f64> {
        &self.redundant_info
    }

    pub fn synergistic_info(&self) -> &HashMap<Vec<usize>, f64> {
        &self.synergistic_info
    }

    pub fn mutual_info(&self) -> &HashMap<Vec<usize>, f64> {
        &self.mutual_info
    }

    pub fn info_leak(&self) -> f64 {
        self.info_leak
    }
}

// --- Getters for state-dependent causal maps ---
impl<T> SurdResult<T> {
    pub fn causal_redundant_states(&self) -> &HashMap<Vec<usize>, CausalTensor<T>> {
        &self.causal_redundant_states
    }

    pub fn causal_unique_states(&self) -> &HashMap<Vec<usize>, CausalTensor<T>> {
        &self.causal_unique_states
    }

    pub fn causal_synergistic_states(&self) -> &HashMap<Vec<usize>, CausalTensor<T>> {
        &self.causal_synergistic_states
    }
}

// --- Getters for state-dependent non-causal maps ---
impl<T> SurdResult<T> {
    pub fn non_causal_redundant_states(&self) -> &HashMap<Vec<usize>, CausalTensor<T>> {
        &self.non_causal_redundant_states
    }

    pub fn non_causal_unique_states(&self) -> &HashMap<Vec<usize>, CausalTensor<T>> {
        &self.non_causal_unique_states
    }

    pub fn non_causal_synergistic_states(&self) -> &HashMap<Vec<usize>, CausalTensor<T>> {
        &self.non_causal_synergistic_states
    }
}

impl<T> std::fmt::Display for SurdResult<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "--- SURD Decomposition Result ---")?;
        writeln!(f, "Aggregate Redundant Info: {:?}", self.redundant_info)?;
        writeln!(f, "Aggregate Synergistic Info: {:?}", self.synergistic_info)?;
        writeln!(f, "Aggregate Mutual Info: {:?}", self.mutual_info)?;
        writeln!(f, "Information Leak: {}", self.info_leak)?;
        writeln!(f, "--- State-Dependent Maps ---")?;
        writeln!(
            f,
            "Causal Redundant States: {:?}",
            self.causal_redundant_states.keys().collect::<Vec<_>>()
        )
        .expect("Failed to write causal_redundant_states to formatter");
        writeln!(
            f,
            "Causal Unique States: {:?}",
            self.causal_unique_states.keys().collect::<Vec<_>>()
        )
        .expect("Failed to write causal_unique_states to formatter");
        writeln!(
            f,
            "Causal Synergistic States: {:?}",
            self.causal_synergistic_states.keys().collect::<Vec<_>>()
        )
        .expect("Failed to write causal_synergistic_states to formatter");
        writeln!(
            f,
            "Non-Causal Redundant States: {:?}",
            self.non_causal_redundant_states.keys().collect::<Vec<_>>()
        )
        .expect("Failed to write non_causal_unique_states to formatter");
        writeln!(
            f,
            "Non-Causal Unique States: {:?}",
            self.non_causal_unique_states.keys().collect::<Vec<_>>()
        )
        .expect("Failed to write non_causal_unique_states to formatter");
        writeln!(
            f,
            "Non-Causal Synergistic States: {:?}",
            self.non_causal_synergistic_states
                .keys()
                .collect::<Vec<_>>()
        )?;
        Ok(())
    }
}
