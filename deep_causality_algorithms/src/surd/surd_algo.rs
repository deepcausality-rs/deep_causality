/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::surd::surd_utils;
use crate::SurdResult;
use deep_causality_data_structures::{
    CausalTensor, CausalTensorCollectionExt, CausalTensorError, CausalTensorLogMathExt,
};
use std::collections::HashMap;

/// Decomposes mutual information into its Synergistic, Unique, and Redundant components
/// for each state of the target variable, based on the SURD-states algorithm.
///
/// This is a high-performance Rust port of the SURD-State algorithm described in the paper
/// "Observational causality by states and interaction type for scientific discovery" (martínezsánchez2025).
/// It prioritizes mathematical faithfulness to the original paper and performance.
///
/// The algorithm follows these main steps:
/// 1.  **Pre-processing**: The input probability distribution is validated and normalized.
/// 2.  **Information Leak Calculation**: The causality leak, representing the influence of unobserved
///     variables, is calculated as `H(Q_f|Q) / H(Q_f)`.
/// 3.  **Specific Mutual Information**: For every combination of source variables, the specific mutual
///     information `i(Q_f=t; Q_i)` is calculated for each target state `t`. This is based on
///     (martínezsánchez2025, Supplementary Material, Eq. S6).
/// 4.  **Decomposition Loop**: For each target state `t`, the specific information values are sorted.
///     This ordering is crucial for the decomposition. An information-theoretic exclusion principle
///     is applied to prevent double-counting, as described in (martínezsánchez2025, Fig. S2).
/// 5.  **State-Dependent Slice Calculation**: The sorted information values are differenced, and these
///     increments are used to calculate the state-dependent causal maps for redundant, unique, and
///     synergistic components. This step implements the core logic from (martínezsánchez2025,
///     Supplementary Material, Eqs. S17, S22, S28), separating positive (causal) and negative (non-causal) contributions.
/// 6.  **Aggregation**: The state-dependent maps are stacked, and the aggregate SURD values are
///     calculated by summing the increments over all target states, consistent with (martínezsánchez2025, Eq. 7).
///
/// # Arguments
/// * `p_raw` - A `CausalTensor` representing the joint probability distribution.
///   The first dimension (axis 0) must correspond to the target variable.
///
/// # Returns
/// A `Result` containing a `SurdResult` struct with the full decomposition, including
/// separate causal (positive) and non-causal (negative) state-dependent maps.
pub fn surd_states(p_raw: &CausalTensor<f64>) -> Result<SurdResult<f64>, CausalTensorError> {
    if p_raw.is_empty() {
        return Err(CausalTensorError::EmptyTensor);
    }

    // --- 1. Pre-processing: Normalize the probability distribution ---
    let mut p_data = p_raw.as_slice().to_vec();
    p_data.iter_mut().for_each(|x| *x += 1e-14); // Avoid log(0)
    let total_sum: f64 = p_data.iter().sum();
    if total_sum.abs() < 1e-14 {
        return Err(CausalTensorError::InvalidOperation);
    }
    p_data.iter_mut().for_each(|x| *x /= total_sum);
    let p = CausalTensor::new(p_data, p_raw.shape().to_vec())?;

    let n_total_dims = p.num_dim();
    let n_vars = n_total_dims - 1;
    let n_target_states = p.shape()[0];
    let agent_indices: Vec<usize> = (1..n_total_dims).collect();

    // --- 2. Calculate Information Leak ---
    let h = surd_utils::entropy_nvars(&p, &[0])?;
    let hc = surd_utils::cond_entropy(&p, &[0], &agent_indices)?;
    let info_leak = if h > 1e-14 { hc / h } else { 0.0 };

    // --- 3. Compute Specific and Mutual Information ---
    let p_s = p.sum_axes(&agent_indices)?;

    let mut combs: Vec<Vec<usize>> = Vec::new();
    let mut is_map: HashMap<Vec<usize>, CausalTensor<f64>> = HashMap::new();

    for i in 1..=n_vars {
        let combinations_for_i = surd_utils::combinations(&agent_indices, i);

        for j_comb in combinations_for_i {
            let noj: Vec<usize> = agent_indices
                .iter()
                .filter(|&ax| !j_comb.contains(ax))
                .cloned()
                .collect();
            let p_as = p.sum_axes(&noj)?;
            let p_s_a = (&p_as / &p.sum_axes(&[0]).unwrap().sum_axes(&noj)?)?;
            let p_a_s = (&p_as / &p_s)?;
            let log_diff = (p_s_a.log2()? - p_s.log2()?)?;
            let specific_info_map = (p_a_s * &log_diff)?;
            is_map.insert(j_comb.clone(), specific_info_map.sum_axes(&j_comb)?.ravel());
            combs.push(j_comb);
        }
    }

    let mi: HashMap<Vec<usize>, f64> = is_map
        .iter()
        .map(|(k, v)| Ok((k.clone(), (v * &p_s)?.as_slice().iter().sum())))
        .collect::<Result<_, CausalTensorError>>()?;

    // --- 4. Initialize Result Containers ---
    let mut i_r = HashMap::new();
    let mut i_s = HashMap::new();

    // Temporary storage for slices before stacking
    let mut temp_causal_rd_states: HashMap<Vec<usize>, Vec<CausalTensor<f64>>> = HashMap::new();
    let mut temp_causal_un_states: HashMap<Vec<usize>, Vec<CausalTensor<f64>>> = HashMap::new();
    let mut temp_causal_sy_states: HashMap<Vec<usize>, Vec<CausalTensor<f64>>> = HashMap::new();
    let mut temp_non_causal_rd_states: HashMap<Vec<usize>, Vec<CausalTensor<f64>>> = HashMap::new();
    let mut temp_non_causal_un_states: HashMap<Vec<usize>, Vec<CausalTensor<f64>>> = HashMap::new();
    let mut temp_non_causal_sy_states: HashMap<Vec<usize>, Vec<CausalTensor<f64>>> = HashMap::new();

    // --- 5. Main Loop: Decompose for each Target State `t` ---
    for t in 0..n_target_states {
        let i1_values: Vec<f64> = combs.iter().map(|c| is_map[c].as_slice()[t]).collect();
        let i1_sorted_indices = surd_utils::arg_sort(&i1_values);

        let lab: Vec<Vec<usize>> = i1_sorted_indices
            .iter()
            .map(|&i| combs[i].clone())
            .collect();
        let mut i1_sorted: Vec<f64> = i1_sorted_indices.iter().map(|&i| i1_values[i]).collect();
        let lens: Vec<usize> = lab.iter().map(|l| l.len()).collect();

        if let Some(&max_len) = lens.iter().max() {
            for l in 1..max_len {
                let max_prev_level = i1_sorted
                    .iter()
                    .zip(&lens)
                    .filter(|(_, &len)| len == l)
                    .map(|(&val, _)| val)
                    .fold(f64::NEG_INFINITY, f64::max);
                if max_prev_level.is_finite() {
                    i1_sorted
                        .iter_mut()
                        .zip(&lens)
                        .filter(|(_, &len)| len == l + 1)
                        .for_each(|(val, _)| {
                            if *val < max_prev_level {
                                *val = 0.0;
                            }
                        });
                }
            }
        }

        let new_sorted_indices = surd_utils::arg_sort(&i1_sorted);
        let final_i1: Vec<f64> = new_sorted_indices.iter().map(|&i| i1_sorted[i]).collect();
        let final_lab: Vec<Vec<usize>> =
            new_sorted_indices.iter().map(|&i| lab[i].clone()).collect();
        let di_values = surd_utils::diff(&final_i1);

        let num_zeros = di_values.iter().filter(|&&d| d.abs() < 1e-14).count();
        let mut red_vars: Vec<usize> = (1..=n_vars).collect();

        for (i, ll) in final_lab.iter().enumerate() {
            let info = di_values[i] * p_s.as_slice()[t];
            let n_vars_plus_zeros = n_vars + num_zeros;

            if ll.len() == 1 {
                if i == n_vars_plus_zeros.saturating_sub(2) {
                    let prev_ll = final_lab
                        .get(i.saturating_sub(1))
                        .ok_or(CausalTensorError::InvalidOperation)?;
                    let (causal_slice, non_causal_slice) =
                        calculate_state_slice(&p, ll, prev_ll, t, &agent_indices)?;
                    temp_causal_rd_states
                        .entry(red_vars.clone())
                        .or_default()
                        .push(causal_slice);
                    temp_non_causal_rd_states
                        .entry(red_vars.clone())
                        .or_default()
                        .push(non_causal_slice);
                }
                if i == n_vars_plus_zeros.saturating_sub(1) {
                    let prev_ll = final_lab
                        .get(i.saturating_sub(1))
                        .ok_or(CausalTensorError::InvalidOperation)?;
                    let base_is_zero = di_values
                        .get(i.saturating_sub(1))
                        .map_or(true, |&d| d.abs() < 1e-10);

                    let (causal_slice, non_causal_slice) = if base_is_zero {
                        calculate_state_slice(&p, ll, &[], t, &agent_indices)?
                    } else {
                        calculate_state_slice(&p, ll, prev_ll, t, &agent_indices)?
                    };
                    temp_causal_un_states
                        .entry(ll.clone())
                        .or_default()
                        .push(causal_slice);
                    temp_non_causal_un_states
                        .entry(ll.clone())
                        .or_default()
                        .push(non_causal_slice);
                }
            } else if i >= n_vars_plus_zeros {
                let prev_ll = final_lab
                    .get(i.saturating_sub(1))
                    .ok_or(CausalTensorError::InvalidOperation)?;
                let (causal_slice, non_causal_slice) =
                    calculate_state_slice(&p, ll, prev_ll, t, &agent_indices)?;
                temp_causal_sy_states
                    .entry(ll.clone())
                    .or_default()
                    .push(causal_slice);
                temp_non_causal_sy_states
                    .entry(ll.clone())
                    .or_default()
                    .push(non_causal_slice);
            }

            if ll.len() == 1 {
                *i_r.entry(red_vars.clone()).or_insert(0.0) += info;
                red_vars.retain(|&v| v != ll[0]);
            } else {
                *i_s.entry(ll.clone()).or_insert(0.0) += info;
            }
        }
    }

    // --- 6. Finalize State-Dependent Maps by Stacking ---
    // Calculate causal maps
    let causal_redundant_states = temp_causal_rd_states
        .into_iter()
        .map(|(k, slices)| Ok((k, slices.stack(0)?)))
        .collect::<Result<_, _>>()?;

    let causal_unique_states = temp_causal_un_states
        .into_iter()
        .map(|(k, slices)| Ok((k, slices.stack(0)?)))
        .collect::<Result<_, _>>()?;

    let causal_synergistic_states = temp_causal_sy_states
        .into_iter()
        .map(|(k, slices)| Ok((k, slices.stack(0)?)))
        .collect::<Result<_, _>>()?;

    // Calculate non-causal maps
    let non_causal_redundant_states = temp_non_causal_rd_states
        .into_iter()
        .map(|(k, slices)| Ok((k, slices.stack(0)?)))
        .collect::<Result<_, _>>()?;

    let non_causal_unique_states = temp_non_causal_un_states
        .into_iter()
        .map(|(k, slices)| Ok((k, slices.stack(0)?)))
        .collect::<Result<_, _>>()?;

    let non_causal_synergistic_states = temp_non_causal_sy_states
        .into_iter()
        .map(|(k, slices)| Ok((k, slices.stack(0)?)))
        .collect::<Result<_, _>>()?;

    Ok(SurdResult::new(
        i_r,
        i_s,
        mi,
        info_leak,
        causal_redundant_states,
        causal_unique_states,
        causal_synergistic_states,
        non_causal_redundant_states,
        non_causal_unique_states,
        non_causal_synergistic_states,
    ))
}

/// Computes the causal (positive) and non-causal (negative) state-dependent information slices.
///
/// This function implements the core calculation of the state-dependent decomposition. It computes
/// the term `p(t, i, j) * log( p(t|i) / p(t|j) )` and separates the result into two tensors
/// based on the sign of the log ratio. This corresponds to the state-dependent causal (`AC`) and
/// non-causal (`AN`) components in (martínezsánchez2025, Supplementary Material, e.g., Eq. S22 & S24).
///
/// # Arguments
/// * `p` - The full joint probability distribution tensor.
/// * `current_vars` - The set of source variables for the current information term (e.g., `{i}`).
/// * `prev_vars` - The set of source variables from the previous step in the sorted hierarchy (e.g., `{j}`).
/// * `target_state_index` - The index `t` of the target variable's state.
/// * `agent_indices` - A slice containing the indices of all source variables.
///
/// # Returns
/// A tuple `(causal_slice, non_causal_slice)` containing the two resulting tensors.
fn calculate_state_slice(
    p: &CausalTensor<f64>,
    current_vars: &[usize], // Corresponds to `ll` in Python (e.g., [1])
    prev_vars: &[usize],    // Corresponds to `lab[i-1]` (e.g., [2])
    target_state_index: usize,
    agent_indices: &[usize],
) -> Result<(CausalTensor<f64>, CausalTensor<f64>), CausalTensorError> {
    // 1. Get the slice corresponding to the current target state.
    let p_slice = p.slice(0, target_state_index)?;

    // 2. Remap axes since the target dimension (axis 0) is now gone.
    let current_vars_mapped: Vec<usize> = current_vars.iter().map(|&ax| ax - 1).collect();
    let prev_vars_mapped: Vec<usize> = prev_vars.iter().map(|&ax| ax - 1).collect();
    let all_vars_mapped: Vec<usize> = (0..p_slice.num_dim()).collect();

    let p_ti = p_slice.sum_axes(&surd_utils::set_difference(
        &all_vars_mapped,
        &current_vars_mapped,
    ))?;
    let p_i = p
        .sum_axes(&[0])
        .unwrap()
        .sum_axes(&surd_utils::set_difference(agent_indices, current_vars))?;
    let p_target_given_i = (p_ti / p_i)?;

    let p_target_given_j = if prev_vars.is_empty() {
        p.sum_axes(agent_indices)?
    } else {
        let p_tj = p_slice.sum_axes(&surd_utils::set_difference(
            &all_vars_mapped,
            &prev_vars_mapped,
        ))?;
        let p_j = p
            .sum_axes(&[0])
            .unwrap()
            .sum_axes(&surd_utils::set_difference(agent_indices, prev_vars))?;
        (p_tj / p_j)?
    };

    let log_ratio = (p_target_given_i / p_target_given_j)?.log2()?;

    let mut all_involved_vars = current_vars_mapped.to_vec();
    all_involved_vars.extend_from_slice(&prev_vars_mapped);
    all_involved_vars.sort();
    all_involved_vars.dedup();
    let axes_to_sum_out = surd_utils::set_difference(&all_vars_mapped, &all_involved_vars);
    let p_tij = p_slice.sum_axes(&axes_to_sum_out)?;

    // Separate into causal (>0) and non-causal (<0) components
    let causal_log_ratio_data: Vec<f64> =
        log_ratio.as_slice().iter().map(|&v| v.max(0.0)).collect();
    let non_causal_log_ratio_data: Vec<f64> =
        log_ratio.as_slice().iter().map(|&v| v.min(0.0)).collect();

    let causal_log_ratio = CausalTensor::new(causal_log_ratio_data, log_ratio.shape().to_vec())?;
    let non_causal_log_ratio =
        CausalTensor::new(non_causal_log_ratio_data, log_ratio.shape().to_vec())?;

    let causal_slice = (p_tij.clone() * causal_log_ratio)?;
    let non_causal_slice = (p_tij * non_causal_log_ratio)?;

    Ok((causal_slice, non_causal_slice))
}
