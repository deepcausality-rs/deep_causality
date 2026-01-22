/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalDiscovery;
use crate::{CausalDiscoveryConfig, CausalDiscoveryError};
use deep_causality_algorithms::surd::{SurdResult, surd_states_cdl};
use deep_causality_tensor::{CausalTensor, Tensor};

/// A concrete implementation of the `CausalDiscovery` trait using the SURD algorithm.
///
/// This struct acts as a bridge between the CDL pipeline and the `surd_states`
/// algorithms from the `deep_causality_algorithms` crate.
pub struct SurdCausalDiscovery;

impl CausalDiscovery for SurdCausalDiscovery {
    fn discover(
        &self,
        tensor: CausalTensor<Option<f64>>,
        config: &CausalDiscoveryConfig,
    ) -> Result<SurdResult<f64>, CausalDiscoveryError> {
        let CausalDiscoveryConfig::Surd(surd_config) = config;
        Self::discover_res(&tensor, surd_config)
    }
}

impl SurdCausalDiscovery {
    pub fn discover_res(
        tensor: &CausalTensor<Option<f64>>,
        config: &crate::SurdConfig,
    ) -> Result<SurdResult<f64>, CausalDiscoveryError> {
        let target_col = config.target_col();
        let num_dims = tensor.num_dim();

        if target_col >= num_dims {
            return Err(CausalDiscoveryError::TensorError(
                deep_causality_tensor::CausalTensorError::InvalidParameter(format!(
                    "target_col {} is out of bounds for tensor with {} dimensions",
                    target_col, num_dims
                )),
            ));
        }

        // If target is not already at axis 0, permute axes to move it there
        let arranged_tensor = if target_col != 0 {
            // Create permutation: [target_col, 0, 1, ..., target_col-1, target_col+1, ..., num_dims-1]
            let mut axes: Vec<usize> = Vec::with_capacity(num_dims);
            axes.push(target_col); // Target goes to position 0
            for i in 0..num_dims {
                if i != target_col {
                    axes.push(i);
                }
            }

            let arranged_view = tensor.permute_axes(&axes)?;
            // Materialize the view into a contiguous tensor to ensure the algorithm sees the correct data order
            // (algorithms often iterate over raw data for performance, ignoring strides)
            CausalTensor::from_shape_fn(arranged_view.shape(), |idx| {
                *arranged_view.get(idx).expect("Index out of bounds")
            })
        } else {
            tensor.clone()
        };

        Ok(surd_states_cdl(&arranged_tensor, config.max_order())?)
    }
}
