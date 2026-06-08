/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared cast from the file loaders' `f64` tensor into the pipeline precision `T`.

use crate::Precision;
use deep_causality_num::FromPrimitive;
use deep_causality_tensor::CausalTensor;

/// Casts a freshly-loaded `CausalTensor<f64>` (the precision the file loaders
/// produce) into the pipeline precision `T`. `NaN` is preserved as `T::nan()` so a
/// later cleaning stage can map it to `None`; all other values convert via
/// `from_f64`. The shape is preserved, so construction cannot fail.
pub(crate) fn cast_loaded_tensor<T: Precision>(tensor: CausalTensor<f64>) -> CausalTensor<T> {
    let shape = tensor.shape().to_vec();
    let data: Vec<T> = tensor
        .as_slice()
        .iter()
        .map(|&v| {
            if v.is_nan() {
                T::nan()
            } else {
                <T as FromPrimitive>::from_f64(v)
                    .expect("every RealField precision converts a loaded f64 value")
            }
        })
        .collect();
    CausalTensor::new(data, shape).expect("cast preserves shape; construction cannot fail")
}
