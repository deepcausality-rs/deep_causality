/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::RealField;
use deep_causality_tensor::{CausalTensor, CausalTensorError};

/// Helper function to unravel a flat index into multi-dimensional coordinates.
///
/// This function converts a single linear index into its corresponding multi-dimensional
/// coordinates based on the provided shape. It is used internally for tensor operations.
///
/// # Arguments
/// * `flat_index` - The linear index to unravel.
/// * `shape` - A slice representing the dimensions of the tensor.
///
/// # Returns
/// A `Result` containing a `Vec<usize>` representing the multi-dimensional coordinates,
/// or a `CausalTensorError` if the operation fails.
fn unravel_index_option(
    flat_index: usize,
    shape: &[usize],
) -> Result<Vec<usize>, CausalTensorError> {
    let mut coords = vec![0; shape.len()];
    let mut remainder = flat_index;
    for i in 0..shape.len() {
        let stride: usize = shape[i + 1..].iter().product();
        coords[i] = remainder / stride;
        remainder %= stride;
    }
    Ok(coords)
}

/// Helper function to ravel multi-dimensional coordinates into a flat index.
///
/// This function converts multi-dimensional coordinates into a single linear index based on
/// the provided shape. It is used internally for tensor operations.
///
/// # Arguments
/// * `coords` - A slice representing the multi-dimensional coordinates.
/// * `shape` - A slice representing the dimensions of the tensor.
///
/// # Returns
/// A `Result` containing a `usize` representing the linear index,
/// or a `CausalTensorError` if the coordinates are out of bounds or dimensions mismatch.
pub(super) fn ravel_index_from_coords_option(
    coords: &[usize],
    shape: &[usize],
) -> Result<usize, CausalTensorError> {
    if coords.len() != shape.len() {
        return Err(CausalTensorError::DimensionMismatch);
    }

    let mut flat_index = 0;
    for i in 0..coords.len() {
        if coords[i] >= shape[i] {
            return Err(CausalTensorError::AxisOutOfBounds);
        }
        flat_index = flat_index * shape[i] + coords[i];
    }
    Ok(flat_index)
}

/// Sums elements along specified axes for a `CausalTensor<Option<T>>`, ignoring `None`s.
///
/// `None` values in the input tensor are **ignored** during summation: they do not contribute
/// `0` to the sum, nor do they force the sum to `None` unless all contributing values are `None`.
/// If all values contributing to a particular output element are `None`, that output element is `None`.
///
/// # Arguments
/// * `tensor` - The input `CausalTensor<Option<T>>`.
/// * `axes_to_sum_out` - A slice of `usize` representing the axes to sum over.
///
/// # Returns
/// A `Result` containing a new `CausalTensor<Option<T>>` with the specified axes summed out,
/// or a `CausalTensorError` if the operation fails.
pub fn sum_axes_option_f64<T: RealField + Default>(
    tensor: &CausalTensor<Option<T>>,
    axes_to_sum_out: &[usize],
) -> Result<CausalTensor<Option<T>>, CausalTensorError> {
    let input_shape = tensor.shape();
    let input_data = tensor.as_slice();
    let num_dims = input_shape.len();

    // Determine the output shape
    let mut output_shape = Vec::with_capacity(num_dims - axes_to_sum_out.len());
    for (i, &dim_size) in input_shape.iter().enumerate() {
        if !axes_to_sum_out.contains(&i) {
            output_shape.push(dim_size);
        }
    }

    let output_size: usize = output_shape.iter().product();
    // Initialize with None, as we only sum Some values
    let mut output_data: Vec<Option<T>> = vec![None; output_size];

    // Iterate through the input tensor
    for (flat_idx, &val_option) in input_data.iter().enumerate() {
        if let Some(val) = val_option {
            let input_coords = unravel_index_option(flat_idx, input_shape)?;

            // Map input coordinates to output coordinates
            let mut output_coords = Vec::with_capacity(output_shape.len());
            for (i, &coord_val) in input_coords.iter().enumerate() {
                if !axes_to_sum_out.contains(&i) {
                    output_coords.push(coord_val);
                }
            }

            let output_flat_idx = ravel_index_from_coords_option(&output_coords, &output_shape)?;
            // Sum only Some values. If output_data[output_flat_idx] is None, initialize with val.
            // Otherwise, add val to the existing Some value.
            output_data[output_flat_idx] = match output_data[output_flat_idx] {
                Some(existing_val) => Some(existing_val + val),
                None => Some(val),
            };
        }
    }

    CausalTensor::new(output_data, output_shape)
}

/// Calculates the Shannon entropy H(X) over a set of variables for `CausalTensor<Option<T>>`.
///
/// `None` values are **ignored** during marginalization and during the entropy sum. If the sum of
/// all `Some` marginal probabilities is effectively zero, the entropy is returned as `0`.
///
/// # Arguments
/// * `p` - The input `CausalTensor<Option<T>>` representing the joint probability distribution.
/// * `axes` - A slice of `usize` representing the axes for which to calculate the entropy.
///
/// # Returns
/// A `Result` containing a `T` representing the calculated entropy, or a `CausalTensorError`
/// if the operation fails.
pub(crate) fn entropy_nvars_cdl<T: RealField + Default>(
    p: &CausalTensor<Option<T>>,
    axes: &[usize],
) -> Result<T, CausalTensorError> {
    let all_axes: Vec<_> = (0..p.num_dim()).collect();
    let axes_to_sum_out: Vec<_> = all_axes
        .into_iter()
        .filter(|ax| !axes.contains(ax))
        .collect();

    let marginal = sum_axes_option_f64(p, &axes_to_sum_out)?;

    let zero = T::zero();
    let eps = T::epsilon();

    // Normalize the marginal distribution based on the sum of its Some values.
    let sum_of_marginals: T = marginal
        .as_slice()
        .iter()
        .filter_map(|&x| x)
        .fold(zero, |acc, v| acc + v);

    if sum_of_marginals.abs() < eps {
        return Ok(zero); // If all probabilities are zero or None, entropy is 0.
    }

    let entropy = marginal.as_slice().iter().fold(zero, |acc, &prob_opt| {
        if let Some(prob) = prob_opt {
            let normalized_prob = prob / sum_of_marginals;
            if normalized_prob > eps {
                acc - normalized_prob * normalized_prob.log2()
            } else {
                acc
            }
        } else {
            acc // Ignore None values for entropy calculation
        }
    });
    Ok(entropy)
}

/// Calculates the conditional Shannon entropy H(X | Y) for `CausalTensor<Option<T>>`.
///
/// Uses the formula: H(X | Y) = H(X, Y) - H(Y). `None` values are handled implicitly by
/// `entropy_nvars_cdl`, which ignores them during marginalization and entropy calculation.
///
/// # Arguments
/// * `p` - The input `CausalTensor<Option<T>>` representing the joint probability distribution.
/// * `target_axes` - A slice of `usize` representing the axes of the target variable (X).
/// * `cond_axes` - A slice of `usize` representing the axes of the conditioning variable (Y).
///
/// # Returns
/// A `Result` containing a `T` representing the calculated conditional entropy,
/// or a `CausalTensorError` if the operation fails.
pub(crate) fn cond_entropy_cdl<T: RealField + Default>(
    p: &CausalTensor<Option<T>>,
    target_axes: &[usize],
    cond_axes: &[usize],
) -> Result<T, CausalTensorError> {
    let mut joint_axes = target_axes.to_vec();
    joint_axes.extend_from_slice(cond_axes);
    joint_axes.sort();
    joint_axes.dedup();

    let h_xy = entropy_nvars_cdl(p, &joint_axes)?;
    let h_y = entropy_nvars_cdl(p, cond_axes)?;

    Ok(h_xy - h_y)
}

/// Performs element-wise safe division for two `CausalTensor<Option<T>>` tensors.
///
/// If either operand is `None`, the result is `None`. If both are `Some` but the denominator
/// is effectively zero, the result is `None` (division by zero). Otherwise `Some(num / den)`.
///
/// # Arguments
/// * `numerator` - The `CausalTensor<Option<T>>` representing the numerator.
/// * `denominator` - The `CausalTensor<Option<T>>` representing the denominator.
///
/// # Returns
/// A `Result` containing a new `CausalTensor<Option<T>>` with the element-wise division,
/// or a `CausalTensorError` if the tensor shapes do not match.
pub(crate) fn safe_div_cdl<T: RealField + Default>(
    numerator: &CausalTensor<Option<T>>,
    denominator: &CausalTensor<Option<T>>,
) -> Result<CausalTensor<Option<T>>, CausalTensorError> {
    if numerator.shape() != denominator.shape() {
        dbg!("safe_div_cdl: Input tensor ShapeMismatch");
        return Err(CausalTensorError::ShapeMismatch);
    }

    let eps = T::epsilon();
    let result_data: Vec<Option<T>> = numerator
        .as_slice()
        .iter()
        .zip(denominator.as_slice().iter())
        .map(|(&num_opt, &den_opt)| match (num_opt, den_opt) {
            (Some(num), Some(den)) => {
                if den.abs() < eps {
                    None // Division by zero
                } else {
                    Some(num / den)
                }
            }
            _ => None, // If any operand is None, result is None
        })
        .collect();

    CausalTensor::new(result_data, numerator.shape().to_vec())
}

/// Performs element-wise base-2 logarithm for a `CausalTensor<Option<T>>`.
///
/// If an element is `None`, the output is `None`. If an element is `Some(val)` with `val`
/// non-positive, the result is `None` (logarithm undefined). Otherwise `Some(val.log2())`.
///
/// # Arguments
/// * `tensor` - The input `CausalTensor<Option<T>>`.
///
/// # Returns
/// A `Result` containing a new `CausalTensor<Option<T>>` with the element-wise `log2`,
/// or a `CausalTensorError` if the operation fails.
pub(crate) fn surd_log2_cdl<T: RealField + Default>(
    tensor: &CausalTensor<Option<T>>,
) -> Result<CausalTensor<Option<T>>, CausalTensorError> {
    let eps = T::epsilon();
    let result_data: Vec<Option<T>> = tensor
        .as_slice()
        .iter()
        .map(|&val_opt| match val_opt {
            Some(val) => {
                if val > eps {
                    // Only take log of positive numbers
                    Some(val.log2())
                } else {
                    None // Log of non-positive is undefined, propagate None
                }
            }
            None => None, // Propagate None
        })
        .collect();

    CausalTensor::new(result_data, tensor.shape().to_vec())
}

/// Performs element-wise multiplication for two `CausalTensor<Option<T>>` tensors.
///
/// If either operand is `None`, the result is `None`. Otherwise `Some(v_a * v_b)`.
///
/// # Arguments
/// * `a` - The first `CausalTensor<Option<T>>` operand.
/// * `b` - The second `CausalTensor<Option<T>>` operand.
///
/// # Returns
/// A `Result` containing a new `CausalTensor<Option<T>>` with the element-wise multiplication,
/// or a `CausalTensorError` if the tensor shapes do not match.
pub(crate) fn mul_cdl<T: RealField + Default>(
    a: &CausalTensor<Option<T>>,
    b: &CausalTensor<Option<T>>,
) -> Result<CausalTensor<Option<T>>, CausalTensorError> {
    if a.shape() != b.shape() {
        dbg!("mul_cdl: Input tensor ShapeMismatch");
        return Err(CausalTensorError::ShapeMismatch);
    }

    let result_data: Vec<Option<T>> = a
        .as_slice()
        .iter()
        .zip(b.as_slice().iter())
        .map(|(&val_a, &val_b)| match (val_a, val_b) {
            (Some(v_a), Some(v_b)) => Some(v_a * v_b),
            _ => None, // If any is None, the result is None
        })
        .collect();

    CausalTensor::new(result_data, a.shape().to_vec())
}

/// Performs element-wise subtraction for two `CausalTensor<Option<T>>` tensors.
///
/// If either operand is `None`, the result is `None`. Otherwise `Some(v_a - v_b)`.
///
/// # Arguments
/// * `a` - The first `CausalTensor<Option<T>>` operand.
/// * `b` - The second `CausalTensor<Option<T>>` operand.
///
/// # Returns
/// A `Result` containing a new `CausalTensor<Option<T>>` with the element-wise subtraction,
/// or a `CausalTensorError` if the tensor shapes do not match.
pub(crate) fn sub_cdl<T: RealField + Default>(
    a: &CausalTensor<Option<T>>,
    b: &CausalTensor<Option<T>>,
) -> Result<CausalTensor<Option<T>>, CausalTensorError> {
    if a.shape() != b.shape() {
        dbg!("sub_cdl: Input tensor ShapeMismatch");
        return Err(CausalTensorError::ShapeMismatch);
    }

    let result_data: Vec<Option<T>> = a
        .as_slice()
        .iter()
        .zip(b.as_slice().iter())
        .map(|(&val_a, &val_b)| match (val_a, val_b) {
            (Some(v_a), Some(v_b)) => Some(v_a - v_b),
            _ => None, // If any is None, the result is None
        })
        .collect();

    CausalTensor::new(result_data, a.shape().to_vec())
}

/// Computes the set difference (A - B) between two slices of elements.
///
/// Returns a new vector containing all elements from slice `a` that are not present in slice `b`.
///
/// # Arguments
/// * `a` - The first slice (set A).
/// * `b` - The second slice (set B).
///
/// # Returns
/// A `Vec<T>` containing elements present in `a` but not in `b`.
pub(crate) fn set_difference<T: PartialEq + Clone>(a: &[T], b: &[T]) -> Vec<T> {
    a.iter()
        .filter(|&item_a| !b.contains(item_a))
        .cloned()
        .collect()
}

pub(crate) fn broadcast_to_cdl<T: RealField + Default>(
    tensor: &CausalTensor<Option<T>>,
    target_shape: &[usize],
) -> Result<CausalTensor<Option<T>>, CausalTensorError> {
    let tensor_shape = tensor.shape();
    let tensor_data = tensor.as_slice();

    let target_ndim = target_shape.len();
    let tensor_ndim = tensor_shape.len();

    if tensor_ndim > target_ndim {
        dbg!("broadcast_to_cdl: Input tensor ShapeMismatch");
        return Err(CausalTensorError::ShapeMismatch);
    }

    let mut result_data = Vec::with_capacity(target_shape.iter().product());

    for flat_idx_target in 0..target_shape.iter().product() {
        let coords_target = unravel_index_option(flat_idx_target, target_shape)?;

        let mut flat_idx_tensor = 0;
        let mut multiplier = 1;
        for i in (0..tensor_ndim).rev() {
            let target_coord_idx = target_ndim - (tensor_ndim - i);
            let tensor_dim = tensor_shape[i];
            let target_coord = coords_target[target_coord_idx];

            if tensor_dim != 1 {
                flat_idx_tensor += (target_coord % tensor_dim) * multiplier;
            }
            multiplier *= tensor_dim;
        }
        result_data.push(tensor_data[flat_idx_tensor]);
    }

    CausalTensor::new(result_data, target_shape.to_vec())
}
