/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::{CausalTensor, CausalTensorError};

/// Helper function to unravel a flat index into multi-dimensional coordinates for `CausalTensor<Option<f64>>`.
///
/// This function converts a single linear index into its corresponding multi-dimensional
/// coordinates based on the provided shape. It is used internally for tensor operations.
///
/// # None Handling
/// This function operates on indices and shape, not directly on the `Option<f64>` values
/// within the tensor. Therefore, it does not perform any `None` handling.
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
    let mut coords = Vec::with_capacity(shape.len());
    let mut temp_flat_index = flat_index;
    for &dim_size in shape.iter().rev() {
        coords.push(temp_flat_index % dim_size);
        temp_flat_index /= dim_size;
    }
    coords.reverse();
    Ok(coords)
}

/// Helper function to ravel multi-dimensional coordinates into a flat index for `CausalTensor<Option<f64>>`.
///
/// This function converts multi-dimensional coordinates into a single linear index based on
/// the provided shape. It is used internally for tensor operations.
///
/// # None Handling
/// This function operates on coordinates and shape, not directly on the `Option<f64>` values
/// within the tensor. Therefore, it does not perform any `None` handling.
///
/// # Arguments
/// * `coords` - A slice representing the multi-dimensional coordinates.
/// * `shape` - A slice representing the dimensions of the tensor.
///
/// # Returns
/// A `Result` containing a `usize` representing the linear index,
/// or a `CausalTensorError` if the coordinates are out of bounds or dimensions mismatch.
fn ravel_index_from_coords_option(
    coords: &[usize],
    shape: &[usize],
) -> Result<usize, CausalTensorError> {
    if coords.len() != shape.len() {
        return Err(CausalTensorError::DimensionMismatch);
    }

    let mut flat_index = 0;
    let mut multiplier = 1;
    for i in (0..shape.len()).rev() {
        if coords[i] >= shape[i] {
            return Err(CausalTensorError::AxisOutOfBounds);
        }
        flat_index += coords[i] * multiplier;
        multiplier *= shape[i];
    }
    Ok(flat_index)
}

/// Sums elements along specified axes for a `CausalTensor<Option<f64>>`.
///
/// This function computes the marginal distribution by summing elements along the `axes_to_sum_out`.
/// It is designed to handle `Option<f64>` values by ignoring `None`s during summation.
///
/// # None Handling
/// -   `None` values in the input tensor are **ignored** during summation. They do not contribute
///     `0.0` to the sum, nor do they cause the sum to become `None` unless all contributing values
///     are `None`.
/// -   If all values contributing to a particular output element are `None`, the corresponding
///     output element will be `None`.
/// -   If at least one `Some(f64)` value contributes to an output element, that `Some` value (and
///     any other `Some` values) will be summed, and the `None` values will be skipped.
///
/// # Rationale for None Handling
/// This strategy implements a form of pairwise deletion for summation. It ensures that missing
/// data (`None`) does not artificially influence marginal probabilities by being treated as `0.0`.
/// This is crucial for datasets where `None` means "not measured" or "unobserved" rather than
/// "zero probability," allowing calculations to proceed based solely on observed data.
///
/// # Arguments
/// * `tensor` - The input `CausalTensor<Option<f64>>`.
/// * `axes_to_sum_out` - A slice of `usize` representing the axes to sum over.
///
/// # Returns
/// A `Result` containing a new `CausalTensor<Option<f64>>` with the specified axes summed out,
/// or a `CausalTensorError` if the operation fails.
pub fn sum_axes_option_f64(
    tensor: &CausalTensor<Option<f64>>,
    axes_to_sum_out: &[usize],
) -> Result<CausalTensor<Option<f64>>, CausalTensorError> {
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
    let mut output_data = vec![None; output_size];

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

/// Calculates the Shannon entropy H(X) over a set of variables for `CausalTensor<Option<f64>>`.
///
/// The input tensor `p` is assumed to be a joint probability distribution. This function
/// computes the entropy of the marginal distribution over the specified `axes`.
///
/// # None Handling
/// -   `None` values are **ignored** during the calculation of the marginal distribution
///     (via `sum_axes_option_f64`).
/// -   When normalizing the marginal distribution, only the sum of `Some` values is used.
/// -   During the entropy calculation (`-p * log2(p)`), `None` values in the marginal distribution
///     are **skipped**. They do not contribute to the entropy sum.
/// -   If the sum of all `Some` marginal probabilities is effectively zero (less than `f64::EPSILON`),
///     the entropy is returned as `0.0`.
///
/// # Rationale for None Handling
/// This approach ensures that missing data (`None`) does not artificially influence the entropy
/// calculation. By ignoring `None`s, the entropy is computed solely based on the observed
/// probabilities, reflecting the information content present in the available data. This is
/// consistent with treating `None` as "not measured" rather than "zero probability."
///
/// # Arguments
/// * `p` - The input `CausalTensor<Option<f64>>` representing the joint probability distribution.
/// * `axes` - A slice of `usize` representing the axes for which to calculate the entropy.
///
/// # Returns
/// A `Result` containing an `f64` representing the calculated entropy, or a `CausalTensorError`
/// if the operation fails.
pub(crate) fn entropy_nvars_cdl(
    p: &CausalTensor<Option<f64>>,
    axes: &[usize],
) -> Result<f64, CausalTensorError> {
    let all_axes: Vec<_> = (0..p.num_dim()).collect();
    let axes_to_sum_out: Vec<_> = all_axes
        .into_iter()
        .filter(|ax| !axes.contains(ax))
        .collect();

    let marginal = sum_axes_option_f64(p, &axes_to_sum_out)?;

    // Normalize the marginal distribution based on the sum of its Some values.
    let sum_of_marginals: f64 = marginal.as_slice().iter().filter_map(|&x| x).sum();

    if sum_of_marginals.abs() < f64::EPSILON {
        // Use EPSILON for float comparison
        return Ok(0.0); // If all probabilities are zero or None, entropy is 0.
    }

    let entropy = marginal.as_slice().iter().fold(0.0, |acc, &prob_opt| {
        if let Some(prob) = prob_opt {
            let normalized_prob = prob / sum_of_marginals;
            if normalized_prob > f64::EPSILON {
                // Use EPSILON for float comparison
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

/// Calculates the conditional Shannon entropy H(X | Y) for `CausalTensor<Option<f64>>`.
///
/// This function computes conditional entropy using the formula: H(X | Y) = H(X, Y) - H(Y).
/// It relies on `entropy_nvars_cdl` for its calculations.
///
/// # None Handling
/// `None` values are handled implicitly by the underlying `entropy_nvars_cdl` function,
/// which ignores `None`s during marginalization and entropy calculation.
///
/// # Rationale for None Handling
/// Consistent with `entropy_nvars_cdl`, this ensures that missing data does not artificially
/// influence the conditional entropy, allowing the calculation to be based solely on observed data.
///
/// # Arguments
/// * `p` - The input `CausalTensor<Option<f64>>` representing the joint probability distribution.
/// * `target_axes` - A slice of `usize` representing the axes of the target variable (X).
/// * `cond_axes` - A slice of `usize` representing the axes of the conditioning variable (Y).
///
/// # Returns
/// A `Result` containing an `f64` representing the calculated conditional entropy,
/// or a `CausalTensorError` if the operation fails.
pub(crate) fn cond_entropy_cdl(
    p: &CausalTensor<Option<f64>>,
    target_axes: &[usize],
    cond_axes: &[usize],
) -> Result<f64, CausalTensorError>
where
{
    let mut joint_axes = target_axes.to_vec();
    joint_axes.extend_from_slice(cond_axes);
    joint_axes.sort();
    joint_axes.dedup();

    let h_xy = entropy_nvars_cdl(p, &joint_axes)?;
    let h_y = entropy_nvars_cdl(p, cond_axes)?;

    Ok(h_xy - h_y)
}

/// Performs element-wise safe division for two `CausalTensor<Option<f64>>` tensors.
///
/// This function divides each element of the `numerator` tensor by the corresponding element
/// of the `denominator` tensor. It includes safety checks for division by zero and handles `None` values.
///
/// # None Handling
/// -   If either the `numerator` or `denominator` element is `None`, the result for that
///     corresponding element will be `None`.
/// -   If both elements are `Some(f64)` but the `denominator` value is effectively `0.0`
///     (less than `f64::EPSILON`), the result for that element will be `None` (division by zero).
/// -   Otherwise, the division `Some(num / den)` is performed.
///
/// # Rationale for None Handling
/// This strict propagation of `None` ensures that any uncertainty or lack of data is carried
/// forward through the division operation. It prevents the generation of spurious numerical
/// results from missing data or undefined operations (like division by zero).
///
/// # Arguments
/// * `numerator` - The `CausalTensor<Option<f64>>` representing the numerator.
/// * `denominator` - The `CausalTensor<Option<f64>>` representing the denominator.
///
/// # Returns
/// A `Result` containing a new `CausalTensor<Option<f64>>` with the results of the element-wise
/// division, or a `CausalTensorError` if the tensor shapes do not match.
pub(crate) fn safe_div_cdl(
    numerator: &CausalTensor<Option<f64>>,
    denominator: &CausalTensor<Option<f64>>,
) -> Result<CausalTensor<Option<f64>>, CausalTensorError> {
    if numerator.shape() != denominator.shape() {
        return Err(CausalTensorError::ShapeMismatch);
    }

    let result_data: Vec<Option<f64>> = numerator
        .as_slice()
        .iter()
        .zip(denominator.as_slice().iter())
        .map(|(&num_opt, &den_opt)| match (num_opt, den_opt) {
            (Some(num), Some(den)) => {
                if den.abs() < f64::EPSILON {
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

/// Performs element-wise natural logarithm (base 2) for a `CausalTensor<Option<f64>>`.
///
/// This function calculates `log2` for each `Some(f64)` element in the input tensor.
///
/// # None Handling
/// -   If an element in the input tensor is `None`, the corresponding output element will be `None`.
/// -   If an element is `Some(val)` where `val` is `0.0` or negative, the result for that element
///     will be `None` (logarithm of non-positive number is undefined).
/// -   Otherwise, `Some(val.log2())` is performed.
///
/// # Rationale for None Handling
/// This strict propagation of `None` ensures that undefined logarithmic operations (from missing
/// data or non-positive values) do not produce spurious numerical results (like `NaN` or `Inf`)
/// that could then propagate incorrectly.
///
/// # Arguments
/// * `tensor` - The input `CausalTensor<Option<f64>>`.
///
/// # Returns
/// A `Result` containing a new `CausalTensor<Option<f64>>` with the results of the element-wise
/// `log2` operation, or a `CausalTensorError` if the operation fails.
pub(crate) fn surd_log2_cdl(
    tensor: &CausalTensor<Option<f64>>,
) -> Result<CausalTensor<Option<f64>>, CausalTensorError> {
    let result_data: Vec<Option<f64>> = tensor
        .as_slice()
        .iter()
        .map(|&val_opt| match val_opt {
            Some(val) => {
                if val > f64::EPSILON {
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

/// Performs element-wise multiplication for two `CausalTensor<Option<f64>>` tensors.
///
/// This function multiplies each element of the first tensor by the corresponding element
/// of the second tensor.
///
/// # None Handling
/// -   If either operand for an element-wise multiplication is `None`, the result for that
///     corresponding element will be `None`.
/// -   Otherwise, the multiplication `Some(v_a * v_b)` is performed.
///
/// # Rationale for None Handling
/// This strict propagation of `None` ensures that any uncertainty or lack of data is carried
/// forward through the multiplication operation, preventing spurious numerical results from
/// missing data.
///
/// # Arguments
/// * `a` - The first `CausalTensor<Option<f64>>` operand.
/// * `b` - The second `CausalTensor<Option<f64>>` operand.
///
/// # Returns
/// A `Result` containing a new `CausalTensor<Option<f64>>` with the results of the element-wise
/// multiplication, or a `CausalTensorError` if the tensor shapes do not match.
pub(crate) fn mul_cdl(
    a: &CausalTensor<Option<f64>>,
    b: &CausalTensor<Option<f64>>,
) -> Result<CausalTensor<Option<f64>>, CausalTensorError> {
    if a.shape() != b.shape() {
        return Err(CausalTensorError::ShapeMismatch);
    }

    let result_data: Vec<Option<f64>> = a
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

/// Performs element-wise subtraction for two `CausalTensor<Option<f64>>` tensors.
///
/// This function subtracts each element of the second tensor from the corresponding element
/// of the first tensor.
///
/// # None Handling
/// -   If either operand for an element-wise subtraction is `None`, the result for that
///     corresponding element will be `None`.
/// -   Otherwise, the subtraction `Some(v_a - v_b)` is performed.
///
/// # Rationale for None Handling
/// This strict propagation of `None` ensures that any uncertainty or lack of data is carried
/// forward through the subtraction operation, preventing spurious numerical results from
/// missing data.
///
/// # Arguments
/// * `a` - The first `CausalTensor<Option<f64>>` operand.
/// * `b` - The second `CausalTensor<Option<f64>>` operand.
///
/// # Returns
/// A `Result` containing a new `CausalTensor<Option<f64>>` with the results of the element-wise
/// subtraction, or a `CausalTensorError` if the tensor shapes do not match.
pub(crate) fn sub_cdl(
    a: &CausalTensor<Option<f64>>,
    b: &CausalTensor<Option<f64>>,
) -> Result<CausalTensor<Option<f64>>, CausalTensorError> {
    if a.shape() != b.shape() {
        return Err(CausalTensorError::ShapeMismatch);
    }

    let result_data: Vec<Option<f64>> = a
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
/// This function returns a new vector containing all elements from slice `a` that are not present in slice `b`.
///
/// # None Handling
/// This function operates on generic types `T` that implement `PartialEq` and `Clone`. It does not
/// directly handle `Option<f64>` values but can be used with slices of `usize` (like axis indices)
/// which are not `Option` types.
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
