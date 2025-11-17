/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{BaseCausaloid, CausaloidGraph, EffectValue};

pub type DBNCausaloid = BaseCausaloid<EffectValue, EffectValue>;

pub type DBNGraph = CausaloidGraph<DBNCausaloid>;
