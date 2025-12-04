/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalTensor, CausalTensorError};

impl<T> CausalTensor<T>
where
    T: Clone,
{
    pub(in crate::types::causal_tensor) fn reshape_impl(
        &self,
        new_shape: &[usize],
    ) -> Result<Self, CausalTensorError> {
        let new_len: usize = new_shape.iter().product();
        if new_len != self.len() {
            return Err(CausalTensorError::ShapeMismatch);
        }
        // This is a metadata-only operation, so we clone the data but re-calculate strides.
        Ok(Self::from_vec_and_shape_unchecked(
            self.data.clone(),
            new_shape,
        ))
    }

    pub(in crate::types::causal_tensor) fn ravel_impl(mut self) -> Self {
        let len = self.len();
        self.shape = vec![len];
        self.strides = vec![1];
        self
    }

    pub(in crate::types::causal_tensor) fn permute_axes_impl(
        &self,
        axes: &[usize],
    ) -> Result<Self, CausalTensorError> {
        if axes.len() != self.num_dim() {
            return Err(CausalTensorError::DimensionMismatch);
        }

        let mut seen_axes = vec![false; self.num_dim()];
        for &axis in axes {
            if axis >= self.num_dim() || seen_axes[axis] {
                return Err(CausalTensorError::InvalidParameter(
                    "Invalid axes permutation".to_string(),
                ));
            }
            seen_axes[axis] = true;
        }

        let mut new_shape = Vec::with_capacity(self.num_dim());
        for &axis in axes {
            new_shape.push(self.shape[axis]);
        }

        let mut new_strides = vec![0; new_shape.len()];
        if !new_shape.is_empty() {
            let mut current_stride = 1;
            for i in (0..new_shape.len()).rev() {
                new_strides[i] = current_stride;
                current_stride *= new_shape[i];
            }
        }

        // Create a new tensor with the same data but new shape and strides
        Ok(Self {
            data: self.data.clone(),
            shape: new_shape,
            strides: new_strides,
        })
    }
}
