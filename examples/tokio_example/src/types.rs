/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{BaseContext, Model, NumericalValue};

// Model<I, O, C> where:
// - I: Input type (NumericalValue)
// - O: Output type (bool)
// - C: Context type (BaseContext)
pub type BaseModelTokio = Model<NumericalValue, bool, BaseContext>;
