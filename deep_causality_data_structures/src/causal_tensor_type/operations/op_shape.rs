/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalTensor, CausalTensorError};

impl<T> CausalTensor<T>
where
    T: Copy + Default + PartialOrd,
{
    pub(in crate::causal_tensor_type) fn reshape_impl(
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

    pub(in crate::causal_tensor_type) fn ravel_impl(mut self) -> Self {
        let len = self.len();
        self.shape = vec![len];
        self.strides = vec![1];
        self
    }
}
