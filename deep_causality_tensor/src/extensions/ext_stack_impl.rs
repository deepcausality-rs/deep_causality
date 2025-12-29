/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensor, CausalTensorError, CausalTensorStackExt};

impl<T> CausalTensorStackExt<T> for [CausalTensor<T>]
where
    T: Clone + Default + PartialOrd,
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
            result_data.push(value.clone());

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

impl<T> CausalTensorStackExt<T> for [crate::InternalCpuTensor<T>]
where
    T: Clone + Default + PartialOrd,
{
    fn stack(&self, axis: usize) -> Result<CausalTensor<T>, CausalTensorError> {
        if self.is_empty() {
            return Err(CausalTensorError::EmptyTensor);
        }

        let first_shape = self[0].shape();
        if axis > first_shape.len() {
            return Err(CausalTensorError::AxisOutOfBounds);
        }

        for tensor in self.iter().skip(1) {
            if tensor.shape() != first_shape {
                return Err(CausalTensorError::ShapeMismatch);
            }
        }

        let mut result_shape = first_shape.to_vec();
        result_shape.insert(axis, self.len());
        let result_len: usize = result_shape.iter().product();
        let mut result_data = Vec::with_capacity(result_len);

        let mut current_index = vec![0; result_shape.len()];
        for _ in 0..result_len {
            let slice_index = current_index[axis];
            let mut source_index = current_index.clone();
            source_index.remove(axis);

            let value = self[slice_index]
                .get(&source_index)
                .expect("Internal logic error");
            result_data.push(value.clone()); // InternalCpuTensor::get returns Option<&T>, so we clone

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
