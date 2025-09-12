/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_data_structures::{CausalTensor, CausalTensorError};
use std::cmp::Ordering;

/// Computes the difference between adjacent elements of a slice.
///
/// Returns a new `Vec` containing the differences. If the input slice has `N`
/// elements, the output vector will have `N-1` elements. Returns an empty
/// vector if the input has fewer than 2 elements.
///
pub(super) fn diff(slice: &[f64]) -> Vec<f64> {
    let mut result = Vec::new();
    if slice.is_empty() {
        return result;
    }

    // Python's np.diff(arr, prepend=0) effectively calculates diff on [0] + arr
    // So the first element is slice[0] - 0
    result.push(slice[0] - 0f64); // This is effectively slice[0]

    for i in 1..slice.len() {
        result.push(slice[i] - slice[i - 1]);
    }
    result
}

pub(super) fn arg_sort(slice: &[f64]) -> Vec<usize> {
    // 1. Create a vector of the original indices: [0, 1, 2, ..., n-1]
    let mut indices: Vec<usize> = (0..slice.len()).collect();

    // 2. Sort the `indices` vector. The magic is in the comparison closure.
    //    Rust's `sort_by` is a stable sort.
    indices.sort_by(|&a_index, &b_index| {
        // 3. For any two indices, compare the values in the *original data slice*
        //    at those positions.
        let a_value = slice[a_index];
        let b_value = slice[b_index];

        // 4. Use `partial_cmp` because f64 does not have a total ordering (due to NaN).
        //    `.unwrap_or(Ordering::Equal)` is the key to robustly handling NaN.
        //    It treats any non-comparable values (NaNs) as equal, preventing a panic.
        a_value.partial_cmp(&b_value).unwrap_or(Ordering::Equal)
    });

    indices
}

pub(super) fn set_difference<T: PartialEq + Clone>(a: &[T], b: &[T]) -> Vec<T> {
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
pub(super) fn combinations<T: Copy>(pool: &[T], r: usize) -> Vec<Vec<T>> {
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
pub fn entropy_nvars(p: &CausalTensor<f64>, axes: &[usize]) -> Result<f64, CausalTensorError> {
    // Determine the axes to sum out to get the marginal distribution.
    let all_axes: Vec<_> = (0..p.num_dim()).collect();
    let axes_to_sum_out: Vec<_> = all_axes
        .into_iter()
        .filter(|ax| !axes.contains(ax))
        .collect();

    // If we are not summing over any axes, it means we want the entropy of the
    // full joint distribution `p`. We must use `p` directly instead of calling
    // `sum_axes(&[])`, which would incorrectly perform a full reduction.
    let marginal = if axes_to_sum_out.is_empty() {
        // PERF: We clone the tensor here to maintain a consistent type for `marginal`.
        // A more advanced implementation might use references (`Cow`) to avoid this.
        p.clone()
    } else {
        p.sum_axes(&axes_to_sum_out)?
    };

    let entropy = marginal.as_slice().iter().fold(0.0, |acc, &prob| {
        if prob > 0.0 {
            acc - prob * prob.log2()
        } else {
            acc
        }
    });

    Ok(entropy)
}

/// Calculates the conditional Shannon entropy H(X | Y).
///
/// Uses the formula: H(X | Y) = H(X, Y) - H(Y).
pub fn cond_entropy(
    p: &CausalTensor<f64>,
    target_axes: &[usize],
    cond_axes: &[usize],
) -> Result<f64, CausalTensorError>
where
{
    let mut joint_axes = target_axes.to_vec();
    joint_axes.extend_from_slice(cond_axes);
    joint_axes.sort();
    joint_axes.dedup();

    let h_xy = entropy_nvars(p, &joint_axes)?;
    let h_y = entropy_nvars(p, cond_axes)?;

    Ok(h_xy - h_y)
}
