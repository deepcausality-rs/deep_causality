/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::RealField;
use deep_causality_tensor::{CausalTensor, CausalTensorError, Tensor};
use std::cmp::Ordering;

pub(crate) mod surd_utils_cdl;
#[cfg(test)]
mod surd_utils_tests;

/// Computes the difference between adjacent elements of a slice.
///
/// Returns a new `Vec` containing the differences. If the input slice has `N`
/// elements, the output vector will have `N-1` elements. Returns an empty
/// vector if the input has fewer than 2 elements.
///
pub(crate) fn diff<T: RealField>(slice: &[T]) -> Vec<T> {
    let mut result = Vec::new();
    if slice.is_empty() {
        return result;
    }

    // Python's np.diff(arr, prepend=0) effectively calculates diff on [0] + arr
    // So the first element is slice[0] - 0
    result.push(slice[0] - T::zero()); // This is effectively slice[0]

    for i in 1..slice.len() {
        result.push(slice[i] - slice[i - 1]);
    }
    result
}

/// Stable argsort that ranks values on a grid of resolution `tol`.
///
/// Each value is quantized to the nearest multiple of `tol` (`(v / tol).round()`)
/// before comparison, so two values that fall in the same grid cell compare equal
/// and — because the sort is stable — keep their original relative order. The SURD
/// decomposition (martínezsánchez2025, Fig. S2) ranks the specific informations and
/// takes consecutive increments; the paper is tie-robust because equal informations
/// yield a zero increment regardless of their order. Plain value sorting breaks that
/// invariant under any sub-resolution perturbation: differences far below numerical
/// resolution (e.g. Miri's deliberate few-ULP fuzzing of `log2`) reorder
/// would-be-tied terms, which then pair with a different neighbour in the hierarchy
/// and produce a different (and target-state-inconsistent) decomposition. Quantizing
/// the rank key makes the ranking invariant to perturbations below `tol`, so the
/// result is a deterministic function of the data at the chosen resolution.
///
/// The quantized keys are finite, non-NaN floats, so the comparator is a genuine
/// total order (no `sort_by` total-order violation).
pub(crate) fn arg_sort_stable<T: RealField>(slice: &[T], tol: T) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..slice.len()).collect();
    indices.sort_by(|&a_index, &b_index| {
        let a_key = (slice[a_index] / tol).round();
        let b_key = (slice[b_index] / tol).round();
        a_key.partial_cmp(&b_key).unwrap_or(Ordering::Equal)
    });
    indices
}

pub(crate) fn set_difference<T: PartialEq + Clone>(a: &[T], b: &[T]) -> Vec<T> {
    a.iter()
        .filter(|&item| !b.contains(item))
        .cloned()
        .collect()
}

/// Generates all unique combinations of `r` elements from a given `pool`.
///
/// This is a Rust implementation of the logic found in Python's `itertools.combinations`.
///
/// # Panics
/// Panics if `r` is greater than the number of items in `pool`.
/// ```
pub(crate) fn combinations<T: Copy>(pool: &[T], r: usize) -> Vec<Vec<T>> {
    if r > pool.len() {
        panic!("Cannot choose r elements from a pool smaller than r.");
    }
    if r == 0 {
        return vec![vec![]];
    }

    let mut result = Vec::new();
    let mut indices: Vec<usize> = (0..r).collect();

    loop {
        result.push(indices.iter().map(|&i| pool[i]).collect());

        let mut i = r - 1;
        loop {
            indices[i] += 1;
            if indices[i] < pool.len() - (r - 1 - i) {
                for j in (i + 1)..r {
                    indices[j] = indices[j - 1] + 1;
                }
                break;
            }
            if i == 0 {
                return result;
            }
            i -= 1;
        }
    }
}

/// Calculates the Shannon entropy H(X) over a set of variables.
///
/// The input tensor `p` is assumed to be a joint probability distribution.
/// This function computes the entropy of the marginal distribution over the specified `axes`.
pub fn entropy_nvars<T: RealField + Default>(
    p: &CausalTensor<T>,
    axes: &[usize],
) -> Result<T, CausalTensorError> {
    // Determine the axes to sum out to get the marginal distribution.
    let all_axes: Vec<_> = (0..p.num_dim()).collect();
    let axes_to_sum_out: Vec<_> = all_axes
        .into_iter()
        .filter(|ax| !axes.contains(ax))
        .collect();

    let zero = T::zero();

    if axes_to_sum_out.is_empty() {
        // Optimization: If not summing over any axes, calculate entropy directly on the slice
        // to avoid cloning the entire tensor.
        let entropy = p.as_slice().iter().fold(zero, |acc, &prob| {
            if prob > zero {
                acc - prob * prob.log2()
            } else {
                acc
            }
        });
        Ok(entropy)
    } else {
        // Calculate the marginal distribution by summing out the specified axes.
        let marginal = p.sum_axes(&axes_to_sum_out)?;
        let entropy = marginal.as_slice().iter().fold(zero, |acc, &prob| {
            if prob > zero {
                acc - prob * prob.log2()
            } else {
                acc
            }
        });
        Ok(entropy)
    }
}

/// Calculates the conditional Shannon entropy H(X | Y).
///
/// Uses the formula: H(X | Y) = H(X, Y) - H(Y).
pub fn cond_entropy<T: RealField + Default>(
    p: &CausalTensor<T>,
    target_axes: &[usize],
    cond_axes: &[usize],
) -> Result<T, CausalTensorError> {
    let mut joint_axes = target_axes.to_vec();
    joint_axes.extend_from_slice(cond_axes);
    joint_axes.sort();
    joint_axes.dedup();

    let h_xy = entropy_nvars(p, &joint_axes)?;
    let h_y = entropy_nvars(p, cond_axes)?;

    Ok(h_xy - h_y)
}
