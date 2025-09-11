/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensor, CausalTensorError};

/// An extension trait for collections of tensors.
pub trait CausalTensorCollectionExt<T> {
    /// Stacks a sequence of tensors along a new axis to form a single, higher-dimensional tensor.
    ///
    /// This method takes a slice of `CausalTensor`s and combines them into a new `CausalTensor`
    /// with an additional dimension. The new dimension is inserted at the specified `axis`.
    ///
    /// # Arguments
    ///
    /// * `axis` - The axis along which to stack the tensors. This must be a valid axis
    ///   within the dimensions of the input tensors, or one greater than the
    ///   maximum existing axis to append a new dimension.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// * `Ok(CausalTensor<T>)` - The new stacked tensor if the operation was successful.
    /// * `Err(CausalTensorError)` - An error if the input tensors are empty, have mismatched shapes,
    ///   or if the specified `axis` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_data_structures::{CausalTensor, CausalTensorCollectionExt};
    /// let t1 = CausalTensor::new(vec![1, 2], vec![2]).unwrap();
    /// let t2 = CausalTensor::new(vec![3, 4], vec![2]).unwrap();
    /// let stacked_tensor = [t1, t2].stack(0).unwrap();
    /// assert_eq!(stacked_tensor.shape(), &[2, 2]);
    /// ```
    fn stack(&self, axis: usize) -> Result<CausalTensor<T>, CausalTensorError>;
}

// Implement the new trait for slices of CausalTensors.
impl<T> CausalTensorCollectionExt<T> for [CausalTensor<T>]
where
    T: Copy + Default + PartialOrd,
{
    fn stack(&self, axis: usize) -> Result<CausalTensor<T>, CausalTensorError> {
        if self.is_empty() {
            // Cannot determine a shape from an empty slice.
            return Err(CausalTensorError::EmptyTensor);
        }

        let first_shape = self[0].shape();
        if axis > first_shape.len() {
            return Err(CausalTensorError::AxisOutOfBounds);
        }

        // Validate that all tensors in the slice have the same shape.
        for tensor in self.iter().skip(1) {
            if tensor.shape() != first_shape {
                return Err(CausalTensorError::ShapeMismatch);
            }
        }

        // Calculate the shape of the resulting tensor.
        let mut result_shape = first_shape.to_vec();
        result_shape.insert(axis, self.len());
        let result_len: usize = result_shape.iter().product();
        let mut result_data = Vec::with_capacity(result_len);

        // This is the core, generalized stacking logic.
        // It iterates through the output coordinates and calculates which input
        // tensor and which position to pull the data from.
        let mut current_index = vec![0; result_shape.len()];
        for _ in 0..result_len {
            // The value at the specified `axis` in the output index tells us
            // which tensor to sample from in the input slice.
            let slice_index = current_index[axis];

            // The index into that source tensor is the output index with the
            // `axis` dimension removed.
            let mut source_index = current_index.clone();
            source_index.remove(axis);

            // Fetch the value. .unwrap() is safe due to the extensive checks above.
            let value = self[slice_index]
                .get(&source_index)
                .expect("Internal logic error: index should be valid");
            result_data.push(*value);

            // Increment the multi-dimensional index for the next output element.
            for j in (0..result_shape.len()).rev() {
                current_index[j] += 1;
                if current_index[j] < result_shape[j] {
                    break;
                }
                current_index[j] = 0;
            }
        }

        CausalTensor::new(result_data, result_shape)
    }
}
