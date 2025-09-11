/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_data_structures::CausalTensor;
use std::collections::HashMap;
use std::fmt::Debug;

/// A structured result for the SURD state decomposition.
///
/// This provides a type-safe and clear way to access the various components
/// of the causal decomposition.
#[derive(Debug)]
pub struct SurdResult<T> {
    redundant_info: HashMap<Vec<usize>, f64>,
    synergistic_info: HashMap<Vec<usize>, f64>,
    mutual_info: HashMap<Vec<usize>, f64>,
    info_leak: f64,
    redundant_states: HashMap<Vec<usize>, CausalTensor<T>>,
    unique_states: HashMap<Vec<usize>, CausalTensor<T>>,
    synergistic_states: HashMap<Vec<usize>, CausalTensor<T>>,
}

impl<T> SurdResult<T> {
    pub fn new(
        redundant_info: HashMap<Vec<usize>, f64>,
        synergistic_info: HashMap<Vec<usize>, f64>,
        mutual_info: HashMap<Vec<usize>, f64>,
        info_leak: f64,
        redundant_states: HashMap<Vec<usize>, CausalTensor<T>>,
        unique_states: HashMap<Vec<usize>, CausalTensor<T>>,
        synergistic_states: HashMap<Vec<usize>, CausalTensor<T>>,
    ) -> Self {
        Self {
            redundant_info,
            synergistic_info,
            mutual_info,
            info_leak,
            redundant_states,
            unique_states,
            synergistic_states,
        }
    }
}

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

    pub fn redundant_states(&self) -> &HashMap<Vec<usize>, CausalTensor<T>> {
        &self.redundant_states
    }

    pub fn unique_states(&self) -> &HashMap<Vec<usize>, CausalTensor<T>> {
        &self.unique_states
    }

    pub fn synergistic_states(&self) -> &HashMap<Vec<usize>, CausalTensor<T>> {
        &self.synergistic_states
    }
}

impl<T> std::fmt::Display for SurdResult<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "--- SURD Decomposition Result ---")?;
        writeln!(f, "Redundant Information: {:?}", self.redundant_info)?;
        writeln!(f, "Synergistic Information: {:?}", self.synergistic_info)?;
        writeln!(f, "Mutual Information: {:?}", self.mutual_info)?;
        writeln!(f, "Information Leak: {}", self.info_leak)?;
        writeln!(
            f,
            "Redundant States: {:?}",
            self.redundant_states.keys().collect::<Vec<_>>()
        )?;
        writeln!(
            f,
            "Unique States: {:?}",
            self.unique_states.keys().collect::<Vec<_>>()
        )?;
        writeln!(
            f,
            "Synergistic States: {:?}",
            self.synergistic_states.keys().collect::<Vec<_>>()
        )?;
        Ok(())
    }
}
