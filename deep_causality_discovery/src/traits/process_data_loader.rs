/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::DataLoaderConfig;
use crate::errors::DataError;
use deep_causality_tensor::CausalTensor;

pub trait ProcessDataLoader {
    fn load(&self, path: &str, config: &DataLoaderConfig) -> Result<CausalTensor<f64>, DataError>;
}
