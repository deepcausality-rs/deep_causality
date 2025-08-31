/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// A value produced during a sample run. Can be a float or a boolean.
#[derive(Debug, Clone, Copy)]
pub enum SampledValue {
    Float(f64),
    Bool(bool),
}
