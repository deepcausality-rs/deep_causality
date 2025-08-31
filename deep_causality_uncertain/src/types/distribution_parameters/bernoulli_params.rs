/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// Struct to hold the parameters for a Bernoulli distribution.
#[derive(Debug, Clone, Copy)]
pub struct BernoulliParams {
    pub p: f64, // probability of success
}
