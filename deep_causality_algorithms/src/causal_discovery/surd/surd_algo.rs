/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::causal_discovery::surd::surd_utils;
use crate::causal_discovery::surd::{MaxOrder, SurdResult};
use deep_causality_tensor::CausalTensor;
use deep_causality_tensor::CausalTensorCollectionExt;
use deep_causality_tensor::CausalTensorError;
use deep_causality_tensor::CausalTensorMathExt;
use std::collections::HashMap;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Decomposes mutual information into its Synergistic, Unique, and Redundant components
/// for each state of the target variable, based on the SURD-states algorithm.
///
/// The surd_states algorithm, being based on information theory, fundamentally operates on
/// probability distributions of discrete random variables. If the input data is continuous, it
/// must be discretized into bins to form the joint probability distribution that
/// the algorithm consumes. You may  employed a uniform partition based method on the values on the data.
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
/// # Performance Improvements
/// To make the algorithm practical for high-dimensional data, several performance improvements have been implemented:
/// 1.  **Algorithmic Capping**: The `max_order` parameter allows limiting the analysis to a tractable number
///     of interactions (e.g., pairwise, `k=2`), changing the complexity from exponential `O(2^N)` to polynomial `O(N^k)`.
///     This is the most most significant optimization for datasets with many variables.
/// 2.  **Parallel Execution**: When compiled with the `parallel` feature flag, the main decomposition loop runs in parallel
///     across all available CPU cores using the `rayon` crate. This provides a near-linear speedup for datasets
///     with a large number of target states.
/// 3.  **Memory Optimization**: Unnecessary cloning of large data tensors within the hot loop has been eliminated
///     by using references, reducing memory pressure and improving cache performance.
///
/// # Arguments
/// * `p_raw` - A `CausalTensor` representing the joint probability distribution.
///   **Crucially, this must be a joint probability distribution of discrete, binned data.**
///   The first dimension (axis 0) must correspond to the target variable.
/// * `max_order` - An enum specifying the maximum order of interactions to compute.
///
/// # Returns
/// A `Result` containing a `SurdResult` struct with the full decomposition, including
/// separate causal (positive) and non-causal (negative) state-dependent maps.
///
/// # Examples
///
/// ```
/// use deep_causality_algorithms::causal_discovery::surd::{surd_states, MaxOrder};
/// use deep_causality_tensor::CausalTensor;
///
/// // Create a joint probability distribution for a target and 2 source variables.
/// // Shape: [target_states, source1_states, source2_states] = [2, 2, 2]
/// let data = vec![
///     0.1, 0.2, // P(T=0, S1=0, S2=0), P(T=0, S1=0, S2=1)
///     0.0, 0.2, // P(T=0, S1=1, S2=0), P(T=0, S1=1, S2=1)
///     0.3, 0.0, // P(T=1, S1=0, S2=0), P(T=1, S1=0, S2=1)
///     0.1, 0.1, // P(T=1, S1=1, S2=0), P(T=1, S1=1, S2=1)
/// ];
/// let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
///
/// // Perform a full decomposition (k=N=2)
/// let full_result = surd_states(&p_raw, MaxOrder::Max).unwrap();
///
/// // Check some results (synergistic info for pair {1, 2})
/// assert!(full_result.synergistic_info().get(&vec![1, 2]).is_some());
///
/// // Perform a partial, pairwise decomposition (k=2)
/// // This is equivalent to MaxOrder::Max for N=2, but demonstrates the API.
/// let partial_result = surd_states(&p_raw, MaxOrder::Min).unwrap();
/// assert!(partial_result.synergistic_info().get(&vec![1, 2]).is_some());
/// ```
pub fn surd_states(
    p_raw: &CausalTensor<f64>,
    max_order: MaxOrder,
) -> Result<SurdResult<f64>, CausalTensorError> {
    if p_raw.is_empty() {
        return Err(CausalTensorError::EmptyTensor);
    }

    // --- 1. Pre-processing: Normalize the probability distribution ---
    let mut p_data = p_raw.as_slice().to_vec();
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
    let k = max_order.get_k_max_order(n_vars)?;

    // --- 2. Calculate Information Leak ---
    let h = surd_utils::entropy_nvars(&p, &[0])?;
    let hc = surd_utils::cond_entropy(&p, &[0], &agent_indices)?;

    let info_leak = if h > 1e-14 {
        (hc / h).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // --- 3. Compute Specific and Mutual Information ---
    let mut combs: Vec<Vec<usize>> = Vec::new();
    for i in 1..=k.min(n_vars) {
        let combinations_for_i = surd_utils::combinations(&agent_indices, i);
        combs.extend(combinations_for_i);
    }

    let p_s = p.sum_axes(&agent_indices)?;
    let mut is_map: HashMap<Vec<usize>, CausalTensor<f64>> = HashMap::new();
    for j_comb in &combs {
        let noj: Vec<usize> = agent_indices
            .iter()
            .filter(|&ax| !j_comb.contains(ax))
            .cloned()
            .collect();

        // p(T, j) - marginal distribution of target and the current combination of sources
        let p_as = if noj.is_empty() {
            p.clone() // Workaround for CausalTensor::sum_axes(&[]) returning scalar
        } else {
            p.sum_axes(&noj)?
        };

        // p(j) - marginal distribution of the current combination of sources
        // To get this, we first marginalize out the target, then sum over the other source axes.
        let p_sources = p.sum_axes(&[0])?;
        let noj_mapped: Vec<usize> = noj.iter().map(|ax| ax - 1).collect();

        let p_j = if noj_mapped.is_empty() {
            p_sources.clone() // Workaround for CausalTensor::sum_axes(&[]) returning scalar
        } else {
            p_sources.sum_axes(&noj_mapped)?
        };

        // p(T | j) = p(T, j) / p(j)
        // let p_s_a = (&p_as / &p_j)?;
        let p_s_a = p_as.safe_div(&p_j)?;

        // p(j | T) = p(T, j) / p(T)
        // let p_a_s = (&p_as / &p_s)?;
        let p_a_s = p_as.safe_div(&p_s)?;

        // Reshape p_s to allow for correct broadcasting against p_s_a.
        // The shape of p_s is [n_target_states], and p_s_a is [n_target_states, ...source_dims].
        // We need to make p_s compatible for element-wise ops, so we give it trailing `1`s.
        let mut broadcast_shape = vec![1; p_s_a.num_dim()];
        if !broadcast_shape.is_empty() {
            broadcast_shape[0] = p_s.shape()[0];
        }
        let p_s_reshaped = p_s.reshape(&broadcast_shape)?;

        // surd_log2 returns 0.0 for inputs of 0.0, preventing NaN and Inf.
        let log_diff = (p_s_a.surd_log2()? - p_s_reshaped.surd_log2()?)?;
        let specific_info_map = (&p_a_s * &log_diff)?;

        // Sum over agent dimensions corresponding to j_comb.
        // The specific_info_map has shape [n_target_states, agent_dim1, agent_dim2, ...]
        // The dimensions corresponding to the variables in j_comb are now re-indexed from 1.
        let dims_to_sum_for_j_comb: Vec<usize> = (1..=j_comb.len()).collect();
        let sum_axes = specific_info_map
            .sum_axes(&dims_to_sum_for_j_comb)
            .expect("Failed to sum agent axes for specific_info_map");

        let ravel = sum_axes.ravel();
        is_map.insert(j_comb.clone(), ravel);
    }

    let mi: HashMap<Vec<usize>, f64> = is_map
        .iter()
        .map(|(k, v)| Ok((k.clone(), (v * &p_s)?.as_slice().iter().sum())))
        .collect::<Result<_, CausalTensorError>>()?;

    // --- 4. Main Decomposition Loop ---
    let results_per_target: Vec<_> = {
        #[cfg(feature = "parallel")]
        {
            (0..n_target_states)
                .into_par_iter()
                .map(|t| {
                    analyze_single_target_state(
                        t,
                        n_vars,
                        &combs,
                        &is_map,
                        &p_s,
                        &p,
                        &agent_indices,
                        k,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?
        }
        #[cfg(not(feature = "parallel"))]
        {
            (0..n_target_states)
                .map(|t| {
                    analyze_single_target_state(
                        t,
                        n_vars,
                        &combs,
                        &is_map,
                        &p_s,
                        &p,
                        &agent_indices,
                        k,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?
        }
    };

    // --- 5. Merge results from all target states ---
    let mut i_r = HashMap::new();
    let mut i_s = HashMap::new();
    let mut temp_causal_rd_states: HashMap<Vec<usize>, Vec<CausalTensor<f64>>> = HashMap::new();
    let mut temp_causal_un_states: HashMap<Vec<usize>, Vec<CausalTensor<f64>>> = HashMap::new();
    let mut temp_causal_sy_states: HashMap<Vec<usize>, Vec<CausalTensor<f64>>> = HashMap::new();
    let mut temp_non_causal_rd_states: HashMap<Vec<usize>, Vec<CausalTensor<f64>>> = HashMap::new();
    let mut temp_non_causal_un_states: HashMap<Vec<usize>, Vec<CausalTensor<f64>>> = HashMap::new();
    let mut temp_non_causal_sy_states: HashMap<Vec<usize>, Vec<CausalTensor<f64>>> = HashMap::new();

    for result in results_per_target {
        for (k, v) in result.i_r {
            *i_r.entry(k).or_insert(0.0) += v;
        }
        for (k, v) in result.i_s {
            *i_s.entry(k).or_insert(0.0) += v;
        }
        for (k, v) in result.causal_rd_states {
            temp_causal_rd_states.entry(k).or_default().push(v);
        }
        for (k, v) in result.causal_un_states {
            temp_causal_un_states.entry(k).or_default().push(v);
        }
        for (k, v) in result.causal_sy_states {
            temp_causal_sy_states.entry(k).or_default().push(v);
        }
        for (k, v) in result.non_causal_rd_states {
            temp_non_causal_rd_states.entry(k).or_default().push(v);
        }
        for (k, v) in result.non_causal_un_states {
            temp_non_causal_un_states.entry(k).or_default().push(v);
        }
        for (k, v) in result.non_causal_sy_states {
            temp_non_causal_sy_states.entry(k).or_default().push(v);
        }
    }

    // --- 6. Finalize State-Dependent Maps by Stacking ---
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

/// A private struct to hold the results of analyzing a single target state.
struct PerTargetStateResults {
    i_r: HashMap<Vec<usize>, f64>,
    i_s: HashMap<Vec<usize>, f64>,
    causal_rd_states: HashMap<Vec<usize>, CausalTensor<f64>>,
    causal_un_states: HashMap<Vec<usize>, CausalTensor<f64>>,
    causal_sy_states: HashMap<Vec<usize>, CausalTensor<f64>>,
    non_causal_rd_states: HashMap<Vec<usize>, CausalTensor<f64>>,
    non_causal_un_states: HashMap<Vec<usize>, CausalTensor<f64>>,
    non_causal_sy_states: HashMap<Vec<usize>, CausalTensor<f64>>,
}

#[allow(clippy::too_many_arguments)]
fn analyze_single_target_state(
    t: usize,
    n_vars: usize,
    combs: &[Vec<usize>],
    is_map: &HashMap<Vec<usize>, CausalTensor<f64>>,
    p_s: &CausalTensor<f64>,
    p: &CausalTensor<f64>,
    agent_indices: &[usize],
    _k: usize, // k is now implicitly handled by the length of combs
) -> Result<PerTargetStateResults, CausalTensorError> {
    let mut i_r = HashMap::new();
    let mut i_s = HashMap::new();
    let mut causal_rd_states = HashMap::new();
    let mut causal_un_states = HashMap::new();
    let mut causal_sy_states = HashMap::new();
    let mut non_causal_rd_states = HashMap::new();
    let mut non_causal_un_states = HashMap::new();
    let mut non_causal_sy_states = HashMap::new();

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
                .filter(|&(_, &len)| len == l)
                .map(|(&val, _)| val)
                .fold(f64::NEG_INFINITY, f64::max);
            if max_prev_level.is_finite() {
                i1_sorted
                    .iter_mut()
                    .zip(&lens)
                    .filter(|&(_, &len)| len == l + 1)
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
    let final_lab: Vec<Vec<usize>> = new_sorted_indices.iter().map(|&i| lab[i].clone()).collect();
    let di_values = surd_utils::diff(&final_i1);

    // Find the index of the last single-variable term with a significant info contribution.
    let last_single_var_idx = final_lab
        .iter()
        .rposition(|lab| lab.len() == 1)
        .unwrap_or(usize::MAX);

    let mut red_vars: Vec<usize> = (1..=n_vars).collect();

    for (i, ll) in final_lab.iter().enumerate() {
        let info = di_values[i] * p_s.as_slice()[t];

        if info.abs() < 1e-14 {
            continue;
        }

        let prev_ll: &[usize] = if i == 0 { &[] } else { &final_lab[i - 1] };
        let (causal_slice, non_causal_slice) =
            calculate_state_slice(p, ll, prev_ll, t, agent_indices, n_vars)?;

        if ll.len() > 1
        // Synergistic states
        {
            causal_sy_states.insert(ll.clone(), causal_slice);
            non_causal_sy_states.insert(ll.clone(), non_causal_slice);
            *i_s.entry(ll.clone()).or_insert(0.0) += info;
        } else if ll.len() == 1 {
            // Unique vs Redundant states
            if i == last_single_var_idx
            // This is the highest-ordered single-variable term, so it's UNIQUE.
            {
                causal_un_states.insert(ll.clone(), causal_slice);
                non_causal_un_states.insert(ll.clone(), non_causal_slice);
            } else
            // Lower-ordered single-variable terms contribute to REDUNDANCY.
            // The original logic only stored one redundant slice, so we'll model that.
            // This will capture the state map for the main redundant term.
            {
                causal_rd_states.insert(red_vars.clone(), causal_slice);
                non_causal_rd_states.insert(red_vars.clone(), non_causal_slice);
            }
            // Aggregate calculation for I_R
            *i_r.entry(red_vars.clone()).or_insert(0.0) += info;
            red_vars.retain(|&v| v != ll[0]);
        }
    }

    Ok(PerTargetStateResults {
        i_r,
        i_s,
        causal_rd_states,
        causal_un_states,
        causal_sy_states,
        non_causal_rd_states,
        non_causal_un_states,
        non_causal_sy_states,
    })
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
    n_vars: usize,
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

    // To calculate p(i), we marginalize out the target variable and then all other source variables.
    let source_axes: Vec<usize> = (0..n_vars).collect();
    let current_vars_mapped_for_marginal: Vec<usize> =
        current_vars.iter().map(|&ax| ax - 1).collect();
    let axes_to_sum_for_pi =
        surd_utils::set_difference(&source_axes, &current_vars_mapped_for_marginal);
    let p_i = p.sum_axes(&[0])?.sum_axes(&axes_to_sum_for_pi)?;

    let p_target_given_i = p_ti.safe_div(&p_i)?;

    let p_target_given_j = if prev_vars.is_empty() {
        p.sum_axes(agent_indices)?
    } else {
        let p_tj = p_slice.sum_axes(&surd_utils::set_difference(
            &all_vars_mapped,
            &prev_vars_mapped,
        ))?;

        let prev_vars_mapped_for_marginal: Vec<usize> =
            prev_vars.iter().map(|&ax| ax - 1).collect();
        let axes_to_sum_for_pj =
            surd_utils::set_difference(&source_axes, &prev_vars_mapped_for_marginal);
        let p_j = p.sum_axes(&[0])?.sum_axes(&axes_to_sum_for_pj)?;

        p_tj.safe_div(&p_j)?
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

    let causal_slice = (&p_tij * &causal_log_ratio)?;
    let non_causal_slice = (&p_tij * &non_causal_log_ratio)?;

    Ok((causal_slice, non_causal_slice))
}
