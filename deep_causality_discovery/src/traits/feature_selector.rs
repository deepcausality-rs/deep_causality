/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::FeatureSelectorConfig;
use crate::errors::FeatureSelectError;
use deep_causality_tensor::CausalTensor;

pub trait FeatureSelector {
    fn select(
        &self,
        tensor: CausalTensor<f64>,
        config: &FeatureSelectorConfig,
    ) -> Result<CausalTensor<f64>, FeatureSelectError>;
}
