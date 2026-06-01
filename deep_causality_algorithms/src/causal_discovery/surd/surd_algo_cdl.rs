/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::causal_discovery::surd::surd_utils;
use crate::causal_discovery::surd::surd_utils::surd_utils_cdl;
use crate::causal_discovery::surd::{MaxOrder, SurdResult};
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_tensor::{CausalTensor, CausalTensorError, Tensor};
use std::collections::HashMap;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Decomposes mutual information into its Synergistic, Unique, and Redundant components
/// for each state of the target variable, based on the SURD-states algorithm,
/// specifically designed to handle input tensors with `Option<T>` values.
///
/// This function adapts the core SURD-states algorithm to gracefully manage missing or
/// undefined probability values, represented by `None` in the input `CausalTensor<Option<T>>`.
/// It is generic over the precision type `T: RealField`, so it runs at any supported precision
/// (`f32`, `f64`, `Float106`, …) without code change.
///
/// The algorithm follows these main steps:
/// 1.  **Pre-processing**: The input probability distribution is validated and normalized.
/// 2.  **Information Leak Calculation**: The causality leak, representing the influence of unobserved
///     variables, is calculated as `H(Q_f|Q) / H(Q_f)`.
/// 3.  **Specific Mutual Information**: For every combination of source variables, the specific mutual
///     information `i(Q_f=t; Q_i)` is calculated for each target state `t`.
/// 4.  **Decomposition Loop**: For each target state `t`, the specific information values are sorted.
///     This ordering is crucial for the decomposition. An information-theoretic exclusion principle
///     is applied to prevent double-counting.
/// 5.  **State-Dependent Slice Calculation**: The sorted information values are differenced, and these
///     increments are used to calculate the state-dependent causal maps for redundant, unique, and
///     synergistic components, separating positive (causal) and negative (non-causal) contributions.
/// 6.  **Aggregation**: The state-dependent maps are stacked, and the aggregate SURD values are
///     calculated by summing the increments over all target states.
///
/// # None Value Handling
/// The presence of `Option<T>` in the input `CausalTensor` necessitates a specific strategy
/// for handling `None` values. The chosen approach aims for "no impact" or "ignore for this
/// specific calculation" for `None` values, aligning with a pairwise deletion strategy. The
/// intermediate state-dependent slices are computed as `CausalTensor<Option<T>>`; only at the
/// very last step, before being stored in the `SurdResult`, are remaining `None` values treated
/// as `0` so that the output is consistent with `SurdResult<T>`.
///
/// # Arguments
/// * `p_raw` - A `CausalTensor<Option<T>>` representing the joint probability distribution.
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
/// use deep_causality_algorithms::causal_discovery::surd::{surd_states_cdl, MaxOrder};
/// use deep_causality_tensor::CausalTensor;
///
/// // Create a joint probability distribution for a target and 2 source variables with missing data.
/// // Shape: [target_states, source1_states, source2_states] = [2, 2, 2]
/// let data = vec![
///     Some(0.1), Some(0.2), // P(T=0, S1=0, S2=0), P(T=0, S1=0, S2=1)
///     None,      Some(0.2), // P(T=0, S1=1, S2=0) is missing, P(T=0, S1=1, S2=1)
///     Some(0.3), None,      // P(T=1, S1=0, S2=0), P(T=1, S1=0, S2=1) is missing
///     Some(0.1), Some(0.1), // P(T=1, S1=1, S2=0), P(T=1, S1=1, S2=1)
/// ];
/// let p_raw = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
///
/// // Perform a full decomposition (k=N=2)
/// let full_result = surd_states_cdl(&p_raw, MaxOrder::Max).unwrap();
///
/// // Check some results (synergistic info for pair {1, 2})
/// assert!(full_result.synergistic_info().get(&vec![1, 2]).is_some());
///
/// // Perform a partial, pairwise decomposition (k=2)
/// // This is equivalent to MaxOrder::Max for N=2, but demonstrates the API.
/// let partial_result = surd_states_cdl(&p_raw, MaxOrder::Min).unwrap();
/// assert!(partial_result.synergistic_info().get(&vec![1, 2]).is_some());
/// ```
pub fn surd_states_cdl<T>(
    p_raw: &CausalTensor<Option<T>>,
    max_order: MaxOrder,
) -> Result<SurdResult<T>, CausalTensorError>
where
    T: RealField + FromPrimitive + Default + Send + Sync,
{
    if p_raw.is_empty() {
        return Err(CausalTensorError::EmptyTensor);
    }

    let zero = T::zero();
    let one = T::one();
    let eps = <T as FromPrimitive>::from_f64(1e-14).expect("1e-14 is representable in RealField");

    // --- 1. Pre-processing: Normalize the probability distribution ---
    // Sum only Some values for normalization
    let total_sum: T = p_raw
        .as_slice()
        .iter()
        .filter_map(|&x| x)
        .fold(zero, |acc, v| acc + v);
    if total_sum.abs() < eps {
        return Err(CausalTensorError::InvalidOperation);
    }

    let p_data: Vec<Option<T>> = p_raw
        .as_slice()
        .iter()
        .map(|&x| x.map(|val| val / total_sum))
        .collect();
    let p = CausalTensor::new(p_data, p_raw.shape().to_vec())?;

    let n_total_dims = p.num_dim();
    let n_vars = n_total_dims - 1;
    let n_target_states = p.shape()[0];
    let agent_indices: Vec<usize> = (1..n_total_dims).collect();
    let k = max_order.get_k_max_order(n_vars)?;

    // --- 2. Calculate Information Leak ---
    let h = surd_utils_cdl::entropy_nvars_cdl(&p, &[0])?;
    let hc = surd_utils_cdl::cond_entropy_cdl(&p, &[0], &agent_indices)?;

    let info_leak = if h > eps {
        (hc / h).clamp(zero, one)
    } else {
        zero
    };

    // --- 3. Compute Specific and Mutual Information ---
    let mut combs: Vec<Vec<usize>> = Vec::new();
    for i in 1..=k.min(n_vars) {
        let combinations_for_i = surd_utils::combinations(&agent_indices, i);
        combs.extend(combinations_for_i);
    }

    // p_s is CausalTensor<Option<T>> now
    let p_s = surd_utils_cdl::sum_axes_option_f64(&p, &agent_indices)?;
    let mut is_map: HashMap<Vec<usize>, CausalTensor<Option<T>>> = HashMap::new();
    for j_comb in &combs {
        let noj: Vec<usize> = agent_indices
            .iter()
            .filter(|&ax| !j_comb.contains(ax))
            .cloned()
            .collect();

        // p(T, j) - marginal distribution of target and the current combination of sources
        let p_as = if noj.is_empty() {
            p.clone()
        } else {
            surd_utils_cdl::sum_axes_option_f64(&p, &noj)?
        };

        // p(j) - marginal distribution of the current combination of sources
        let p_sources = surd_utils_cdl::sum_axes_option_f64(&p, &[0])?;
        let noj_mapped: Vec<usize> = noj.iter().map(|ax| ax - 1).collect();

        let p_j = if noj_mapped.is_empty() {
            p_sources.clone()
        } else {
            surd_utils_cdl::sum_axes_option_f64(&p_sources, &noj_mapped)?
        };

        let p_j_broadcasted = surd_utils_cdl::broadcast_to_cdl(&p_j, p_as.shape())?;
        let p_s_a = surd_utils_cdl::safe_div_cdl(&p_as, &p_j_broadcasted)?;

        let p_s_broadcasted_for_pas = surd_utils_cdl::broadcast_to_cdl(&p_s, p_as.shape())?;
        let p_a_s = surd_utils_cdl::safe_div_cdl(&p_as, &p_s_broadcasted_for_pas)?;

        // Broadcast p_s to the shape of p_s_a for the log difference calculation.
        let p_s_broadcasted_for_log = surd_utils_cdl::broadcast_to_cdl(&p_s, p_s_a.shape())?;

        let log_diff = surd_utils_cdl::sub_cdl(
            &surd_utils_cdl::surd_log2_cdl(&p_s_a)?,
            &surd_utils_cdl::surd_log2_cdl(&p_s_broadcasted_for_log)?,
        )?;
        let specific_info_map = surd_utils_cdl::mul_cdl(&p_a_s, &log_diff)?;

        let dims_to_sum_for_j_comb: Vec<usize> = (1..=j_comb.len()).collect();
        let sum_axes =
            surd_utils_cdl::sum_axes_option_f64(&specific_info_map, &dims_to_sum_for_j_comb)?;

        let ravel = sum_axes.ravel();
        is_map.insert(j_comb.clone(), ravel);
    }

    let mi: HashMap<Vec<usize>, T> = is_map
        .iter()
        .map(|(key, v)| {
            let multiplied = surd_utils_cdl::mul_cdl(v, &p_s)?;
            let sum = multiplied
                .as_slice()
                .iter()
                .filter_map(|&x| x)
                .fold(zero, |acc, x| acc + x);
            Ok((key.clone(), sum))
        })
        .collect::<Result<_, CausalTensorError>>()?;

    // --- 4. Main Decomposition Loop ---
    let results_per_target: Vec<_> = {
        #[cfg(feature = "parallel")]
        {
            (0..n_target_states)
                .into_par_iter()
                .map(|t| {
                    analyze_single_target_state_cdl(
                        t, n_vars, &combs, &is_map, &p_s, &p, // Pass CausalTensor<Option<T>>
                        k,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?
        }
        #[cfg(not(feature = "parallel"))]
        {
            (0..n_target_states)
                .map(|t| {
                    analyze_single_target_state_cdl(
                        t, n_vars, &combs, &is_map, &p_s, &p, // Pass CausalTensor<Option<T>>
                        k,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?
        }
    };

    // --- 5. Merge results from all target states ---
    let mut i_r = HashMap::new();
    let mut i_s = HashMap::new();
    let mut temp_causal_rd_states: HashMap<Vec<usize>, Vec<CausalTensor<T>>> = HashMap::new();
    let mut temp_causal_un_states: HashMap<Vec<usize>, Vec<CausalTensor<T>>> = HashMap::new();
    let mut temp_causal_sy_states: HashMap<Vec<usize>, Vec<CausalTensor<T>>> = HashMap::new();
    let mut temp_non_causal_rd_states: HashMap<Vec<usize>, Vec<CausalTensor<T>>> = HashMap::new();
    let mut temp_non_causal_un_states: HashMap<Vec<usize>, Vec<CausalTensor<T>>> = HashMap::new();
    let mut temp_non_causal_sy_states: HashMap<Vec<usize>, Vec<CausalTensor<T>>> = HashMap::new();

    for result in results_per_target {
        for (key, v) in result.i_r {
            *i_r.entry(key).or_insert(zero) += v;
        }
        for (key, v) in result.i_s {
            *i_s.entry(key).or_insert(zero) += v;
        }
        for (key, v) in result.causal_rd_states {
            temp_causal_rd_states.entry(key).or_default().push(v);
        }
        for (key, v) in result.causal_un_states {
            temp_causal_un_states.entry(key).or_default().push(v);
        }
        for (key, v) in result.causal_sy_states {
            temp_causal_sy_states.entry(key).or_default().push(v);
        }
        for (key, v) in result.non_causal_rd_states {
            temp_non_causal_rd_states.entry(key).or_default().push(v);
        }
        for (key, v) in result.non_causal_un_states {
            temp_non_causal_un_states.entry(key).or_default().push(v);
        }
        for (key, v) in result.non_causal_sy_states {
            temp_non_causal_sy_states.entry(key).or_default().push(v);
        }
    }

    // --- 6. Finalize State-Dependent Maps by Stacking ---
    let causal_redundant_states = temp_causal_rd_states
        .into_iter()
        .map(|(key, slices)| Ok((key, CausalTensor::stack(&slices, 0)?)))
        .collect::<Result<_, _>>()?;
    let causal_unique_states = temp_causal_un_states
        .into_iter()
        .map(|(key, slices)| Ok((key, CausalTensor::stack(&slices, 0)?)))
        .collect::<Result<_, _>>()?;
    let causal_synergistic_states = temp_causal_sy_states
        .into_iter()
        .map(|(key, slices)| Ok((key, CausalTensor::stack(&slices, 0)?)))
        .collect::<Result<_, _>>()?;
    let non_causal_redundant_states = temp_non_causal_rd_states
        .into_iter()
        .map(|(key, slices)| Ok((key, CausalTensor::stack(&slices, 0)?)))
        .collect::<Result<_, _>>()?;
    let non_causal_unique_states = temp_non_causal_un_states
        .into_iter()
        .map(|(key, slices)| Ok((key, CausalTensor::stack(&slices, 0)?)))
        .collect::<Result<_, _>>()?;
    let non_causal_synergistic_states = temp_non_causal_sy_states
        .into_iter()
        .map(|(key, slices)| Ok((key, CausalTensor::stack(&slices, 0)?)))
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
struct PerTargetStateResults<T> {
    i_r: HashMap<Vec<usize>, T>,
    i_s: HashMap<Vec<usize>, T>,
    causal_rd_states: HashMap<Vec<usize>, CausalTensor<T>>,
    causal_un_states: HashMap<Vec<usize>, CausalTensor<T>>,
    causal_sy_states: HashMap<Vec<usize>, CausalTensor<T>>,
    non_causal_rd_states: HashMap<Vec<usize>, CausalTensor<T>>,
    non_causal_un_states: HashMap<Vec<usize>, CausalTensor<T>>,
    non_causal_sy_states: HashMap<Vec<usize>, CausalTensor<T>>,
}

#[allow(clippy::too_many_arguments)]
fn analyze_single_target_state_cdl<T>(
    t: usize,
    n_vars: usize,
    combs: &[Vec<usize>],
    is_map: &HashMap<Vec<usize>, CausalTensor<Option<T>>>,
    p_s: &CausalTensor<Option<T>>,
    p: &CausalTensor<Option<T>>,
    _k: usize,
) -> Result<PerTargetStateResults<T>, CausalTensorError>
where
    T: RealField + FromPrimitive + Default + Send + Sync,
{
    let zero = T::zero();
    let eps = T::epsilon();

    let mut i_r = HashMap::new();
    let mut i_s = HashMap::new();
    let mut causal_rd_states = HashMap::new();
    let mut causal_un_states = HashMap::new();
    let mut causal_sy_states = HashMap::new();
    let mut non_causal_rd_states = HashMap::new();
    let mut non_causal_un_states = HashMap::new();
    let mut non_causal_sy_states = HashMap::new();

    // Extract i1_values, filtering out None for sorting, but keeping track of original indices
    let mut i1_values_with_indices: Vec<(T, usize)> = Vec::new();
    for (idx, c) in combs.iter().enumerate() {
        if let Some(val) = is_map[c].as_slice()[t] {
            i1_values_with_indices.push((val, idx));
        }
    }

    // Sort based on the Some values
    i1_values_with_indices
        .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let i1_sorted_indices: Vec<usize> =
        i1_values_with_indices.iter().map(|&(_, idx)| idx).collect();
    let i1_sorted_values: Vec<T> = i1_values_with_indices.iter().map(|&(val, _)| val).collect();

    let lab: Vec<Vec<usize>> = i1_sorted_indices
        .iter()
        .map(|&i| combs[i].clone())
        .collect();
    let mut i1_sorted: Vec<T> = i1_sorted_values.clone();
    let lens: Vec<usize> = lab.iter().map(|l| l.len()).collect();

    if let Some(&max_len) = lens.iter().max() {
        for l in 1..max_len {
            // Largest value among the entries whose label length is exactly `l`.
            let max_prev_level = i1_sorted
                .iter()
                .zip(&lens)
                .filter(|&(_, &len)| len == l)
                .map(|(&val, _)| val)
                .fold(None, |acc: Option<T>, v| match acc {
                    None => Some(v),
                    Some(a) => Some(if v > a { v } else { a }),
                });
            if let Some(max_prev_level) = max_prev_level {
                i1_sorted
                    .iter_mut()
                    .zip(&lens)
                    .filter(|&(_, &len)| len == l + 1)
                    .for_each(|(val, _)| {
                        if *val < max_prev_level {
                            *val = max_prev_level;
                        }
                    });
            }
        }
    }

    let new_sorted_indices = surd_utils::arg_sort(&i1_sorted);
    let final_i1: Vec<T> = new_sorted_indices.iter().map(|&i| i1_sorted[i]).collect();
    let final_lab: Vec<Vec<usize>> = new_sorted_indices.iter().map(|&i| lab[i].clone()).collect();
    let di_values = surd_utils::diff(&final_i1);

    // Find the index of the last single-variable term with a significant info contribution.
    let last_single_var_idx = final_lab
        .iter()
        .rposition(|lab| lab.len() == 1)
        .unwrap_or(usize::MAX);

    let mut red_vars: Vec<usize> = (1..=n_vars).collect();

    for (i, ll) in final_lab.iter().enumerate() {
        // info calculation needs to handle Option<T>
        let info = if let Some(p_s_val) = p_s.as_slice()[t] {
            di_values[i] * p_s_val
        } else {
            zero // If p_s is None, info contribution is 0
        };

        if info.abs() < eps {
            continue;
        }

        let prev_ll: &[usize] = if i == 0 { &[] } else { &final_lab[i - 1] };
        let (causal_slice, non_causal_slice) =
            calculate_state_slice_cdl(p, ll, prev_ll, t, n_vars)?;

        if ll.len() > 1
        // Synergistic states
        {
            causal_sy_states.insert(ll.clone(), causal_slice);
            non_causal_sy_states.insert(ll.clone(), non_causal_slice);
            *i_s.entry(ll.clone()).or_insert(zero) += info;
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
            *i_r.entry(red_vars.clone()).or_insert(zero) += info;
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
/// based on the sign of the log ratio. This corresponds to the state-dependent causal (`AC`)
/// and non-causal (`AN`) components in (martínezsánchez2025, Supplementary Material, e.g., Eq. S22 & S24).
///
/// # Arguments
/// * `p` - The full joint probability distribution tensor.
/// * `current_vars` - The set of source variables for the current information term (e.g., `{i}`).
/// * `prev_vars` - The set of source variables from the previous step in the sorted hierarchy (e.g., `{j}`).
/// * `target_state_index` - The index `t` of the target variable's state.
///
/// # Returns
/// A tuple `(causal_slice, non_causal_slice)` containing the two resulting tensors.
fn calculate_state_slice_cdl<T>(
    p: &CausalTensor<Option<T>>,
    current_vars: &[usize],
    prev_vars: &[usize],
    target_state_index: usize,
    n_vars: usize,
) -> Result<(CausalTensor<T>, CausalTensor<T>), CausalTensorError>
where
    T: RealField + FromPrimitive + Default + Send + Sync,
{
    let zero = T::zero();

    // 1. Get the slice corresponding to the current target state.
    let p_slice_option = p.slice(0, target_state_index)?;

    // 2. Remap axes since the target dimension (axis 0) is now gone.
    let current_vars_mapped: Vec<usize> = current_vars.iter().map(|&ax| ax - 1).collect();
    let prev_vars_mapped: Vec<usize> = prev_vars.iter().map(|&ax| ax - 1).collect();
    let all_vars_mapped: Vec<usize> = (0..p_slice_option.num_dim()).collect();

    let p_ti_option = surd_utils_cdl::sum_axes_option_f64(
        &p_slice_option,
        &surd_utils_cdl::set_difference(&all_vars_mapped, &current_vars_mapped),
    )?;

    // To calculate p(i), we marginalize out the target variable and then all other source variables.
    let source_axes: Vec<usize> = (0..n_vars).collect();
    let current_vars_mapped_for_marginal: Vec<usize> =
        current_vars.iter().map(|&ax| ax - 1).collect();
    let axes_to_sum_for_pi =
        surd_utils_cdl::set_difference(&source_axes, &current_vars_mapped_for_marginal);
    let p_i_option = surd_utils_cdl::sum_axes_option_f64(
        &surd_utils_cdl::sum_axes_option_f64(p, &[0])?,
        &axes_to_sum_for_pi,
    )?;

    if p_ti_option.shape() != p_i_option.shape() {
        dbg!("if p_ti_option.shape() != p_i_option.shape() : Tensor ShapeMismatch");

        return Err(CausalTensorError::ShapeMismatch);
    }
    let p_target_given_i_option = surd_utils_cdl::safe_div_cdl(&p_ti_option, &p_i_option)?;

    let p_target_given_j_option = if prev_vars.is_empty() {
        // If prev_vars is empty, p_target_given_j should represent p(T=t).
        // p_slice_option has shape [n_s1_states, n_s2_states, ...]
        // Summing all axes of p_slice_option gives p(T=t) as a scalar tensor.
        let p_t_scalar_option =
            surd_utils_cdl::sum_axes_option_f64(&p_slice_option, &all_vars_mapped)?;
        // This scalar tensor needs to be broadcast to match the shape of p_target_given_i_option.
        surd_utils_cdl::broadcast_to_cdl(&p_t_scalar_option, p_target_given_i_option.shape())?
    } else {
        let p_tj_option = surd_utils_cdl::sum_axes_option_f64(
            &p_slice_option,
            &surd_utils_cdl::set_difference(&all_vars_mapped, &prev_vars_mapped),
        )?;

        let prev_vars_mapped_for_marginal: Vec<usize> =
            prev_vars.iter().map(|&ax| ax - 1).collect();
        let axes_to_sum_for_pj =
            surd_utils_cdl::set_difference(&source_axes, &prev_vars_mapped_for_marginal);
        let p_j_option = surd_utils_cdl::sum_axes_option_f64(
            &surd_utils_cdl::sum_axes_option_f64(p, &[0])?,
            &axes_to_sum_for_pj,
        )?;

        if p_tj_option.shape() != p_j_option.shape() {
            dbg!("if p_tj_option.shape() != p_j_option.shape() : Tensor ShapeMismatch");
            return Err(CausalTensorError::ShapeMismatch);
        }
        surd_utils_cdl::safe_div_cdl(&p_tj_option, &p_j_option)?
    };

    let mut all_involved_vars = current_vars_mapped.to_vec();
    all_involved_vars.extend_from_slice(&prev_vars_mapped);
    all_involved_vars.sort();
    all_involved_vars.dedup();
    let axes_to_sum_out = surd_utils_cdl::set_difference(&all_vars_mapped, &all_involved_vars);
    let p_tij_option = surd_utils_cdl::sum_axes_option_f64(&p_slice_option, &axes_to_sum_out)?;

    // Broadcast p_target_given_i_option to the shape of p_tij_option
    let p_target_given_i_broadcasted_option =
        surd_utils_cdl::broadcast_to_cdl(&p_target_given_i_option, p_tij_option.shape())?;

    // Broadcast p_target_given_j_option to the shape of p_tij_option
    let p_target_given_j_broadcasted_option =
        surd_utils_cdl::broadcast_to_cdl(&p_target_given_j_option, p_tij_option.shape())?;

    let log_ratio_option = surd_utils_cdl::sub_cdl(
        &surd_utils_cdl::surd_log2_cdl(&p_target_given_i_broadcasted_option)?,
        &surd_utils_cdl::surd_log2_cdl(&p_target_given_j_broadcasted_option)?,
    )?;

    // Separate into causal (>0) and non-causal (<0) components
    let causal_log_ratio_option_data: Vec<Option<T>> = log_ratio_option
        .as_slice()
        .iter()
        .map(|&v_opt| v_opt.map(|v| if v > zero { v } else { zero }))
        .collect();

    let non_causal_log_ratio_option_data: Vec<Option<T>> = log_ratio_option
        .as_slice()
        .iter()
        .map(|&v_opt| v_opt.map(|v| if v < zero { v } else { zero }))
        .collect();

    let causal_log_ratio_tensor = CausalTensor::new(
        causal_log_ratio_option_data,
        log_ratio_option.shape().to_vec(),
    )?;
    let non_causal_log_ratio_tensor = CausalTensor::new(
        non_causal_log_ratio_option_data,
        log_ratio_option.shape().to_vec(),
    )?;

    let causal_slice_option = surd_utils_cdl::mul_cdl(&p_tij_option, &causal_log_ratio_tensor)?;
    let non_causal_slice_option =
        surd_utils_cdl::mul_cdl(&p_tij_option, &non_causal_log_ratio_tensor)?;

    // Convert final Option<T> slices to T, treating None as 0 for the output
    let causal_slice_data: Vec<T> = causal_slice_option
        .as_slice()
        .iter()
        .map(|&x| x.unwrap_or(zero))
        .collect();

    let non_causal_slice_data: Vec<T> = non_causal_slice_option
        .as_slice()
        .iter()
        .map(|&x| x.unwrap_or(zero))
        .collect();

    let causal_slice = CausalTensor::new(causal_slice_data, causal_slice_option.shape().to_vec())?;
    let non_causal_slice = CausalTensor::new(
        non_causal_slice_data,
        non_causal_slice_option.shape().to_vec(),
    )?;

    Ok((causal_slice, non_causal_slice))
}
